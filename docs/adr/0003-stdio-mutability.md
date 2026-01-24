# ADR-0003: Stdio trait mutability

## Status

Accepted

## Context

The initial `portals-cli` interface defined stdio traits with `&self`:

```rust
pub trait Stdin {
    fn read(&self, buf: &mut [u8]) -> std::io::Result<usize>;
}
```

However, Rust's `std::io::Read` and `std::io::Write` traits use `&mut self`:

```rust
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}
```

This is because:
- Stdin may buffer data internally
- Stdout/Stderr may buffer before flushing
- File position tracking requires state mutation

## Decision

**Change Stdin, Stdout, Stderr to use `&mut self`.**

Args and Environment keep `&self` since they're truly stateless queries.

## Consequences

```rust
pub trait Stdin {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

pub trait Stdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}

pub trait Stderr {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}
```

This aligns with Rust's standard io traits and allows efficient implementation.
