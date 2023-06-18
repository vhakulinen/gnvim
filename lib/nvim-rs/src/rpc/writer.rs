use std::io;

use futures::prelude::*;

use super::message::{Notification, Request, Response};

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
pub trait RpcWriter
where
    Self: AsyncWrite + Unpin + Sized,
{
    async fn write_rpc_request<D: serde::Serialize>(
        self,
        msgid: u32,
        method: &str,
        params: &D,
    ) -> Result<(), WriteError> {
        self.write_rpc(&Request::new(msgid, method, params)).await
    }

    async fn write_rpc_response<R: serde::Serialize, E: serde::Serialize>(
        self,
        msgid: u32,
        error: Option<&E>,
        result: Option<&R>,
    ) -> Result<(), WriteError> {
        self.write_rpc(&Response::new(msgid, error, result)).await
    }

    async fn write_rpc_notification<D: serde::Serialize>(
        self,
        method: &str,
        params: &D,
    ) -> Result<(), WriteError> {
        self.write_rpc(&Notification::new(method, params)).await
    }

    async fn write_rpc<T: serde::Serialize>(self, msg: &T) -> Result<(), WriteError> {
        self.write(&Self::encode(msg)?).await
    }

    async fn write(mut self, buf: &[u8]) -> Result<(), WriteError> {
        self.write_all(&buf).await?;
        self.flush().await?;

        Ok(())
    }

    fn encode<T: serde::Serialize>(data: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let mut buf = Vec::new();
        rmp_serde::encode::write_named(&mut buf, data)?;
        Ok(buf)
    }
}

#[async_trait::async_trait(?Send)]
impl<T> RpcWriter for T where T: AsyncWrite + Unpin {}
