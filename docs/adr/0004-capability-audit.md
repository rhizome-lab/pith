# ADR-0004: Capability Audit

## Status

Accepted

## Context

Portals follows capability-based design: interfaces never acquire resources by path/name - they receive pre-opened handles from the host. This audit examines all 23 interfaces for compliance.

## Audit Results

### Compliant (18 interfaces)

| Interface | Notes |
|-----------|-------|
| portals-filesystem | Pre-opened `Directory` capability, all paths relative |
| portals-http | `HttpClient`/`HttpHandler` use `&self` |
| portals-keyvalue | No `open_store()`, operates on received capability |
| portals-dns | `Resolver` uses `&self` |
| portals-config | `Config` uses `&self` |
| portals-cache | `Cache` uses `&self` |
| portals-observe | All traits use `&self` |
| portals-clocks | Uses `&self` |
| portals-io | Uses `&self`/`&mut self` |
| portals-random | Uses `&self`/`&mut self` |
| portals-logging | Uses `&self` |
| portals-sql | Explicitly documented as capability-based |
| portals-nanoid | Uses `&self` |
| portals-snowflake | Uses `&self` |
| portals-cron | Uses `&self` |
| portals-markdown | Uses `&self` |
| portals-encoding | Pure functions (no resource acquisition) |
| portals-crypto | Pure algorithms (no resource acquisition) |

### Violations (4 interfaces)

#### 1. portals-sockets

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

#### 2. portals-blobstore

**Violation**: Opens containers by name within the interface.

```rust
// Current - WRONG
pub trait BlobStore {
    fn container(&self, name: &str) -> impl Future<Output = Result<Self::Container, Error>>;
    fn create_container(&self, name: &str) -> impl Future<Output = Result<(), Error>>;
}
```

#### 3. portals-websocket

**Violation**: Connects by URL string.

```rust
// Current - WRONG
pub trait WebSocketConnector {
    fn connect(&self, url: &str) -> impl Future<Output = Result<Self::Client, Error>>;
}
```

#### 4. portals-messaging

**Violation**: Gets topics by name.

```rust
// Current - WRONG
pub trait Messaging {
    fn topic(&self, name: &str) -> impl Future<Output = Result<Self::Topic, Error>>;
}
```

### Borderline Cases

#### portals-timezone

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

**portals-sockets**:
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

**portals-blobstore**:
```rust
// Interface - only Container operations
pub trait Container {
    fn get(&self, name: &str) -> impl Future<Output = Result<Vec<u8>, Error>>;
    // ...
}

// Remove BlobStore trait entirely, or make it backend-only
```

**portals-websocket**:
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

**portals-messaging**:
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
| portals-sockets | `TcpListen::bind()` (renamed trait to `TcpListener`), `UdpSocket::bind()` | `TcpListener`, `TcpStream`, `UdpSocket` (as instance trait) |
| portals-blobstore | `BlobStore` trait | `Container` trait |
| portals-websocket | `WebSocketConnector` trait | `WebSocketClient` trait |
| portals-messaging | `Messaging` trait | `Channel`, `Topic`, `Sender`, `Receiver` traits |

Backend crates provide constructors. Interfaces define operations on opened resources.

## References

- DESIGN.md "Capability-Based Design" section
- WASI sockets proposal
- portals-sql refactoring (removed `Database::open`, kept `Connection`)
