//! I/O interfaces.
//!
//! Based on WASI I/O.

use std::future::Future;

/// Error type for stream operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StreamError {
    /// End of stream reached.
    #[error("stream closed")]
    Closed,
    /// Last operation failed.
    #[error("last operation failed")]
    LastOperationFailed,
    /// Other error.
    #[error("{0}")]
    Other(String),
}

/// An input stream.
pub trait InputStream {
    /// Read bytes from the stream.
    fn read(&mut self, len: usize) -> Result<Vec<u8>, StreamError>;

    /// Block until data is available.
    fn blocking_read(&mut self, len: usize) -> Result<Vec<u8>, StreamError>;

    /// Subscribe to readiness.
    fn subscribe(&self) -> impl Future<Output = ()>;
}

/// An output stream.
pub trait OutputStream {
    /// Check how many bytes can be written.
    fn check_write(&self) -> Result<usize, StreamError>;

    /// Write bytes to the stream.
    fn write(&mut self, bytes: &[u8]) -> Result<(), StreamError>;

    /// Block until bytes can be written, then write.
    fn blocking_write(&mut self, bytes: &[u8]) -> Result<(), StreamError>;

    /// Flush the stream.
    fn flush(&mut self) -> Result<(), StreamError>;

    /// Block until flushed.
    fn blocking_flush(&mut self) -> Result<(), StreamError>;

    /// Subscribe to writability.
    fn subscribe(&self) -> impl Future<Output = ()>;
}

/// A pollable resource.
pub trait Pollable {
    /// Check if ready without blocking.
    fn ready(&self) -> bool;

    /// Block until ready.
    fn block(&self);
}
