//! Native implementation of pith-io.

use pith_io::{InputStream, OutputStream, Pollable, StreamError};
use std::io::{Read, Write};

/// An input stream wrapping any `std::io::Read`.
pub struct ReaderStream<R> {
    inner: R,
}

impl<R> ReaderStream<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> InputStream for ReaderStream<R> {
    fn read(&mut self, len: usize) -> Result<Vec<u8>, StreamError> {
        let mut buf = vec![0u8; len];
        match self.inner.read(&mut buf) {
            Ok(0) => Err(StreamError::Closed),
            Ok(n) => {
                buf.truncate(n);
                Ok(buf)
            }
            Err(_) => Err(StreamError::LastOperationFailed),
        }
    }

    fn blocking_read(&mut self, len: usize) -> Result<Vec<u8>, StreamError> {
        // For std::io::Read, read() is already blocking
        self.read(len)
    }

    fn subscribe(&self) -> impl std::future::Future<Output = ()> {
        // For blocking readers, always ready
        std::future::ready(())
    }
}

/// An output stream wrapping any `std::io::Write`.
pub struct WriterStream<W> {
    inner: W,
}

impl<W> WriterStream<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> OutputStream for WriterStream<W> {
    fn check_write(&self) -> Result<usize, StreamError> {
        // Blocking writers are always ready, return a reasonable buffer size
        Ok(8192)
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), StreamError> {
        self.inner
            .write_all(bytes)
            .map_err(|_| StreamError::LastOperationFailed)
    }

    fn blocking_write(&mut self, bytes: &[u8]) -> Result<(), StreamError> {
        self.write(bytes)
    }

    fn flush(&mut self) -> Result<(), StreamError> {
        self.inner
            .flush()
            .map_err(|_| StreamError::LastOperationFailed)
    }

    fn blocking_flush(&mut self) -> Result<(), StreamError> {
        self.flush()
    }

    fn subscribe(&self) -> impl std::future::Future<Output = ()> {
        // For blocking writers, always ready
        std::future::ready(())
    }
}

/// A simple pollable that's always ready.
#[derive(Debug, Default, Clone, Copy)]
pub struct AlwaysReady;

impl Pollable for AlwaysReady {
    fn ready(&self) -> bool {
        true
    }

    fn block(&self) {
        // Already ready, nothing to do
    }
}

/// A pollable that's never ready (for testing).
#[derive(Debug, Default, Clone, Copy)]
pub struct NeverReady;

impl Pollable for NeverReady {
    fn ready(&self) -> bool {
        false
    }

    fn block(&self) {
        // Block forever - only useful for testing
        loop {
            std::thread::park();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reader_stream_reads_data() {
        let data = b"hello world";
        let mut stream = ReaderStream::new(Cursor::new(data.to_vec()));
        let result = stream.read(5).unwrap();
        assert_eq!(&result, b"hello");
    }

    #[test]
    fn reader_stream_returns_closed_at_eof() {
        let data = b"hi";
        let mut stream = ReaderStream::new(Cursor::new(data.to_vec()));
        let _ = stream.read(2).unwrap();
        let result = stream.read(1);
        assert_eq!(result, Err(StreamError::Closed));
    }

    #[test]
    fn writer_stream_writes_data() {
        let mut buf = Vec::new();
        {
            let mut stream = WriterStream::new(&mut buf);
            stream.write(b"hello").unwrap();
            stream.flush().unwrap();
        }
        assert_eq!(&buf, b"hello");
    }

    #[test]
    fn always_ready_is_ready() {
        let p = AlwaysReady;
        assert!(p.ready());
    }
}
