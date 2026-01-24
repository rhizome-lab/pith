//! Native implementation of portals-filesystem.

use portals_filesystem::{DirEntry, Directory, Error, FileType, Metadata};
use portals_io_native::{ReaderStream, WriterStream};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

/// A capability to access a native directory.
#[derive(Debug, Clone)]
pub struct NativeDir {
    root: PathBuf,
}

impl NativeDir {
    /// Create a new directory capability rooted at the given path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the root path of this directory capability.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a relative path against the root.
    fn resolve(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }
}

impl Directory for NativeDir {
    fn open_read(&self, path: &Path) -> Result<impl portals_filesystem::InputStream + portals_filesystem::Seek, Error> {
        let full_path = self.resolve(path);
        let file = File::open(&full_path)?;
        Ok(ReaderStream::new(file))
    }

    fn open_write(&self, path: &Path) -> Result<impl portals_filesystem::OutputStream + portals_filesystem::Seek, Error> {
        let full_path = self.resolve(path);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&full_path)?;
        Ok(WriterStream::new(file))
    }

    fn open_append(&self, path: &Path) -> Result<impl portals_filesystem::OutputStream, Error> {
        let full_path = self.resolve(path);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&full_path)?;
        Ok(WriterStream::new(file))
    }

    fn metadata(&self, path: &Path) -> Result<Metadata, Error> {
        let full_path = self.resolve(path);
        let meta = fs::metadata(&full_path)?;

        let file_type = if meta.is_file() {
            FileType::Regular
        } else if meta.is_dir() {
            FileType::Directory
        } else if meta.is_symlink() {
            FileType::Symlink
        } else {
            FileType::Unknown
        };

        Ok(Metadata {
            file_type,
            size: meta.len(),
            modified: meta.modified().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            }),
            accessed: meta.accessed().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            }),
            created: meta.created().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            }),
        })
    }

    fn read_dir(&self, path: &Path) -> Result<impl Iterator<Item = Result<DirEntry, Error>>, Error> {
        let full_path = self.resolve(path);
        let entries = fs::read_dir(&full_path)?;
        Ok(entries.map(|entry| {
            let entry = entry?;
            let file_type = entry.file_type().map_or(FileType::Unknown, |ft| {
                if ft.is_file() {
                    FileType::Regular
                } else if ft.is_dir() {
                    FileType::Directory
                } else if ft.is_symlink() {
                    FileType::Symlink
                } else {
                    FileType::Unknown
                }
            });
            Ok(DirEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                file_type,
            })
        }))
    }

    fn create_dir(&self, path: &Path) -> Result<(), Error> {
        let full_path = self.resolve(path);
        fs::create_dir(&full_path)?;
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), Error> {
        let full_path = self.resolve(path);
        fs::remove_file(&full_path)?;
        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> Result<(), Error> {
        let full_path = self.resolve(path);
        fs::remove_dir(&full_path)?;
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), Error> {
        let full_from = self.resolve(from);
        let full_to = self.resolve(to);
        fs::rename(&full_from, &full_to)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_read_file() {
        use portals_filesystem::{InputStream, OutputStream};

        let temp_dir = std::env::temp_dir().join("portals-fs-test-1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let dir = NativeDir::new(&temp_dir);

        // Write a file
        {
            let mut file = dir.open_write(Path::new("test.txt")).unwrap();
            file.write(b"hello").unwrap();
            file.flush().unwrap();
        }

        // Read it back
        {
            let mut file = dir.open_read(Path::new("test.txt")).unwrap();
            let buf = file.read(5).unwrap();
            assert_eq!(&buf, b"hello");
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn metadata_works() {
        let temp_dir = std::env::temp_dir().join("portals-fs-test-2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let dir = NativeDir::new(&temp_dir);

        // Create a file
        fs::write(temp_dir.join("test.txt"), b"hello").unwrap();

        let meta = dir.metadata(Path::new("test.txt")).unwrap();
        assert_eq!(meta.file_type, FileType::Regular);
        assert_eq!(meta.size, 5);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn read_dir_works() {
        let temp_dir = std::env::temp_dir().join("portals-fs-test-3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let dir = NativeDir::new(&temp_dir);

        // Create some entries
        fs::write(temp_dir.join("a.txt"), b"").unwrap();
        fs::write(temp_dir.join("b.txt"), b"").unwrap();
        fs::create_dir(temp_dir.join("subdir")).unwrap();

        let entries: Vec<_> = dir
            .read_dir(Path::new(""))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(entries.len(), 3);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn seek_in_file() {
        use portals_filesystem::{InputStream, Seek, SeekFrom};

        let temp_dir = std::env::temp_dir().join("portals-fs-test-4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let dir = NativeDir::new(&temp_dir);

        // Write a file
        fs::write(temp_dir.join("seek.txt"), b"hello world").unwrap();

        // Open and seek
        let mut file = dir.open_read(Path::new("seek.txt")).unwrap();
        let buf = file.read(5).unwrap();
        assert_eq!(&buf, b"hello");

        // Seek back and read again
        file.rewind().unwrap();
        let buf = file.read(5).unwrap();
        assert_eq!(&buf, b"hello");

        // Seek to offset 6
        file.seek(SeekFrom::Start(6)).unwrap();
        let buf = file.read(5).unwrap();
        assert_eq!(&buf, b"world");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
