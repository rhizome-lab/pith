//! Filesystem interfaces.
//!
//! Based on WASI filesystem.

use std::path::Path;

pub use pith_io::{InputStream, OutputStream, StreamError};

/// Filesystem error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("access denied")]
    Access,
    #[error("already exists")]
    Exist,
    #[error("not found")]
    NotFound,
    #[error("not a directory")]
    NotDirectory,
    #[error("is a directory")]
    IsDirectory,
    #[error("invalid argument")]
    Invalid,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// File type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    Unknown,
}

/// File metadata.
#[derive(Debug, Clone)]
pub struct Metadata {
    pub file_type: FileType,
    pub size: u64,
    pub modified: Option<u64>,
    pub accessed: Option<u64>,
    pub created: Option<u64>,
}

/// A capability to access a directory and its contents.
pub trait Directory {
    /// Open a file for reading.
    fn open_read(&self, path: &Path) -> Result<impl InputStream, Error>;

    /// Open a file for writing (creates if not exists, truncates if exists).
    fn open_write(&self, path: &Path) -> Result<impl OutputStream, Error>;

    /// Open a file for appending.
    fn open_append(&self, path: &Path) -> Result<impl OutputStream, Error>;

    /// Get metadata for a path.
    fn metadata(&self, path: &Path) -> Result<Metadata, Error>;

    /// List directory contents.
    fn read_dir(&self, path: &Path) -> Result<impl Iterator<Item = Result<DirEntry, Error>>, Error>;

    /// Create a directory.
    fn create_dir(&self, path: &Path) -> Result<(), Error>;

    /// Remove a file.
    fn remove_file(&self, path: &Path) -> Result<(), Error>;

    /// Remove a directory.
    fn remove_dir(&self, path: &Path) -> Result<(), Error>;

    /// Rename a file or directory.
    fn rename(&self, from: &Path, to: &Path) -> Result<(), Error>;
}

/// A directory entry.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub file_type: FileType,
}
