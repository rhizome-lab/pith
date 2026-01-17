//! Socket interfaces.
//!
//! Based on WASI sockets. Follows capability-based design: traits define
//! operations on already-bound/connected sockets. Backends provide constructors.
//!
//! See ADR-0004 for rationale.

use std::future::Future;
use std::net::{IpAddr, SocketAddr};

/// Socket errors.
#[derive(Debug)]
pub enum Error {
    AddressInUse,
    AddressNotAvailable,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    Timeout,
    Access,
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddressInUse => write!(f, "address in use"),
            Self::AddressNotAvailable => write!(f, "address not available"),
            Self::ConnectionRefused => write!(f, "connection refused"),
            Self::ConnectionReset => write!(f, "connection reset"),
            Self::ConnectionAborted => write!(f, "connection aborted"),
            Self::NotConnected => write!(f, "not connected"),
            Self::Timeout => write!(f, "timeout"),
            Self::Access => write!(f, "access denied"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// A capability to initiate TCP connections.
///
/// The host provides this capability, potentially restricted to certain
/// addresses or ports. Use the backend constructor to obtain an instance.
pub trait TcpConnect {
    type Stream: TcpStream;

    /// Connect to a remote address.
    fn connect(&self, addr: SocketAddr) -> impl Future<Output = Result<Self::Stream, Error>>;
}

/// A bound TCP listener that accepts connections.
///
/// This trait operates on an already-bound listener. The binding is done
/// by the backend constructor, not the interface.
///
/// ```ignore
/// // Backend provides construction
/// let listener = NativeTcpListener::bind("127.0.0.1:8080")?;
///
/// // Interface defines operations
/// let (stream, addr) = listener.accept().await?;
/// ```
pub trait TcpListener {
    type Stream: TcpStream;

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

/// A bound UDP socket.
///
/// This trait operates on an already-bound socket. The binding is done
/// by the backend constructor, not the interface.
///
/// ```ignore
/// // Backend provides construction
/// let socket = NativeUdpSocket::bind("0.0.0.0:0")?;
///
/// // Interface defines operations
/// socket.send_to(b"hello", target_addr).await?;
/// ```
pub trait UdpSocket {
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
