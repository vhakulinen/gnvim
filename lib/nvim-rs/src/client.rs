use std::{
    any::{Any, TypeId},
    pin::Pin,
};

use futures::channel::oneshot;
use futures::prelude::*;

use serde::Deserialize;

use crate::rpc::{
    message::Response,
    writer::{RpcWriter, WriteError},
};

#[macro_export]
macro_rules! args {
    ($($x:expr),*) => {{
        ($($x,)*)
    }};
}

#[macro_export]
macro_rules! dict {
    ($($key:expr => $val:expr),*) => {{
        use $crate::types::Dictionary;
        Dictionary::new(vec![
            $(($key, $val),)*
        ])
    }};
}

#[derive(Debug)]
pub struct Client<W: RpcWriter> {
    writer: W,
    msgid_counter: u32,
    callbacks: Vec<(u32, oneshot::Sender<Response>)>,
}

#[derive(Debug, PartialEq)]
pub enum CallError {
    /// The operation was cancelled, because other end (of the internal channel)
    /// was dropped.
    Cancelled,
    /// The result field is missing.
    MissingResult,
    /// The call resulted into a error response (e.g. bad API call).
    Error(rmpv::Value),
    /// Decoding the result failed.
    DecodeResult,
    // Decoding the error failed.
    //DecodeError,
}

impl From<oneshot::Canceled> for CallError {
    fn from(_: oneshot::Canceled) -> Self {
        Self::Cancelled
    }
}

pub type CallResponse<T> = Pin<Box<dyn Future<Output = Result<T, CallError>> + Send>>;

impl<W: RpcWriter> Client<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            callbacks: Vec::new(),
            msgid_counter: 0,
        }
    }

    pub async fn call<T, S, V>(&mut self, method: S, args: V) -> Result<CallResponse<T>, WriteError>
    where
        T: for<'de> Deserialize<'de> + Any,
        S: AsRef<str>,
        V: serde::Serialize,
    {
        let msgid = self.msgid_counter;
        self.msgid_counter += 1;

        let (sender, receiver) = oneshot::channel();
        self.callbacks.push((msgid, sender));

        self.writer
            .write_rpc_request(msgid, method.as_ref(), &args)
            .await?;

        Ok(receiver
            .map(|res| {
                let res = res?;

                if let Some(error) = res.error {
                    // TODO(ville): Decode the error value.
                    return Err(CallError::Error(error));
                }

                // The type `()` is special in a sense that it signals voidness of the
                // returned result. But since the returned result doesn't necessarily exist,
                // we'll need to handle it our selves.
                let res = if res.result.is_none() && TypeId::of::<T>() == TypeId::of::<()>() {
                    rmpv::Value::Nil
                } else {
                    res.result.ok_or(CallError::MissingResult)?
                };

                rmpv::ext::from_value::<T>(res).map_err(|_err| CallError::DecodeResult)
            })
            .boxed())
    }

    pub fn handle_response(&mut self, response: Response) -> Result<(), HandleError> {
        let caller = self
            .callbacks
            .iter()
            .position(|(msgid, _)| *msgid == response.msgid)
            .map(|index| self.callbacks.swap_remove(index));

        match caller {
            Some((_, recv)) => recv.send(response).map_err(HandleError::CallerDropped),
            None => Err(HandleError::CallerMissing(response)),
        }
    }
}

#[derive(Debug)]
pub enum HandleError {
    /// The message was a response, but the caller wasn't found based on the
    /// response's id.
    CallerMissing(Response),
    /// The message was a response and the caller was found but it was dropped
    /// and thus' the response couldn't be delivered.
    CallerDropped(Response),
}
