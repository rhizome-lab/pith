//! Native implementation of portals-sockets using tokio.

use portals_sockets::{Error, Resolver, TcpConnect, TcpListener, TcpStream, UdpSocket};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

/// Native TCP connector using tokio.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeTcpConnect;

impl TcpConnect for NativeTcpConnect {
    type Stream = NativeTcpStream;

    async fn connect(&self, addr: SocketAddr) -> Result<Self::Stream, Error> {
        let stream = net::TcpStream::connect(addr).await?;
        Ok(NativeTcpStream(stream))
    }
}

/// Native TCP listener using tokio.
#[derive(Debug)]
pub struct NativeTcpListener(net::TcpListener);

impl NativeTcpListener {
    /// Bind to a local address.
    pub fn bind(addr: SocketAddr) -> Result<Self, Error> {
        // Use std to bind synchronously, then convert to tokio
        let std_listener = std::net::TcpListener::bind(addr)?;
        std_listener.set_nonblocking(true)?;
        let listener = net::TcpListener::from_std(std_listener)?;
        Ok(Self(listener))
    }
}

impl TcpListener for NativeTcpListener {
    type Stream = NativeTcpStream;

    async fn accept(&self) -> Result<(Self::Stream, SocketAddr), Error> {
        let (stream, addr) = self.0.accept().await?;
        Ok((NativeTcpStream(stream), addr))
    }

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        Ok(self.0.local_addr()?)
    }
}

/// Native TCP stream using tokio.
#[derive(Debug)]
pub struct NativeTcpStream(net::TcpStream);

impl TcpStream for NativeTcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(self.0.read(buf).await?)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        Ok(self.0.write(buf).await?)
    }

    async fn flush(&mut self) -> Result<(), Error> {
        Ok(self.0.flush().await?)
    }

    fn shutdown(&mut self) -> Result<(), Error> {
        // Use std's shutdown via TryInto
        use std::os::unix::io::{AsRawFd, FromRawFd};
        // This is a bit unsafe but necessary for sync shutdown
        let fd = self.0.as_raw_fd();
        // SAFETY: We're not taking ownership, just calling shutdown
        unsafe {
            let std_stream = std::net::TcpStream::from_raw_fd(fd);
            let result = std_stream.shutdown(std::net::Shutdown::Both);
            // Forget so we don't close the fd
            std::mem::forget(std_stream);
            result?;
        }
        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        Ok(self.0.local_addr()?)
    }

    fn peer_addr(&self) -> Result<SocketAddr, Error> {
        Ok(self.0.peer_addr()?)
    }
}

/// Native UDP socket using tokio.
#[derive(Debug)]
pub struct NativeUdpSocket(net::UdpSocket);

impl NativeUdpSocket {
    /// Bind to a local address.
    pub fn bind(addr: SocketAddr) -> Result<Self, Error> {
        let std_socket = std::net::UdpSocket::bind(addr)?;
        std_socket.set_nonblocking(true)?;
        let socket = net::UdpSocket::from_std(std_socket)?;
        Ok(Self(socket))
    }
}

impl UdpSocket for NativeUdpSocket {
    async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, Error> {
        Ok(self.0.send_to(buf, addr).await?)
    }

    async fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        Ok(self.0.recv_from(buf).await?)
    }

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        Ok(self.0.local_addr()?)
    }
}

/// Native DNS resolver using tokio.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeResolver;

impl Resolver for NativeResolver {
    async fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, Error> {
        let addrs = tokio::net::lookup_host(format!("{}:0", host)).await?;
        Ok(addrs.map(|a| a.ip()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolver_works() {
        let resolver = NativeResolver;
        let addrs = resolver.resolve("localhost").await.unwrap();
        assert!(!addrs.is_empty());
    }

    #[tokio::test]
    async fn tcp_echo() {
        // Start a listener
        let listener = NativeTcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn a task to accept and echo
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 5];
            let n = stream.read(&mut buf).await.unwrap();
            stream.write(&buf[..n]).await.unwrap();
        });

        // Connect and send data
        let connector = NativeTcpConnect;
        let mut stream = connector.connect(addr).await.unwrap();
        stream.write(b"hello").await.unwrap();

        let mut buf = [0u8; 5];
        let n = stream.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn udp_echo() {
        let server = NativeUdpSocket::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let server_addr = server.local_addr().unwrap();

        let client = NativeUdpSocket::bind("127.0.0.1:0".parse().unwrap()).unwrap();

        // Send from client
        client.send_to(b"hello", server_addr).await.unwrap();

        // Receive on server
        let mut server = server;
        let mut buf = [0u8; 5];
        let (n, client_addr) = server.recv_from(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");

        // Echo back
        server.send_to(&buf[..n], client_addr).await.unwrap();

        // Receive on client
        let mut client = client;
        let (n, _) = client.recv_from(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
    }
}
