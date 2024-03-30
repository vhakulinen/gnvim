use std::any::{Any, TypeId};

use futures::{channel::oneshot, prelude::*};
use serde::Deserialize;

use crate::rpc::{message, WriteError};

#[derive(Debug)]
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
    /// The write operation over RPC failed.
    WriteError(WriteError),
}

pub type CallResponse<T> = Result<T, CallError>;

pub type Response = message::Response<rmpv::Value, rmpv::Value>;

pub type Sender = oneshot::Sender<Response>;

impl From<oneshot::Canceled> for CallError {
    fn from(_: oneshot::Canceled) -> Self {
        Self::Cancelled
    }
}

impl From<WriteError> for CallError {
    fn from(value: WriteError) -> Self {
        Self::WriteError(value)
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

#[async_trait::async_trait(?Send)]
pub trait Caller
where
    Self: Sized,
{
    async fn call<T, S, V>(mut self, method: S, args: V) -> CallResponse<T>
    where
        T: for<'de> Deserialize<'de> + Any,
        S: AsRef<str>,
        V: serde::Serialize,
    {
        let msgid = self.next_msgid();

        let (sender, receiver) = oneshot::channel();
        self.store_handler(msgid, sender);

        self.write(msgid, method, &args).await?;

        receiver
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
            .boxed()
            .await
    }

    async fn write<S: AsRef<str>, V: serde::Serialize>(
        self,
        msgid: u32,
        method: S,
        args: V,
    ) -> Result<(), WriteError>;

    fn next_msgid(&mut self) -> u32;

    fn store_handler(&mut self, msgid: u32, sender: Sender);
}
