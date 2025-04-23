//! This module provides very lightweight, small utilities to quickly initialize clients.

use std::io;
use std::net::{TcpStream, ToSocketAddrs};

use crate::Peer;

/// Connects to a TCP server listening at `addr`.
/// ```no_run
/// use wing_rpc::client;
///
/// let peer = client::tcp("localhost:6000").unwrap();
/// ```
pub fn tcp(addr: impl ToSocketAddrs) -> io::Result<Peer> {
    let sock = TcpStream::connect(addr)?;
    Ok(Peer::from_socket(sock))
}
