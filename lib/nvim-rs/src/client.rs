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
    writer: W,
    msgid_counter: u32,
    callbacks: Vec<(u32, Sender)>,
}

impl<W: RpcWriter> Client<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            callbacks: Vec::new(),
            msgid_counter: 0,
        }
    }

    pub fn handle_response(
        &mut self,
        response: Response<rmpv::Value, rmpv::Value>,
    ) -> Result<(), HandleError> {
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

impl<W: RpcWriter> AsMut<W> for Client<W> {
    fn as_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

#[async_trait::async_trait(?Send)]
impl<W: futures::AsyncWrite + Unpin> Caller for &mut Client<W> {
    fn next_msgid(&mut self) -> u32 {
        let msgid = self.msgid_counter;
        self.msgid_counter += 1;

        msgid
    }

    fn store_handler(&mut self, msgid: u32, sender: Sender) {
        self.callbacks.push((msgid, sender));
    }

    async fn write<S: AsRef<str>, V: serde::Serialize>(
        self,
        msgid: u32,
        method: S,
        args: V,
    ) -> Result<(), WriteError> {
        self.as_mut()
            .write_rpc_request(msgid, method.as_ref(), &args)
            .await
    }
}
