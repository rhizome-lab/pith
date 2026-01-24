//! Filesystem interfaces.
//!
//! Based on WASI filesystem.

use std::path::Path;

pub use portals_io::{InputStream, OutputStream, Seek, SeekFrom, StreamError};

/// Filesystem error types.
#[derive(Debug)]
pub enum Error {
    Access,
    Exist,
    NotFound,
    NotDirectory,
    IsDirectory,
    Invalid,
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Access => write!(f, "access denied"),
            Self::Exist => write!(f, "already exists"),
            Self::NotFound => write!(f, "not found"),
            Self::NotDirectory => write!(f, "not a directory"),
            Self::IsDirectory => write!(f, "is a directory"),
            Self::Invalid => write!(f, "invalid argument"),
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
    fn open_read(&self, path: &Path) -> Result<impl InputStream + Seek, Error>;

    /// Open a file for writing (creates if not exists, truncates if exists).
    fn open_write(&self, path: &Path) -> Result<impl OutputStream + Seek, Error>;

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
