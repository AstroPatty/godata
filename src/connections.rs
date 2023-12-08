use tokio::net::TcpStream;
use bytes::{BytesMut, Buf};
use std::io::{Cursor, Result};
use tokio::io::AsyncReadExt;
use crate::commands;
use tokio::io::AsyncWriteExt;
pub(crate) struct Connection {
    stream: TcpStream,
    buffer: BytesMut
}

impl Connection {
    pub(crate) fn new (stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(4096)
        }
    }
    pub(crate) async fn read_frame(&mut self) -> Result<commands::GodataCommand> {
        loop {

            // Attempt to parse a frame from the buffered data. If
            // enough data has been buffered, the frame is returned.
            if let Some(cmd) = self.parse_frame()? {
                return Ok(cmd);
            }

            // There is not enough buffered data to read a frame.
            // Attempt to read more data from the socket.
            //
            // On success, the number of bytes is returned. `0`
            // indicates "end of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "broken pipe",
                ));
            }

        }
    }
    pub(crate) async fn send_response(&mut self, response: String) -> Result<()> {
        let result = self.stream.write_all(response.as_bytes());
        result.await
    }

    fn parse_frame(&mut self) -> Result<Option<commands::GodataCommand>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        if !buf.has_remaining() {
            return Ok(None);
        }

        match buf.get_u8() {
            b'*' => {
                let val = get_line(&mut buf)?;
                let len = buf.position() as usize;
        
                // Parse into a string
                let cmd = String::from_utf8(val.to_vec()).unwrap();
                self.buffer.advance(len);
                // Remove the parsed frame from the buffer
                let cmd = commands::parse_command(&cmd)?;
                Ok(Some(cmd))
            }
            _ => Ok(None)
        }
    }
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8]> {
    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 2) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "no line ending found",
    ))
}