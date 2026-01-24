//! Native implementation of pith-io.

use rhizome_rhi_portals_io::{InputStream, OutputStream, Pollable, Seek, SeekFrom, StreamError};
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
    fn read_into(&mut self, buf: &mut [u8]) -> Result<usize, StreamError> {
        match self.inner.read(buf) {
            Ok(0) => Err(StreamError::Closed),
            Ok(n) => Ok(n),
            Err(_) => Err(StreamError::LastOperationFailed),
        }
    }

    fn blocking_read_into(&mut self, buf: &mut [u8]) -> Result<usize, StreamError> {
        // For std::io::Read, read() is already blocking
        self.read_into(buf)
    }

    fn subscribe(&self) -> impl std::future::Future<Output = ()> {
        // For blocking readers, always ready
        std::future::ready(())
    }
}

impl<R: std::io::Seek> Seek for ReaderStream<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, StreamError> {
        self.inner
            .seek(pos.into())
            .map_err(|_| StreamError::LastOperationFailed)
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

impl<W: std::io::Seek> Seek for WriterStream<W> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, StreamError> {
        self.inner
            .seek(pos.into())
            .map_err(|_| StreamError::LastOperationFailed)
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
    fn reader_stream_read_into_zero_copy() {
        let data = b"hello world";
        let mut stream = ReaderStream::new(Cursor::new(data.to_vec()));

        // Read into pre-allocated buffer
        let mut buf = [0u8; 5];
        let n = stream.read_into(&mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, b"hello");

        // Read more
        let mut buf = [0u8; 10];
        let n = stream.read_into(&mut buf).unwrap();
        assert_eq!(n, 6); // " world" = 6 bytes
        assert_eq!(&buf[..n], b" world");
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

    #[test]
    fn reader_stream_seek() {
        let data = b"hello world";
        let mut stream = ReaderStream::new(Cursor::new(data.to_vec()));

        // Read first 5 bytes
        let result = stream.read(5).unwrap();
        assert_eq!(&result, b"hello");

        // Seek back to start
        stream.rewind().unwrap();
        let result = stream.read(5).unwrap();
        assert_eq!(&result, b"hello");

        // Seek to position 6
        let pos = stream.seek(SeekFrom::Start(6)).unwrap();
        assert_eq!(pos, 6);
        let result = stream.read(5).unwrap();
        assert_eq!(&result, b"world");

        // Get stream length
        let len = stream.stream_len().unwrap();
        assert_eq!(len, 11);

        // Get current position
        let pos = stream.stream_position().unwrap();
        assert_eq!(pos, 11);
    }

    #[test]
    fn writer_stream_seek() {
        let mut buf = Cursor::new(vec![0u8; 11]);
        {
            let mut stream = WriterStream::new(&mut buf);
            stream.write(b"hello").unwrap();
            stream.seek(SeekFrom::Start(6)).unwrap();
            stream.write(b"world").unwrap();
            stream.flush().unwrap();
        }
        assert_eq!(buf.into_inner(), b"hello\0world".to_vec());
    }
}
