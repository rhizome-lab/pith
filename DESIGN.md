# Design Guidelines

Principles and conventions for pith interfaces.

## Core Philosophy

**Portability over power.** Prefer simpler interfaces that work across all platforms over powerful interfaces that only work on some. When in doubt, leave it out - it's easier to add than remove.

**Capability-based first.** No ambient authority. Interfaces receive capabilities (pre-opened handles, connections, etc.) rather than acquiring them by path or name.

## Capability-Based Design

Pith follows the capability-based security model, inspired by WASI. This is a core principle, not optional.

### The Rule

**Interfaces never acquire resources by path/name. They receive pre-opened handles from the host.**

### What This Means

```rust
// WRONG - interface acquires resource by path
pub trait Database {
    fn open(path: &str) -> impl Future<Output = Result<Self::Conn, Error>>;
}

// RIGHT - interface operates on provided connection
pub trait Connection {
    fn query(&self, sql: &str, params: &[Value]) -> impl Future<Output = Result<Vec<Row>, Error>>;
    fn execute(&self, sql: &str, params: &[Value]) -> impl Future<Output = Result<u64, Error>>;
}

// Backend provides the constructor, not the interface
impl SqliteConnection {
    pub fn new(path: &str) -> Result<Self, Error> { ... }
}
```

### Why This Matters

1. **Security**: The host controls what resources the application can access
2. **Testability**: Easy to inject mock capabilities
3. **Portability**: Same interface works whether the resource is local, remote, or sandboxed
4. **WASI alignment**: Matches how WASI handles filesystem, sockets, etc.

### Checklist for New Interfaces

- [ ] No `open(path)`, `connect(url)`, or similar in the trait
- [ ] Constructors live in backends, not interfaces
- [ ] The trait operates on an already-acquired capability
- [ ] Tests can inject mock implementations without filesystem/network access

## Interface Categories

Pith's value varies by domain. The goal is **reducing decision fatigue** while providing **consistent APIs**.

### Primitives (high value)

Fundamental capabilities where abstraction enables portability and testability:

- `rhizome-pith-clocks` - time
- `rhizome-pith-random` - randomness
- `rhizome-pith-filesystem` - file I/O
- `rhizome-pith-io` - streams
- `rhizome-pith-sockets` - raw networking

These have genuinely different implementations across platforms (native, WASM, embedded). The abstraction is the value.

### Contested domains (medium value)

Areas where the Rust ecosystem has multiple viable options and no clear winner:

| Domain | Ecosystem fragmentation | Pith's role |
|--------|------------------------|-------------|
| HTTP client | reqwest, ureq, hyper, surf | Pick one, consistent API |
| Datetime | chrono, time | Pick one, consistent API |
| Markdown | pulldown-cmark, comrak | Pick one, consistent API |
| Logging | log, tracing | Pick one, consistent API |
| SQL | rusqlite, sqlx, diesel | Abstract over them |
| Caching | many options | Provide standard interface |

Here pith's value is **the decision itself** plus API consistency with other pith crates. The interface may be thin over the chosen library.

### Solved domains (low value - defer)

Areas where ecosystem consensus already exists:

| Domain | Ecosystem standard | Pith's role |
|--------|-------------------|-------------|
| Serialization | serde | Don't wrap, just use serde |
| CLI parsing | clap | Don't wrap, just use clap |
| URL parsing | url | Don't wrap, just use url |
| Regex | regex | Don't wrap, just use regex |

Creating pith wrappers here adds friction without benefit. Users already know these APIs.

### Guidelines

1. **Ask "is there ecosystem consensus?"** - if yes, defer to it
2. **Ask "does abstraction enable something?"** - portability, testability, swapping impls
3. **Consistency is valuable but not free** - don't wrap just for uniformity
4. **It's okay to just recommend** - document "use X" without wrapping

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
