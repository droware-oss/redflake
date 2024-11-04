use crate::frame::Frame::{Array, BulkString, Integer, Map, SimpleError, SimpleString};
use bytes::Buf;
use std::fmt::{Display, Formatter};
use std::io::{Cursor, ErrorKind, Result};
use std::str::FromStr;

const TERMINATOR: &[u8; 2] = b"\r\n";

pub enum Frame {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<Frame>),
    Map(Vec<(Frame, Frame)>),
}

impl Frame {
    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Frame> {
        if !cursor.has_remaining() {
            return Err(ErrorKind::NotFound.into());
        }
        match cursor.get_u8() {
            b'+' => Ok(SimpleString(read_string(cursor)?)),
            b'-' => Ok(SimpleError(read_string(cursor)?)),
            b':' => Ok(Integer(read_number(cursor)?)),
            b'$' => Ok(BulkString(read_binary(cursor)?)),
            b'*' => Ok(Array(read_array(cursor)?)),
            b'%' => Ok(Map(read_map(cursor)?)),
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        match self {
            SimpleString(s) => {
                vec.push(b'+');
                vec.extend_from_slice(s.as_bytes());
                vec.extend_from_slice(TERMINATOR);
            }
            SimpleError(e) => {
                vec.push(b'-');
                vec.extend_from_slice(e.as_bytes());
                vec.extend_from_slice(TERMINATOR);
            }
            Integer(i) => {
                vec.push(b':');
                vec.extend_from_slice(i.to_string().as_bytes());
                vec.extend_from_slice(TERMINATOR);
            }
            BulkString(b) => {
                vec.push(b'$');
                vec.extend_from_slice(b.len().to_string().as_bytes());
                vec.extend_from_slice(TERMINATOR);
                vec.extend_from_slice(b);
                vec.extend_from_slice(TERMINATOR);
            }
            Array(vals) => {
                vec.push(b'*');
                vec.extend_from_slice(vals.len().to_string().as_bytes());
                vec.extend_from_slice(TERMINATOR);
                for val in vals {
                    vec.extend(val.as_bytes());
                }
            }
            Map(items) => {
                vec.push(b'%');
                vec.extend_from_slice(items.len().to_string().as_bytes());
                vec.extend_from_slice(TERMINATOR);
                for (key, val) in items {
                    vec.extend(key.as_bytes());
                    vec.extend(val.as_bytes());
                }
            }
        }
        vec
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleString(s) => write!(f, "SimpleString({:?})", s),
            SimpleError(e) => write!(f, "SimpleError({:?})", e),
            Integer(i) => write!(f, "Integer({})", i),
            BulkString(b) => {
                write!(f, "BulkString(")?;
                if let Ok(s) = str::from_utf8(b) {
                    write!(f, "{:?}", s)?;
                } else {
                    write!(f, "{:?}", b)?;
                }
                write!(f, ")")
            }
            Array(vals) => {
                write!(f, "Array([")?;
                for (idx, val) in vals.iter().enumerate() {
                    write!(f, "{}", val)?;
                    if idx < vals.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "])")
            }
            Map(items) => {
                write!(f, "Map([")?;
                for (idx, item) in items.iter().enumerate() {
                    write!(f, "({}: {})", item.0, item.1)?;
                    if idx < items.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "])")
            }
        }
    }
}

pub fn string_from_binary(v: &[u8]) -> Result<String> {
    str::from_utf8(v)
        .map(String::from)
        .map_err(|_| ErrorKind::InvalidData.into())
}

fn read_string(cursor: &mut Cursor<&[u8]>) -> Result<String> {
    let start = cursor.position() as usize;
    let end = find_terminator(cursor)?;
    string_from_binary(&cursor.get_ref()[start..end])
}

pub fn number_from_binary<T>(v: &[u8]) -> Result<T>
where
    T: FromStr,
{
    str::from_utf8(v)
        .map_err(|_| ErrorKind::InvalidData.into())
        .and_then(|s| s.parse().map_err(|_| ErrorKind::InvalidData.into()))
}

fn read_number<T>(cursor: &mut Cursor<&[u8]>) -> Result<T>
where
    T: FromStr,
{
    let start = cursor.position() as usize;
    let end = find_terminator(cursor)?;
    number_from_binary(&cursor.get_ref()[start..end])
}

fn read_binary(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let end: usize = read_number(cursor)?;
    let len = end + TERMINATOR.len();
    if cursor.remaining() < len {
        return Err(ErrorKind::NotFound.into());
    }
    let vec = Vec::from(&cursor.chunk()[..end]);
    cursor.advance(len);
    Ok(vec)
}

fn read_array(cursor: &mut Cursor<&[u8]>) -> Result<Vec<Frame>> {
    let cnt: usize = read_number(cursor)?;
    let mut vec = Vec::with_capacity(cnt);
    for _ in 0..cnt {
        vec.push(Frame::parse(cursor)?);
    }
    Ok(vec)
}

fn read_map(cursor: &mut Cursor<&[u8]>) -> Result<Vec<(Frame, Frame)>> {
    let cnt: usize = read_number(cursor)?;
    let mut vec = Vec::with_capacity(cnt);
    for _ in 0..cnt {
        vec.push((Frame::parse(cursor)?, Frame::parse(cursor)?));
    }
    Ok(vec)
}

fn find_terminator(cursor: &mut Cursor<&[u8]>) -> Result<usize> {
    let buf = cursor.get_ref();
    for i in cursor.position() as usize..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            cursor.set_position((i + TERMINATOR.len()) as u64);
            return Ok(i);
        }
    }
    Err(ErrorKind::NotFound.into())
}
