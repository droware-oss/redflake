use crate::connection::{Connection, Protocol};
use crate::frame::{number_from_binary, string_from_binary, Frame};
use crate::snowflake::next_id;
use std::io::{ErrorKind, Result};

#[derive(Debug)]
pub enum Command {
    AUTH,
    CLIENT,
    HELLO(Option<u8>),
    NEXT,
    SELECT,
    UNKNOWN(String),
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command> {
        match frame {
            Frame::Array(frames) => {
                if frames.is_empty() {
                    Err(ErrorKind::InvalidData.into())
                } else {
                    match &frames[0] {
                        Frame::BulkString(b) => string_from_binary(b)
                            .and_then(|s| match s.to_ascii_lowercase().as_str() {
                                "auth" => Ok(Command::AUTH),
                                "client" => Ok(Command::CLIENT),
                                "hello" => {
                                    if frames.len() > 1 {
                                        match &frames[1] {
                                            Frame::BulkString(protocol) => {
                                                match number_from_binary(protocol) {
                                                    Ok(version) => Ok(Command::HELLO(Some(version))),
                                                    Err(_) => Ok(Command::UNKNOWN("Protocol version is not an integer or out of range".to_string())),
                                                }
                                            }
                                            _ => Ok(Command::UNKNOWN("Protocol version is not an integer or out of range".to_string())),
                                        }
                                    } else {
                                        Ok(Command::HELLO(None))
                                    }
                                }
                                "next" => Ok(Command::NEXT),
                                "select" => Ok(Command::SELECT),
                                _ => Ok(Command::UNKNOWN("Unknown command".to_string())),
                            }),
                        _ => Ok(Command::UNKNOWN("Unknown command".to_string())),
                    }
                }
            }
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }

    pub async fn apply(self, conn: &mut Connection) -> Frame {
        match self {
            Command::AUTH => Frame::SimpleString("OK".to_string()),
            Command::CLIENT => Frame::SimpleString("OK".to_string()),
            Command::HELLO(protocol_version) => {
                if let Some(version) = protocol_version {
                    match Protocol::try_from(version) {
                        Ok(protocol) => conn.protocol = protocol,
                        Err(error) => return Frame::SimpleError(error),
                    }
                }
                match conn.protocol {
                    Protocol::RESP2 => Frame::Array(vec![
                        Frame::SimpleString("server".to_string()),
                        Frame::SimpleString(env!("CARGO_PKG_NAME").to_string()),
                        Frame::SimpleString("version".to_string()),
                        Frame::SimpleString(env!("CARGO_PKG_VERSION").to_string()),
                        Frame::SimpleString("proto".to_string()),
                        Frame::Integer(2),
                    ]),
                    Protocol::RESP3 => Frame::Map(vec![
                        (
                            Frame::SimpleString("server".to_string()),
                            Frame::SimpleString(env!("CARGO_PKG_NAME").to_string()),
                        ),
                        (
                            Frame::SimpleString("version".to_string()),
                            Frame::SimpleString(env!("CARGO_PKG_VERSION").to_string()),
                        ),
                        (Frame::SimpleString("proto".to_string()), Frame::Integer(3)),
                    ]),
                }
            }
            Command::NEXT => next_id()
                .map(|id| Frame::Integer(id))
                .unwrap_or_else(|err| Frame::SimpleError(err)),
            Command::SELECT => Frame::SimpleString("OK".to_string()),
            Command::UNKNOWN(error) => Frame::SimpleError(format!("ERR {}", error)),
        }
    }
}
