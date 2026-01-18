//! I/O interfaces.
//!
//! Based on WASI I/O.

use std::future::Future;

/// Error type for stream operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamError {
    /// End of stream reached.
    Closed,
    /// Last operation failed.
    LastOperationFailed,
    /// Other error.
    Other(String),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "stream closed"),
            Self::LastOperationFailed => write!(f, "last operation failed"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for StreamError {}

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

/// Seek position for streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    /// Seek from the start of the stream.
    Start(u64),
    /// Seek from the end of the stream.
    End(i64),
    /// Seek from the current position.
    Current(i64),
}

impl From<std::io::SeekFrom> for SeekFrom {
    fn from(s: std::io::SeekFrom) -> Self {
        match s {
            std::io::SeekFrom::Start(n) => SeekFrom::Start(n),
            std::io::SeekFrom::End(n) => SeekFrom::End(n),
            std::io::SeekFrom::Current(n) => SeekFrom::Current(n),
        }
    }
}

impl From<SeekFrom> for std::io::SeekFrom {
    fn from(s: SeekFrom) -> Self {
        match s {
            SeekFrom::Start(n) => std::io::SeekFrom::Start(n),
            SeekFrom::End(n) => std::io::SeekFrom::End(n),
            SeekFrom::Current(n) => std::io::SeekFrom::Current(n),
        }
    }
}

/// A seekable stream.
pub trait Seek {
    /// Seek to a position in the stream.
    ///
    /// Returns the new position from the start of the stream.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, StreamError>;

    /// Get the current position in the stream.
    fn stream_position(&mut self) -> Result<u64, StreamError> {
        self.seek(SeekFrom::Current(0))
    }

    /// Rewind to the beginning of the stream.
    fn rewind(&mut self) -> Result<(), StreamError> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Get the length of the stream.
    fn stream_len(&mut self) -> Result<u64, StreamError> {
        let current = self.stream_position()?;
        let end = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(current))?;
        Ok(end)
    }
}
