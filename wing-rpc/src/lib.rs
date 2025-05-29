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
// #![doc=include_str!("../examples/hello.rs")]
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

use derive_more::From;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

mod sansio;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WirePacket<'a> {
    flags: u8,
    len: u16,
    data: &'a [u8],
}

impl<'a> WirePacket<'a> {
    fn new(flags: u8, data: &'a [u8]) -> Self {
        Self {
            flags,
            len: data.len() as u16,
            data,
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

/// An active connection to a `Wing RPC` peer.
pub struct Peer {
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
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
    // fn retry_for2<'a, T: 'a>(
    //     &self,
    //     mut retry: impl FnMut() -> io::Result<T>,
    // ) -> io::Result<Option<T>> {
    //     loop {
    //         match polonius::<_, _, ForLt!(T)>(&mut retry, |retry| match retry() {
    //             Ok(val) => PoloniusResult::Borrowing(val),
    //             Err(e) => PoloniusResult::Owned {
    //                 value: e,
    //                 input_borrow: Placeholder,
    //             },
    //         }) {
    //             PoloniusResult::Borrowing(o) => return Ok(Some(o)),
    //             PoloniusResult::Owned {
    //                 value: err,
    //                 input_borrow: retry2,
    //             } => retry = retry2,
    //         }
    //     }
    // }
    fn retry_for<'a, T: 'a, A>(
        &self,
        retry: fn(&'a mut A) -> io::Result<T>,
        args: &'a mut A,
    ) -> io::Result<Option<T>> {
        let timeout = match self {
            Timeout::Block => Duration::MAX,
            Timeout::DontBlock => Duration::ZERO,
            Timeout::WaitFor(duration) => duration.clone(),
        };
        let now = Instant::now();
        let time = Duration::from_millis(1);
        let args = args as *mut A;
        loop {
            // SAFETY: This is safe because we guarantee the data is borrowed only on the Ok variant,
            // and then not ran again.
            match retry(unsafe { &mut *args }) {
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
            reader: Box::new(read),
            writer: Box::new(write),
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
        let header = WirePacket::new(0, data.as_slice());
        header.write(&mut self.buf, &mut self.writer)?;
        self.writer.flush()?;
        Ok(())
    }
    /// Waits for a message of `T` to arrive.
    ///
    /// This operation blocks the current thread while waiting for a message.
    /// Check out [`Self::try_recv`] for more control over this.
    pub fn recv<'a, T: Message<'a>>(&'a mut self) -> Result<T, Error> {
        self.try_recv(Timeout::Block)
            .map(|h| h.expect("Blocked operation returned None"))
    }
    /// Waits for a message of `T` to arrive with a Timeout.
    ///
    /// - [`Timeout::Block`]:
    ///     Blocks the current thread and always returns `Some`. This is the same as [`Self::recv`].
    /// - [`Timeout::DontBlock`]:
    ///     Returns a message if one is available, otherwise, returns `None`.
    /// - [`Timeout::WaitFor`]:
    ///     Same as [`Timeout::DontBlock`], except that it keeps retrying
    ///     for the [`Duration`] specified and returns `None` if a message didn't arrive in that time.
    ///
    pub fn try_recv<'a, T: Message<'a>>(
        &'a mut self,
        timeout: Timeout,
    ) -> Result<Option<T>, Error> {
        let wire = try_harder!(timeout.retry_for(
            |this| WirePacket::read(&mut this.buf, &mut this.reader),
            self
        ));
        let msg = serde_json::from_slice::<WrappedData<T>>(wire.data)?;
        Ok(Some(msg.data))
    }
}
