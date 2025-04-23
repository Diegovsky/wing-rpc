//! Rust runtime crate to talk with other `Wing RPC` peers.
//!
//! ## Compiler
//! `Wing RPC` follows a specific pattern for messages. For completeness and future extensibility, use the [wing compiler].
//!
//! [wing compiler]: https://crates.io/crates/wing_compiler
//!
//! To quickly get started, try this example:
//! <details>
//! <summary> Example code </summary>
//!
//! ```no_run
#![doc=include_str!("../examples/hello.rs")]
//! ```
//!
//! </details>
//!
//! ## Features
//! - `client`: Enables the [`client`] module.
//! - `server`: Enables the [`server`] module.
use std::{
    io::{self, BufReader, BufWriter, ErrorKind, Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

use bytemuck::Pod;
use derive_more::From;
use easy_ext::ext;
use pack1::U16LE;

use serde::{Deserialize, Serialize};
use simple_coro::{self, Coro, CoroState, Handle};
use thiserror::Error;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WireHeader {
    flags: u8,
    len: u16,
}

#[ext]
impl Handle<usize, Vec<u8>> {
    async fn bytes_for<N: Pod>(&mut self) -> N {
        let n = size_of::<N>();
        let buf = self.yield_value(n).await;
        *bytemuck::from_bytes(buf.as_ref())
    }
}

impl WireHeader {
    async fn read_sansio(mut handle: Handle<usize, Vec<u8>>) -> Self {
        let flags: u8 = handle.bytes_for().await;
        let len: U16LE = handle.bytes_for().await;
        Self {
            flags,
            len: len.get(),
        }
    }
    pub fn write(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.push(self.flags);
        buf.extend_from_slice(&self.len.to_le_bytes());
        buf
    }
    /// Reads byte-by-byte, so it's better to wrap it in a [`BufReader`]
    pub fn read(read: &mut dyn io::Read) -> io::Result<Self> {
        let mut coro = Coro::from(Self::read_sansio);
        let mut buf = vec![0; 16];
        loop {
            coro = match coro.resume() {
                CoroState::Pending(c, n) => {
                    buf.clear();
                    buf.resize(n, 0);
                    read.read_exact(&mut buf)?;
                    c.send(buf.clone())
                }
                CoroState::Complete(this) => return Ok(this),
            }
        }
    }
}

/// This crate's [`std::error::Error`] type.
#[derive(Debug, Error, From)]
pub enum Error {
    #[error(transparent)]
    Io(io::Error),
    #[error(transparent)]
    Serde(serde_json::Error),
}

impl Error {
    pub fn is_would_block(&self) -> bool {
        return matches!(self, Self::Io(e) if e.kind() == ErrorKind::WouldBlock);
    }
}

/// An active connection to a `Wing RPC` peer.
pub struct Peer {
    read: Box<dyn Read + Send>,
    write: Box<dyn Write + Send>,
    buf: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct WrappedData<T> {
    #[serde(rename = "type")]
    typ: String,
    data: T,
}

impl<'a, T: Message<'a>> WrappedData<T> {
    pub fn wrap(data: T) -> Self {
        Self {
            typ: T::NAME.into(),
            data,
        }
    }
}

/// A trait for types that can be sent and received through a [`Peer`].
pub trait Message<'a>: Serialize + Deserialize<'a> {
    const NAME: &'static str;
}

impl<'a, M: Message<'a>> Message<'a> for &M
where
    for<'any> &'any M: Deserialize<'a> + Serialize,
{
    const NAME: &'static str = M::NAME;
}

/// Useful type alias for fallible operations.
pub type WingResult<T> = Result<T, Error>;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Timeout {
    #[default]
    Block,
    DontBlock,
    WaitFor(Duration),
}

impl From<Duration> for Timeout {
    fn from(value: Duration) -> Self {
        Self::WaitFor(value)
    }
}

impl Timeout {
    fn retry_for<T>(&self, mut retry: impl FnMut() -> io::Result<T>) -> io::Result<Option<T>> {
        let timeout = match self {
            Timeout::Block => Duration::MAX,
            Timeout::DontBlock => Duration::ZERO,
            Timeout::WaitFor(duration) => duration.clone(),
        };
        let now = Instant::now();
        let time = Duration::from_millis(1);
        loop {
            match retry() {
                Ok(o) => return Ok(Some(o)),
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    if now.elapsed() < timeout {
                        std::thread::sleep(time);
                        continue;
                    } else {
                        return Ok(None);
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

// useful macro gotten from here: https://users.rust-lang.org/t/try-operator-for-result-option-t-e/74187/2
// I just wish rust had this builtin as an operator or something
macro_rules! try_harder {
    ($x:expr) => {
        match $x {
            Ok(Some(value)) => value,
            Ok(None) => return Ok(None),
            Err(error) => return Err(error.into()),
        }
    };
}

impl Peer {
    /// Creates a new [`Peer`] which sends messages to `write` and receives from `read`.
    ///
    /// This is the most general API, intended to allow every use case, but it's also a bit verbose,
    /// which is why a simpler api for sockets is provided: [`Peer::from_socket`].
    pub fn new(read: impl Read + Send + 'static, write: impl Write + Send + 'static) -> Self {
        Self {
            read: Box::new(read),
            write: Box::new(write),
            buf: Vec::new(),
        }
    }
    /// Creates a new [`Peer`] from a TCP connection.
    ///
    /// Check out [`std::net::TcpStream`] and [`std::net::TcpListener`] on how to create them.
    pub fn from_socket(con: TcpStream) -> Self {
        let read = Box::new(BufReader::new(con.try_clone().unwrap()));
        let write = Box::new(BufWriter::new(con));
        Self::new(read, write)
    }
    pub fn send<'a>(&mut self, message: impl Message<'a>) -> Result<(), Error> {
        let data = serde_json::to_vec(&WrappedData::wrap(message))?;
        let header = WireHeader {
            flags: 0,
            len: data.len() as _,
        };
        self.write.write_all(header.write().as_slice())?;
        self.write.write_all(data.as_slice())?;
        self.write.flush()?;
        Ok(())
    }
    pub fn recv<'a, T: Message<'a>>(&'a mut self) -> Result<T, Error> {
        self.try_recv(Timeout::Block)
            .map(|h| h.expect("Blocked operation returned None"))
    }
    pub fn try_recv<'a, T: Message<'a>>(
        &'a mut self,
        timeout: Timeout,
    ) -> Result<Option<T>, Error> {
        let header = try_harder!(timeout.retry_for(|| WireHeader::read(&mut self.read)));
        self.buf.resize(header.len as _, 0);
        try_harder!(timeout.retry_for(|| self.read.read_exact(&mut self.buf)));
        let msg = serde_json::from_slice::<WrappedData<T>>(self.buf.as_slice())?;
        Ok(Some(msg.data))
    }
}
