//! This module provides very lightweight, small utilities to quickly create servers.
use std::{
    io,
    net::{TcpListener, ToSocketAddrs},
};

use crate::Peer;

/// Possible modes of operation for sockets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Blocking,
    NonBlocking,
}

/// An easy-to-use, simple server that waits for TCP peers.
///
/// This is just a light abstraction over [`std::net::TcpListener`].
/// ```no_run
/// use wing_rpc::server::{TcpServer, Mode};
///
/// let server = TcpServer::bind("localhost:6000").expect("Failed to bind");
/// let peer = server.accept(Mode::Blocking).expect("Failed to accept peer");
/// ```
#[derive(Debug)]
pub struct TcpServer(TcpListener);

impl TcpServer {
    /// Creates a [`TcpServer`] that waits for connections on `addr`
    ///
    /// Use [`TcpServer::accept`] to turn incoming connections into a peer.
    pub fn bind(addr: impl ToSocketAddrs) -> io::Result<Self> {
        Ok(Self(TcpListener::bind(addr)?))
    }

    /// Return the inner [`std::net::TcpListener`].
    ///
    /// If you need to have more fine control over the connection lifecycle, consider implementing your own server.
    pub fn listener(&self) -> &TcpListener {
        &self.0
    }

    /// Wait for [`crate::Peer`] to connect.
    ///
    /// This function will block the current thread while waiting for a connection.
    pub fn accept(&self, mode: Mode) -> io::Result<Peer> {
        let (sock, _) = self.0.accept()?;
        if let Mode::NonBlocking = mode {
            sock.set_nonblocking(true)?;
        }
        Ok(Peer::from_socket(sock))
    }
}
