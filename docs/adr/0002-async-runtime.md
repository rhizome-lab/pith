# ADR-0002: Async runtime dependency

## Status

Proposed

## Context

Several pith interfaces have async methods:
- `MonotonicClock::subscribe_duration` / `subscribe_instant`
- `InputStream::subscribe`, `OutputStream::subscribe`
- `TcpConnect::connect`, `TcpListener::accept`, etc.

These return `impl Future<Output = T>`. For native backends, implementing these futures requires an async runtime (tokio, async-std, smol, etc.).

## Options

1. **Runtime-agnostic**: Use only `std::future` primitives, let caller provide runtime
2. **Tokio dependency**: Native backends depend on tokio
3. **Feature flags**: `tokio`, `async-std`, `smol` features select runtime
4. **Separate crates**: `pith-clocks-native-tokio`, `pith-clocks-native-async-std`

## Decision

**Feature flags with tokio as default.**

```toml
[features]
default = ["tokio"]
tokio = ["dep:tokio"]
async-std = ["dep:async-std"]
```

Rationale:
- Tokio is the most common runtime
- Feature flags allow flexibility without crate explosion
- Can add more runtimes as needed

## Consequences

- Native backends have optional async runtime dependencies
- Users must enable a runtime feature for async methods
- Sync methods (like `WallClock::now`) work without any runtime
