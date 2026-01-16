# Design Guidelines

Principles and conventions for pith interfaces.

## Core Philosophy

**Portability over power.** Prefer simpler interfaces that work across all platforms over powerful interfaces that only work on some. When in doubt, leave it out - it's easier to add than remove.

## Interface Categories

Not all interfaces are equal in terms of how "pure" or opinionated they are.

### Primitive interfaces (preferred)

Generic abstractions over fundamental capabilities:

- `pith-clocks` - time
- `pith-random` - randomness
- `pith-filesystem` - file I/O
- `pith-io` - streams
- `pith-sockets` - raw networking
- `pith-keyvalue` - key-value storage

These are **pure** - they abstract a capability without imposing a specific format or protocol. Backends have freedom in how they implement them.

### Protocol/standard interfaces (use sparingly)

Interfaces that implement specific standards or formats:

- `pith-http` - HTTP protocol
- `pith-websocket` - WebSocket protocol
- `pith-cron` - cron expression syntax
- `pith-markdown` - Markdown format
- `pith-uuid` - UUID specification
- `pith-jwt` - JWT specification

These are **opinionated** - they're already constrained by external specifications. The interface is largely dictated by the protocol, leaving less room for abstraction.

### Guidelines

1. **Prefer primitive interfaces** - they provide more value as abstractions
2. **Protocol interfaces are fine** when the protocol is truly universal (HTTP, UUID)
3. **Be cautious** with niche protocols - they may not warrant an interface
4. **Consider whether abstraction adds value** - if every backend will implement the same spec identically, maybe just use a library directly

### When to create a protocol interface

Good reasons:
- The protocol is universal and long-lived (HTTP, UUID)
- You need to swap implementations (e.g., different HTTP clients)
- Testing requires mocking the protocol layer

Weaker reasons:
- "It would be nice to have" - prefer direct library use
- The protocol has only one viable implementation

## Error Handling

### Manual Display and Error impls

Keep dependencies minimal - use manual impls rather than `thiserror`:

```rust
#[derive(Debug)]
pub enum Error {
    NotFound,
    Invalid(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::Invalid(s) => write!(f, "invalid input: {}", s),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}
```

### Error granularity

- Per-interface error enums (not one giant error type)
- Include a catch-all variant for backend-specific errors: `Other(String)`
- Add manual `From` impls for common conversions (e.g., `std::io::Error`)

### When to use Result vs Option vs bare values

| Situation | Use |
|-----------|-----|
| Operation can fail in ways caller might handle | `Result<T, Error>` |
| Value might not exist (but that's not an error) | `Option<T>` |
| Operation is infallible | `T` |
| Fire-and-forget side effect | `()` |

### Panics

Only panic when:
1. The error is unrecoverable AND
2. The caller cannot reasonably handle it AND
3. Continuing would cause worse problems (data corruption, security issues)

Examples: programmer errors (invalid invariants), not runtime errors (file not found).

## Async

### When to use async

- **Async**: Network I/O, timers, anything that might block
- **Sync**: CPU-bound operations, memory access, infallible lookups

### Syntax

Use `impl Future<Output = T>` in trait definitions (not `async fn` - better compatibility):

```rust
pub trait Store {
    fn get(&self, key: &str) -> impl Future<Output = Result<Vec<u8>, Error>>;
}
```

### Sync variants

For interfaces that are async-first, consider providing sync variants via:
- Separate `*Sync` trait
- Feature-gated sync methods
- Blocking wrapper in backend crate

## Mutability

### Default to `&self`

Use `&self` unless an implementation would *definitely* need mutable state:

```rust
// Good - most impls can use interior mutability (RwLock, etc.)
fn get(&self, key: &str) -> Result<Value, Error>;

// Good - PRNGs must mutate state, documented in ADR
fn fill(&mut self, buf: &mut [u8]);
```

### Document `&mut self` decisions

When using `&mut self`, add an ADR explaining why.

## Traits

### Prefer small, focused traits

Split by capability, not by object:

```rust
// Good - separable capabilities
pub trait KeyValue { ... }
pub trait AtomicKeyValue: KeyValue { ... }

// Avoid - monolithic
pub trait Store {
    fn get(&self, ...);
    fn set(&self, ...);
    fn atomic_cas(&self, ...);  // Not all stores support this!
}
```

### Extension traits

Use supertraits for "enhanced" versions:

```rust
pub trait AtomicKeyValue: KeyValue {
    fn compare_and_swap(&self, ...) -> ...;
}
```

### Convenience methods

Provide default implementations for common patterns:

```rust
pub trait Logger {
    fn log(&self, record: &Record);  // Required

    fn info(&self, target: &str, msg: &str) {  // Convenience
        self.log(&Record::new(Level::Info, target, msg));
    }
}
```

## Associated Types vs Generics

### Associated types for "this trait produces that type"

```rust
pub trait BlobStore {
    type Container: Container;
    fn container(&self, name: &str) -> Result<Self::Container, Error>;
}
```

### Generics for "caller chooses the type"

```rust
pub trait Serializer {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Error>;
}
```

## Data Types

### Prefer plain structs with pub fields

For simple data containers:

```rust
// Good - simple, flexible
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

// Usage
let req = Request {
    method: Method::Get,
    url: "https://example.com".into(),
    ..Default::default()
};
```

### Use builders when construction is complex

Only when:
- Many optional fields with non-obvious defaults
- Validation needed during construction
- Method chaining significantly improves ergonomics

```rust
// Builder makes sense here - many optional fields
let req = Request::builder()
    .method(Method::Post)
    .url("https://api.example.com")
    .header("Content-Type", "application/json")
    .body(json_bytes)
    .build()?;
```

### Implement Default where sensible

```rust
#[derive(Default)]
pub struct Request {
    pub method: Method,  // Method should also impl Default
    // ...
}
```

## Naming Conventions

### CRUD operations

| Operation | Verb | Example |
|-----------|------|---------|
| Read one | `get` | `get(key)` |
| Read many | `list` | `list()`, `list_keys()` |
| Create/Update | `set` | `set(key, value)` |
| Create only | `create` | `create(name)` |
| Delete | `delete` | `delete(key)` |
| Check existence | `exists` | `exists(key)` |

### Filesystem operations

Follow Rust std conventions:
- `create_dir`, `remove_file`, `remove_dir` (not `delete_*`)
- `read_dir` (not `list_dir`)

### Transformations

| Operation | Verb |
|-----------|------|
| Parse from bytes/string | `parse`, `from_*` |
| Convert to bytes/string | `to_*`, `encode` |
| Transform in place | `*_mut` suffix |

### Async method naming

Don't suffix with `_async` - async is the default where it makes sense. Use `blocking_*` prefix for sync variants in async-first interfaces:

```rust
fn read(&mut self, len: usize) -> Result<Vec<u8>, Error>;
fn blocking_read(&mut self, len: usize) -> Result<Vec<u8>, Error>;
```

## Platform Support

### no_std compatibility

For interfaces that should work on embedded:

```rust
#![no_std]
extern crate alloc;

use core::future::Future;
use alloc::vec::Vec;
use alloc::string::String;
```

Feature-gate std-only functionality:

```rust
#[cfg(feature = "std")]
impl From<std::io::Error> for Error { ... }
```

### Platform-specific backends

- Keep interfaces platform-agnostic
- Put platform-specific code in backends only
- Use feature flags for optional platform support

## Documentation

### Every public item needs a doc comment

```rust
/// A key-value store.
///
/// Provides basic CRUD operations for storing and retrieving
/// byte values by string keys.
pub trait KeyValue {
    /// Get a value by key.
    ///
    /// Returns `Error::NotFound` if the key doesn't exist.
    fn get(&self, key: &str) -> impl Future<Output = Result<Vec<u8>, Error>>;
}
```

### Link to ADRs for non-obvious decisions

```rust
/// Source of non-cryptographic random bytes.
///
/// Uses `&mut self` because PRNGs must mutate internal state.
/// See ADR-0001 for rationale.
pub trait InsecureRandom { ... }
```
