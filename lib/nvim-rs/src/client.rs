use std::cell::RefCell;

use futures::lock::Mutex;

use crate::rpc::{caller::Sender, message::Response, Caller, HandleError, RpcWriter, WriteError};

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
    writer: Mutex<Box<W>>,
    msgid_counter: RefCell<u32>,
    callbacks: RefCell<Vec<(u32, Sender)>>,
}

impl<W: RpcWriter> Client<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: Mutex::new(Box::new(writer)),
            callbacks: RefCell::new(Vec::new()),
            msgid_counter: RefCell::new(0),
        }
    }

    pub fn handle_response(
        &self,
        response: Response<rmpv::Value, rmpv::Value>,
    ) -> Result<(), HandleError> {
        let mut callbacks = self.callbacks.borrow_mut();
        let caller = callbacks
            .iter()
            .position(|(msgid, _)| *msgid == response.msgid)
            .map(|index| callbacks.swap_remove(index));

        match caller {
            Some((_, recv)) => recv.send(response).map_err(HandleError::CallerDropped),
            None => Err(HandleError::CallerMissing(response)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<W: futures::AsyncWrite + Unpin> Caller for &Client<W> {
    fn next_msgid(&mut self) -> u32 {
        let mut msgid_counter = self.msgid_counter.borrow_mut();
        let msgid = *msgid_counter;
        *msgid_counter += 1;

        msgid
    }

    fn store_handler(&mut self, msgid: u32, sender: Sender) {
        self.callbacks.borrow_mut().push((msgid, sender));
    }

    async fn write<S: AsRef<str>, V: serde::Serialize>(
        self,
        msgid: u32,
        method: S,
        args: V,
    ) -> Result<(), WriteError> {
        let mut writer = self.writer.lock().await;
        writer
            .as_mut()
            .write_rpc_request(msgid, method.as_ref(), &args)
            .await
    }
}
