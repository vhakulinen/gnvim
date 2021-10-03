use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use gtk::gio;
use pin_project::pin_project;

use crate::thread_guard::ThreadGuard;

#[pin_project]
pub struct Compat<T> {
    #[pin]
    inner: ThreadGuard<T>,
}

impl<T> Compat<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: ThreadGuard::new(inner),
        }
    }
}

impl futures::io::AsyncRead
    for Compat<gio::InputStreamAsyncRead<gio::PollableInputStream>>
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        gio::InputStreamAsyncRead::poll_read(
            Pin::new(&mut *(self.project().inner.borrow_mut())),
            cx,
            buf,
        )
    }
}

impl futures::io::AsyncWrite
    for Compat<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>>
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        gio::OutputStreamAsyncWrite::poll_write(
            Pin::new(&mut *(self.project().inner.borrow_mut())),
            cx,
            buf,
        )
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        gio::OutputStreamAsyncWrite::poll_close(
            Pin::new(&mut *(self.project().inner.borrow_mut())),
            cx,
        )
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        gio::OutputStreamAsyncWrite::poll_flush(
            Pin::new(&mut *(self.project().inner.borrow_mut())),
            cx,
        )
    }
}
