# ADR-0004: Capability Audit

## Status

Accepted

## Context

Pith follows capability-based design: interfaces never acquire resources by path/name - they receive pre-opened handles from the host. This audit examines all 23 interfaces for compliance.

## Audit Results

### Compliant (18 interfaces)

| Interface | Notes |
|-----------|-------|
| pith-filesystem | Pre-opened `Directory` capability, all paths relative |
| pith-http | `HttpClient`/`HttpHandler` use `&self` |
| pith-keyvalue | No `open_store()`, operates on received capability |
| pith-dns | `Resolver` uses `&self` |
| pith-config | `Config` uses `&self` |
| pith-cache | `Cache` uses `&self` |
| pith-observe | All traits use `&self` |
| pith-clocks | Uses `&self` |
| pith-io | Uses `&self`/`&mut self` |
| pith-random | Uses `&self`/`&mut self` |
| pith-logging | Uses `&self` |
| pith-sql | Explicitly documented as capability-based |
| pith-nanoid | Uses `&self` |
| pith-snowflake | Uses `&self` |
| pith-cron | Uses `&self` |
| pith-markdown | Uses `&self` |
| pith-encoding | Pure functions (no resource acquisition) |
| pith-crypto | Pure algorithms (no resource acquisition) |

### Violations (4 interfaces)

#### 1. pith-sockets

**Violation**: Static `bind()` methods allow ambient authority.

```rust
// Current - WRONG
pub trait TcpListen {
    fn bind(addr: SocketAddr) -> Result<Self, Error>;  // Static, ambient
    fn accept(&self) -> impl Future<Output = Result<(Self::Stream, SocketAddr), Error>>;
}

pub trait UdpSocket {
    fn bind(addr: SocketAddr) -> Result<Self, Error>;  // Static, ambient
}
```

**Note**: `TcpConnect::connect(&self, addr)` is already correct - requires capability.

#### 2. pith-blobstore

**Violation**: Opens containers by name within the interface.

```rust
// Current - WRONG
pub trait BlobStore {
    fn container(&self, name: &str) -> impl Future<Output = Result<Self::Container, Error>>;
    fn create_container(&self, name: &str) -> impl Future<Output = Result<(), Error>>;
}
```

#### 3. pith-websocket

**Violation**: Connects by URL string.

```rust
// Current - WRONG
pub trait WebSocketConnector {
    fn connect(&self, url: &str) -> impl Future<Output = Result<Self::Client, Error>>;
}
```

#### 4. pith-messaging

**Violation**: Gets topics by name.

```rust
// Current - WRONG
pub trait Messaging {
    fn topic(&self, name: &str) -> impl Future<Output = Result<Self::Topic, Error>>;
}
```

### Borderline Cases

#### pith-timezone

```rust
pub fn get(name: &str) -> Result<TimeZone, Error>
```

This looks up timezones by IANA name. However:
- Timezones are read-only data, not resources
- No security implications to accessing timezone info
- Similar to looking up a constant by name

**Verdict**: Acceptable - not a capability violation.

## Discussion: Proposed Fixes

### Option A: Remove Constructors from Interfaces (Strict)

Move all resource acquisition to backends.

**pith-sockets**:
```rust
// Interface - only operations on existing sockets
pub trait TcpListener {
    fn accept(&self) -> impl Future<Output = Result<(Self::Stream, SocketAddr), Error>>;
    fn local_addr(&self) -> Result<SocketAddr, Error>;
}

// Backend provides construction
impl NativeTcpListener {
    pub fn bind(addr: SocketAddr) -> Result<Self, Error> { ... }
}
```

**pith-blobstore**:
```rust
// Interface - only Container operations
pub trait Container {
    fn get(&self, name: &str) -> impl Future<Output = Result<Vec<u8>, Error>>;
    // ...
}

// Remove BlobStore trait entirely, or make it backend-only
```

**pith-websocket**:
```rust
// Interface - only WebSocketClient operations
pub trait WebSocketClient {
    fn send(&mut self, msg: Message) -> impl Future<Output = Result<(), Error>>;
    fn recv(&mut self) -> impl Future<Output = Result<Message, Error>>;
}

// Remove WebSocketConnector, backend provides connection
impl NativeWebSocketClient {
    pub fn connect(url: &str) -> Result<Self, Error> { ... }
}
```

**pith-messaging**:
```rust
// Interface - only Sender/Receiver/Topic operations
pub trait Topic {
    fn publish(&self, message: Message) -> impl Future<Output = Result<(), Error>>;
    fn subscribe(&self) -> impl Future<Output = Result<Self::Subscriber, Error>>;
}

// Remove Messaging trait or keep for backend internal use only
```

### Option B: Capability Providers (Less Strict)

Keep the "manager" traits but rename them to clarify they're host-provided capabilities.

```rust
// Explicitly named as a capability provider
pub trait NetworkCapability {
    type Listener: TcpListener;
    fn bind_tcp(&self, addr: SocketAddr) -> Result<Self::Listener, Error>;
}

pub trait BlobStoreCapability {
    type Container: Container;
    fn open_container(&self, name: &str) -> Result<Self::Container, Error>;
}
```

This acknowledges that the *capability provider itself* is a capability received from the host.

### Option C: Document Intent (Minimal Change)

Add documentation clarifying that these traits represent host-provided capabilities, not ambient authority. The host controls what addresses can be bound, what containers can be accessed, etc.

```rust
/// A network capability that permits binding listeners.
///
/// This capability is provided by the host and may be restricted
/// (e.g., only certain ports, only localhost, etc.).
pub trait TcpListen { ... }
```

## Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **A: Strict** | Clear separation, matches WASI | Removes useful abstractions, can't swap backends for "connector" logic |
| **B: Capability Providers** | Keeps abstractions, explicit naming | More traits to maintain |
| **C: Document** | No code changes | Doesn't enforce at type level |

## Questions to Resolve

1. **Is a "connector/binder" trait valuable?** If all WebSocket backends have identical `connect(url)` signatures, is there value in abstracting it?

2. **WASI alignment**: WASI provides socket bindings through host imports. Should our interfaces assume the same model?

3. **Practical sandboxing**: In practice, will hosts actually restrict these operations? If a host gives you `TcpListen`, can it restrict which ports?

4. **Consistency vs pragmatism**: Should we strictly follow the "no open(name)" rule even when the name (URL, port) is fundamental to the operation?

## Decision

**Option A: Strict capability model.**

Remove "manager" traits from interfaces. Keep only resource traits that operate on already-opened handles.

### Rationale

The capability model isn't just about security sandboxing - it's about clean architecture:

1. **Least privilege**: Each module only sees the resources it was given
2. **Explicit dependencies**: Wiring code documents who accesses what
3. **Testability**: Inject a mock `Container`, no need to mock a whole `BlobStore`
4. **Separation of concerns**: Modules can't accidentally access each other's resources

A `BlobStore` capability that opens any container by name is a "god object" - too broad. Pre-opened `Container` handles enforce proper scoping at the type level.

### Changes Required

| Interface | Remove | Keep |
|-----------|--------|------|
| pith-sockets | `TcpListen::bind()` (renamed trait to `TcpListener`), `UdpSocket::bind()` | `TcpListener`, `TcpStream`, `UdpSocket` (as instance trait) |
| pith-blobstore | `BlobStore` trait | `Container` trait |
| pith-websocket | `WebSocketConnector` trait | `WebSocketClient` trait |
| pith-messaging | `Messaging` trait | `Channel`, `Topic`, `Sender`, `Receiver` traits |

Backend crates provide constructors. Interfaces define operations on opened resources.

## References

- DESIGN.md "Capability-Based Design" section
- WASI sockets proposal
- pith-sql refactoring (removed `Database::open`, kept `Connection`)
