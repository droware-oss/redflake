use crate::frame::Frame;
use bytes::{Buf, BytesMut};
use std::io::{Cursor, ErrorKind, Result};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tracing::info;

const BUFFER_SIZE: usize = 128;

#[derive(Debug)]
pub enum Protocol {
    RESP2,
    RESP3,
}

impl TryFrom<u8> for Protocol {
    type Error = String;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        if value == 2 {
            Ok(Self::RESP2)
        } else if value == 3 {
            Ok(Self::RESP3)
        } else {
            Err("Unsupported protocol version".to_string())
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    addr: SocketAddr,
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    pub protocol: Protocol,
}

impl Connection {
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            addr,
            buffer: BytesMut::with_capacity(BUFFER_SIZE),
            protocol: Protocol::RESP2,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        info!("Reading data from {}", self.addr);
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                info!("Client from {} closed", self.addr);
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(ErrorKind::ConnectionReset.into())
                };
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut cursor = Cursor::new(&self.buffer[..]);
        match Frame::parse(&mut cursor) {
            Ok(frame) => {
                if cursor.has_remaining() {
                    let position = cursor.position() as usize;
                    let remaining = cursor.remaining();
                    self.buffer.copy_within(position..position + remaining, 0);
                    self.buffer.truncate(remaining);
                } else {
                    self.buffer.clear();
                }
                Ok(Some(frame))
            }
            Err(err) => {
                if err.kind() == ErrorKind::InvalidData || cursor.get_ref().len() == BUFFER_SIZE {
                    self.buffer.clear();
                    Err(err)
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        self.stream.write_all(frame.as_bytes().as_slice()).await?;
        self.stream.flush().await
    }
}
