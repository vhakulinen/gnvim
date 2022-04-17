use std::pin::Pin;

use futures::io::{AsyncRead, AsyncWrite};
use gio;

#[derive(Debug)]
pub struct CompatWrite {
    inner: gio::OutputStreamAsyncWrite<gio::PollableOutputStream>,
}

impl From<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>> for CompatWrite {
    fn from(inner: gio::OutputStreamAsyncWrite<gio::PollableOutputStream>) -> Self {
        Self { inner }
    }
}

impl AsyncWrite for CompatWrite {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

#[derive(Debug)]
pub struct CompatRead {
    inner: gio::InputStreamAsyncRead<gio::PollableInputStream>,
}

impl From<gio::InputStreamAsyncRead<gio::PollableInputStream>> for CompatRead {
    fn from(inner: gio::InputStreamAsyncRead<gio::PollableInputStream>) -> Self {
        Self { inner }
    }
}

impl AsyncRead for CompatRead {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}
