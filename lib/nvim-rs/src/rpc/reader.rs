use std::io;

use futures::prelude::*;

use super::message::Message;

#[derive(Debug)]
pub enum ReadError {
    IOError(io::Error),
    RmpError(rmp_serde::decode::Error),
}

pub struct RpcReader<R>
where
    R: AsyncRead + Unpin,
{
    reader: futures::io::BufReader<R>,
    buf: Vec<u8>,
}

impl<R> RpcReader<R>
where
    R: AsyncRead + Unpin,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader: futures::io::BufReader::new(reader),
            buf: Vec::new(),
        }
    }

    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }

    async fn fill_buffer(&mut self) -> Result<(), ReadError> {
        match self.reader.fill_buf().await {
            Ok(bytes) if bytes.is_empty() => Err(ReadError::IOError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Read zero bytes",
            ))),
            Ok(bytes) => {
                // Add the available bytes to our buffer.
                self.buf.extend_from_slice(bytes);

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
            let mut cursor = std::io::Cursor::new(&self.buf);

            // Try decoding value from the buffer's current content.
            match rmp_serde::from_read::<_, Message>(&mut cursor) {
                Ok(val) => {
                    // All good, there was enough data. Drop the read data.
                    let at = cursor.position() as usize;
                    self.buf = self.buf.split_off(at);

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
