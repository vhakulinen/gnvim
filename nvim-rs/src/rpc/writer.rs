use std::io;

use futures::prelude::*;
use serde::Serialize;

use super::message::{Request, Response};

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

#[async_trait::async_trait]
pub trait RpcWriter {
    async fn write_rpc_request(&mut self, req: &Request) -> Result<(), WriteError>;
    async fn write_rpc_response(&mut self, res: &Response) -> Result<(), WriteError>;
}

#[async_trait::async_trait]
impl<T> RpcWriter for T
where
    T: AsyncWrite + Unpin + Send,
{
    async fn write_rpc_request(&mut self, req: &Request) -> Result<(), WriteError> {
        write_rpc(self, req).await
    }

    async fn write_rpc_response(&mut self, res: &Response) -> Result<(), WriteError> {
        write_rpc(self, res).await
    }
}

async fn write_rpc<W: AsyncWrite + Unpin + Send, D: Serialize>(
    w: &mut W,
    data: &D,
) -> Result<(), WriteError> {
    let mut buf = Vec::new();
    // Encode the message to a buffer which we can the write to the writer.
    rmp_serde::encode::write(&mut buf, data)?;

    w.write_all(&buf).await?;
    w.flush().await?;

    Ok(())
}
