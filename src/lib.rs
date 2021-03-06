#![feature(macro_rules)]

use std::io::{MemReader, Reader, Writer, IoResult, IoError};
use std::io::net::tcp::TcpStream;
use std::string::String;

macro_rules! try_io {
    ($expr:expr) => (
        match $expr {
            Err(err) => {
                return Err(Error {
                    kind: InternalIoError(err),
                    desc: "Operation failed because of an IO error",
                    detail: None
                });
            },
            Ok(x) => x,
        }
    )
}

#[deriving(Show)]
pub enum ErrorKind {
    ResponseError,
    InternalIoError(IoError),
}

#[deriving(Show)]
pub struct Error {
    pub kind: ErrorKind,
    pub desc: &'static str,
    pub detail: Option<String>,
}

pub type NSQResult<T> = Result<T, Error>;

#[deriving(Show)]
pub struct Frame {
    frame_type: u32,
    body: Vec<u8>,
}

pub struct Connection {
    sock: TcpStream,
}

impl Connection {
    pub fn new(addr: &str, port: u16) -> IoResult<Connection> {
        let sock = try!(TcpStream::connect(addr, port));

        let mut ret = Connection {
            sock: sock,
        };

        try!(ret.send(b"  V2"));

        Ok(ret)
    }

    pub fn send(&mut self, data: &[u8]) -> IoResult<()> {
        let w = &mut self.sock as &mut Writer;
        w.write(data.as_slice())
    }

    pub fn read_frame(&mut self) -> NSQResult<Frame> {
        let r = &mut self.sock as &mut Reader;
        let size = try_io!(r.read_be_u32());
        let data = try_io!(r.read_exact(size as uint));
        let mut frame = MemReader::new(data);
        let frame_type = try_io!(frame.read_be_u32());
        let body = try_io!(frame.read_to_end());
        if frame_type == 0x01 {
            return Err(Error {
                kind: ResponseError,
                desc: "failed to read frame",
                detail: Some(String::from_utf8(body).unwrap()),
            });
        }
        let ret = Frame {
            frame_type: frame_type,
            body: body,
        };
        Ok(ret)
    }
}
