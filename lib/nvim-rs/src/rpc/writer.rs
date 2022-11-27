use std::io;

use futures::prelude::*;

#[derive(Debug)]
pub enum WriteError {
    RmpSerde(rmp_serde::encode::Error),
    IO(io::Error),
}

impl From<rmp_serde::encode::Error> for WriteError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Self::RmpSerde(err)
    }
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

#[async_trait::async_trait(?Send)]
pub trait RpcWriter {
    async fn write_rpc_request<T: serde::Serialize>(
        &mut self,
        msgid: u32,
        method: &str,
        params: &T,
    ) -> Result<(), WriteError>;

    async fn write_rpc_response<R: serde::Serialize, E: serde::Serialize>(
        &mut self,
        msgid: u32,
        error: Option<&E>,
        result: Option<&R>,
    ) -> Result<(), WriteError>;

    async fn write_rpc_notification<T: serde::Serialize>(
        &mut self,
        method: &str,
        params: &T,
    ) -> Result<(), WriteError>;
}

#[async_trait::async_trait(?Send)]
impl<T> RpcWriter for T
where
    T: AsyncWrite + Unpin,
{
    async fn write_rpc_request<D: serde::Serialize>(
        &mut self,
        msgid: u32,
        method: &str,
        params: &D,
    ) -> Result<(), WriteError> {
        write_rpc(self, &(0, msgid, method, params)).await
    }

    async fn write_rpc_response<R: serde::Serialize, E: serde::Serialize>(
        &mut self,
        msgid: u32,
        error: Option<&E>,
        result: Option<&R>,
    ) -> Result<(), WriteError> {
        write_rpc(self, &(1, msgid, error, result)).await
    }

    async fn write_rpc_notification<D: serde::Serialize>(
        &mut self,
        method: &str,
        params: &D,
    ) -> Result<(), WriteError> {
        write_rpc(self, &(2, method, params)).await
    }
}

async fn write_rpc<W: AsyncWrite + Unpin, D: serde::Serialize>(
    w: &mut W,
    data: &D,
) -> Result<(), WriteError> {
    let mut buf = Vec::new();
    // Encode the message to a buffer which we can the write to the writer.
    rmp_serde::encode::write_named(&mut buf, data)?;

    w.write_all(&buf).await?;
    w.flush().await?;

    Ok(())
}
