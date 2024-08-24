use std::{collections::VecDeque, io};

use futures::prelude::*;

use super::message::Message;

/// Cursor implementing non-destructive read for `VecDeque`.
pub(crate) struct Cursor<'a> {
    inner: &'a VecDeque<u8>,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(deque: &'a VecDeque<u8>) -> Self {
        Self {
            inner: deque,
            pos: 0,
        }
    }
}

impl<'a> io::Read for Cursor<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let start = self.pos.min(self.inner.len());

        let (front, back) = self.inner.as_slices();
        let n = if start < front.len() {
            io::Read::read(&mut (&front[start..]), buf)?
        } else {
            io::Read::read(&mut (&back[(start - front.len())..]), buf)?
        };

        self.pos += n;
        std::io::Result::Ok(n)
    }
}

#[derive(Debug)]
pub enum ReadError {
    IOError(io::Error),
    RmpError(rmp_serde::decode::Error),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::IOError(err) => f.write_fmt(format_args!("io error: {}", err)),
            ReadError::RmpError(err) => f.write_fmt(format_args!("rmp error: {}", err)),
        }
    }
}

pub struct RpcReader<R>
where
    R: AsyncRead + Unpin,
{
    reader: futures::io::BufReader<R>,
    buf: VecDeque<u8>,
}

impl<R> RpcReader<R>
where
    R: AsyncRead + Unpin,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader: futures::io::BufReader::new(reader),
            buf: VecDeque::new(),
        }
    }

    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }

    async fn fill_buffer(&mut self) -> Result<(), ReadError> {
        match self.reader.fill_buf().await {
            Ok([]) => Err(ReadError::IOError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Read zero bytes",
            ))),
            Ok(bytes) => {
                // Add the available bytes to our buffer.
                self.buf.extend(bytes);

                // Tell the reader that we consumed the values.
                let len = bytes.len();
                self.reader.consume_unpin(len);
                Ok(())
            }
            Err(err) => Err(ReadError::IOError(err)),
        }
    }

    pub async fn recv(&mut self) -> Result<Message, ReadError> {
        loop {
            let mut cursor = Cursor::new(&self.buf);

            // Try decoding value from the buffer's current content.
            match rmp_serde::from_read::<_, Message>(&mut cursor) {
                Ok(val) => {
                    // All good, there was enough data. Drop the read data.
                    self.buf.drain(..cursor.pos);

                    return Ok(val);
                }
                // If we got an UnexpectedEof error, try reading more data into
                // the buffer.
                Err(rmp_serde::decode::Error::InvalidMarkerRead(err))
                | Err(rmp_serde::decode::Error::InvalidDataRead(err))
                    if err.kind() == io::ErrorKind::UnexpectedEof =>
                {
                    self.fill_buffer().await?
                }
                Err(err) => {
                    return Err(ReadError::RmpError(err));
                }
            }
        }
    }
}

impl<R> From<R> for RpcReader<R>
where
    R: AsyncRead + Unpin,
{
    fn from(r: R) -> Self {
        RpcReader::new(r)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        io::{Read, Write},
    };

    use super::Cursor;

    #[test]
    fn test_cursor_read_wrapping() {
        let mut buf = VecDeque::with_capacity(5);
        buf.write_all(&[1, 2, 3, 4, 5]).unwrap();
        buf.drain(..2);
        buf.write_all(&[6]).unwrap();

        // Buffer should be [6, <empty>, 3, 4, 5]
        let (front, back) = buf.as_slices();
        assert_eq!(&[3, 4, 5], front);
        assert_eq!(&[6], back);

        let mut cursor = Cursor::new(&buf);
        let mut target = Vec::new();
        let n = cursor.read_to_end(&mut target).unwrap();
        assert_eq!(4, n);
        assert_eq!(vec![3, 4, 5, 6], target);
    }
}
