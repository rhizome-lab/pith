//! Socket interfaces.
//!
//! Based on WASI sockets.

use std::future::Future;
use std::net::{IpAddr, SocketAddr};

/// Socket errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("address in use")]
    AddressInUse,
    #[error("address not available")]
    AddressNotAvailable,
    #[error("connection refused")]
    ConnectionRefused,
    #[error("connection reset")]
    ConnectionReset,
    #[error("connection aborted")]
    ConnectionAborted,
    #[error("not connected")]
    NotConnected,
    #[error("timeout")]
    Timeout,
    #[error("access denied")]
    Access,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

/// A TCP socket that can connect to a remote address.
pub trait TcpConnect {
    type Stream: TcpStream;

    /// Connect to a remote address.
    fn connect(&self, addr: SocketAddr) -> impl Future<Output = Result<Self::Stream, Error>>;
}

/// A TCP listener that accepts connections.
pub trait TcpListen {
    type Stream: TcpStream;

    /// Bind to a local address.
    fn bind(addr: SocketAddr) -> Result<Self, Error>
    where
        Self: Sized;

    /// Accept a connection.
    fn accept(&self) -> impl Future<Output = Result<(Self::Stream, SocketAddr), Error>>;

    /// Get the local address.
    fn local_addr(&self) -> Result<SocketAddr, Error>;
}

/// A connected TCP stream.
pub trait TcpStream {
    /// Read data from the stream.
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, Error>>;

    /// Write data to the stream.
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = Result<usize, Error>>;

    /// Flush the stream.
    fn flush(&mut self) -> impl Future<Output = Result<(), Error>>;

    /// Shutdown the stream.
    fn shutdown(&mut self) -> Result<(), Error>;

    /// Get the local address.
    fn local_addr(&self) -> Result<SocketAddr, Error>;

    /// Get the remote address.
    fn peer_addr(&self) -> Result<SocketAddr, Error>;
}

/// A UDP socket.
pub trait UdpSocket {
    /// Bind to a local address.
    fn bind(addr: SocketAddr) -> Result<Self, Error>
    where
        Self: Sized;

    /// Send data to a remote address.
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> impl Future<Output = Result<usize, Error>>;

    /// Receive data and the sender's address.
    fn recv_from(
        &mut self,
        buf: &mut [u8],
    ) -> impl Future<Output = Result<(usize, SocketAddr), Error>>;

    /// Get the local address.
    fn local_addr(&self) -> Result<SocketAddr, Error>;
}

/// DNS resolution.
pub trait Resolver {
    /// Resolve a hostname to IP addresses.
    fn resolve(&self, host: &str) -> impl Future<Output = Result<Vec<IpAddr>, Error>>;
}
