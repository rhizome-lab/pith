//! Native implementation of pith-cli.

use pith_cli::{Args, Environment, Stderr, Stdin, Stdout};
use std::io::{Read, Write};

/// Command-line arguments from the OS.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsArgs;

impl Args for OsArgs {
    fn args(&self) -> impl Iterator<Item = String> {
        std::env::args()
    }
}

/// Environment variables from the OS.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsEnvironment;

impl Environment for OsEnvironment {
    fn vars(&self) -> impl Iterator<Item = (String, String)> {
        std::env::vars()
    }

    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// Standard input from the OS.
#[derive(Debug)]
pub struct OsStdin {
    inner: std::io::Stdin,
}

impl Default for OsStdin {
    fn default() -> Self {
        Self::new()
    }
}

impl OsStdin {
    pub fn new() -> Self {
        Self {
            inner: std::io::stdin(),
        }
    }
}

impl Stdin for OsStdin {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

/// Standard output to the OS.
#[derive(Debug)]
pub struct OsStdout {
    inner: std::io::Stdout,
}

impl Default for OsStdout {
    fn default() -> Self {
        Self::new()
    }
}

impl OsStdout {
    pub fn new() -> Self {
        Self {
            inner: std::io::stdout(),
        }
    }
}

impl Stdout for OsStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Standard error to the OS.
#[derive(Debug)]
pub struct OsStderr {
    inner: std::io::Stderr,
}

impl Default for OsStderr {
    fn default() -> Self {
        Self::new()
    }
}

impl OsStderr {
    pub fn new() -> Self {
        Self {
            inner: std::io::stderr(),
        }
    }
}

impl Stderr for OsStderr {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_returns_at_least_one() {
        let args = OsArgs;
        // Should at least have the test binary name
        assert!(args.args().count() >= 1);
    }

    #[test]
    fn env_has_path() {
        let env = OsEnvironment;
        // PATH should exist on most systems
        assert!(env.var("PATH").is_some() || env.var("Path").is_some());
    }

    #[test]
    fn stdout_write_works() {
        let mut stdout = OsStdout::new();
        // Write empty buffer should succeed
        assert!(stdout.write(b"").is_ok());
        assert!(stdout.flush().is_ok());
    }
}
