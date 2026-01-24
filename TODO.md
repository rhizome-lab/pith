# TODO

## Roadmap

1. [x] Diverge from WASI where ergonomics warrant
2. [x] Add more interfaces (crypto, encoding, sql)
3. [x] Reference implementations for native targets
4. [x] Unified streams (filesystem returns `portals-io` streams)
5. [x] WebSocket and DNS interfaces + native backends
   - [x] portals-websocket interface + portals-websocket-native (via tungstenite)
   - [x] portals-dns interface + portals-dns-native (via hickory-resolver)
6. [x] Protocol implementations (`crates/protocols/`)
   - [x] portals-http1 (HTTP/1.1 wire format parsing/serialization)
7. [x] Mock backends for testing (`crates/backends/mock/`)
   - [x] portals-clocks-mock (controllable wall/monotonic clocks)
   - [x] portals-random-mock (deterministic secure/insecure random)
   - [x] portals-http-mock (request recording, response queuing)
8. [x] Crypto AAD support (AES-GCM, ChaCha20-Poly1305)

## Backlog

### Capability Audit

See [ADR-0004](docs/adr/0004-capability-audit.md) for full audit results.

**Summary**: All 23 interfaces now compliant. Fixed violations:
- [x] portals-sockets - removed `TcpListen::bind()`, `UdpSocket::bind()`; renamed to `TcpListener`
- [x] portals-blobstore - removed `BlobStore` trait, kept `Container`
- [x] portals-websocket - removed `WebSocketConnector` trait, kept `WebSocketClient`
- [x] portals-messaging - removed `Messaging` trait, kept `Channel`/`Topic`/`Sender`/`Receiver`

### WASI Phase 1/2 interfaces

- [x] **portals-url** - URL parsing (WASI Phase 1)
- [x] **portals-timezone** - timezone handling (WASI Phase 2)
- [x] **portals-config** - runtime configuration (WASI Phase 1)
- [x] **portals-logging** - structured logging (WASI Phase 1)
- [x] **portals-keyvalue** - key-value store (WASI Phase 1)
- [x] **portals-blobstore** - blob storage (WASI Phase 1)
- [x] **portals-observe** - observability/telemetry (WASI Phase 1)
- [x] **portals-messaging** - message queues (WASI Phase 1)

### Unstable / Deferred

APIs likely to change or too platform-specific:

- **portals-nn** - neural network inference (WASI Phase 2, ML APIs evolving fast)
- **portals-gfx** - graphics (WASI Phase 2, graphics APIs notoriously unstable)
- **portals-threads** - threading (WASI Phase 1, complex semantics)
- **portals-i2c / portals-spi / portals-usb** - hardware interfaces (niche, platform-specific)
- **portals-distributed-lock** - distributed locking (niche)

## Potential Interfaces

Application-level interfaces to consider (beyond WASI):

### Identity / Auth
- **portals-jwt** - JWT parsing/validation/creation
- **portals-oauth** - OAuth flow abstractions
- **portals-session** - session management

### Data / Validation
- **portals-cache** - caching with TTL/LRU policies
- **portals-validation** - schema validation
- **portals-serialization** - JSON/TOML/YAML/etc (or per-format crates)

### Text / Formatting
- **portals-i18n** - internationalization/localization
- **portals-markdown** - markdown parsing/rendering
- **portals-template** - templating

### Media
- **portals-image** - image metadata/transforms
- **portals-audio** - audio metadata/processing
- **portals-video** - video metadata

### Communication
- **portals-email** - email sending/parsing
- **portals-notification** - push notifications

### Identifiers
- **portals-uuid** - UUID generation/parsing
- **portals-nanoid** - nanoid generation
- **portals-snowflake** - snowflake IDs

### Scheduling
- **portals-cron** - cron expressions/scheduling
- **portals-delay** - delayed/scheduled tasks

### no_std Primitives
- **portals-collections** - portable collections
- **portals-sync** - sync primitives (mutex, rwlock, etc)
- **portals-alloc** - allocator interfaces

## Future Considerations

- **`spore-portals`**: Lua bindings (belongs in Spore, not here)

Potential interface improvements to consider later:

- [x] **Filesystem seek**: Add `Seek` trait for random access file operations
- [x] **Zero-copy reads**: Add `read_into(&mut self, buf: &mut [u8])` to `InputStream`

## ADRs

- 0001: `InsecureRandom` uses `&mut self` (PRNGs need state)
- 0002: Async runtime via tokio feature flag
- 0003: Stdio uses `&mut self` (matches std::io)
- 0004: Capability audit (4 violations fixed)
