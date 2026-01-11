//! CLI interfaces.
//!
//! Based on WASI CLI.

/// Access to command-line arguments.
pub trait Args {
    /// Returns an iterator over the command-line arguments.
    fn args(&self) -> impl Iterator<Item = String>;
}

/// Access to environment variables.
pub trait Environment {
    /// Returns an iterator over environment variables.
    fn vars(&self) -> impl Iterator<Item = (String, String)>;

    /// Gets the value of an environment variable.
    fn var(&self, key: &str) -> Option<String>;
}

/// Standard input stream.
///
/// Uses `&mut self` to match `std::io::Read`. See ADR-0003.
pub trait Stdin {
    /// Read bytes from stdin.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

/// Standard output stream.
///
/// Uses `&mut self` to match `std::io::Write`. See ADR-0003.
pub trait Stdout {
    /// Write bytes to stdout.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;

    /// Flush stdout.
    fn flush(&mut self) -> std::io::Result<()>;
}

/// Standard error stream.
///
/// Uses `&mut self` to match `std::io::Write`. See ADR-0003.
pub trait Stderr {
    /// Write bytes to stderr.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;

    /// Flush stderr.
    fn flush(&mut self) -> std::io::Result<()>;
}
