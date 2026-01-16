# TODO

## Roadmap

1. [x] Diverge from WASI where ergonomics warrant
2. [x] Add more interfaces (crypto, encoding, sql)
3. [x] Reference implementations for native targets
4. [x] Unified streams (filesystem returns `pith-io` streams)
5. [x] WebSocket and DNS interfaces + native backends
   - [x] pith-websocket interface + pith-websocket-native (via tungstenite)
   - [x] pith-dns interface + pith-dns-native (via hickory-resolver)
6. [x] Protocol implementations (`crates/protocols/`)
   - [x] pith-http1 (HTTP/1.1 wire format parsing/serialization)

## Backlog

WASI Phase 1/2 interfaces:

- [x] **pith-url** - URL parsing (WASI Phase 1)
- [x] **pith-timezone** - timezone handling (WASI Phase 2)
- [x] **pith-config** - runtime configuration (WASI Phase 1)
- [x] **pith-logging** - structured logging (WASI Phase 1)
- [x] **pith-keyvalue** - key-value store (WASI Phase 1)
- [x] **pith-blobstore** - blob storage (WASI Phase 1)
- [x] **pith-observe** - observability/telemetry (WASI Phase 1)
- [x] **pith-messaging** - message queues (WASI Phase 1)

### Unstable / Deferred

APIs likely to change or too platform-specific:

- **pith-nn** - neural network inference (WASI Phase 2, ML APIs evolving fast)
- **pith-gfx** - graphics (WASI Phase 2, graphics APIs notoriously unstable)
- **pith-threads** - threading (WASI Phase 1, complex semantics)
- **pith-i2c / pith-spi / pith-usb** - hardware interfaces (niche, platform-specific)
- **pith-distributed-lock** - distributed locking (niche)

## Potential Interfaces

Application-level interfaces to consider (beyond WASI):

### Identity / Auth
- **pith-jwt** - JWT parsing/validation/creation
- **pith-oauth** - OAuth flow abstractions
- **pith-session** - session management

### Data / Validation
- **pith-cache** - caching with TTL/LRU policies
- **pith-validation** - schema validation
- **pith-serialization** - JSON/TOML/YAML/etc (or per-format crates)

### Text / Formatting
- **pith-i18n** - internationalization/localization
- **pith-markdown** - markdown parsing/rendering
- **pith-template** - templating

### Media
- **pith-image** - image metadata/transforms
- **pith-audio** - audio metadata/processing
- **pith-video** - video metadata

### Communication
- **pith-email** - email sending/parsing
- **pith-notification** - push notifications

### Identifiers
- **pith-uuid** - UUID generation/parsing
- **pith-nanoid** - nanoid generation
- **pith-snowflake** - snowflake IDs

### Scheduling
- **pith-cron** - cron expressions/scheduling
- **pith-delay** - delayed/scheduled tasks

### no_std Primitives
- **pith-collections** - portable collections
- **pith-sync** - sync primitives (mutex, rwlock, etc)
- **pith-alloc** - allocator interfaces

## Future Considerations

- **`spore-pith`**: Lua bindings (belongs in Spore, not here)

Potential interface improvements to consider later:

- **Filesystem seek**: Add `Seek` trait for random access file operations
- **Zero-copy reads**: Add `read_into(&mut self, buf: &mut [u8])` to `InputStream`

## ADRs

- 0001: `InsecureRandom` uses `&mut self` (PRNGs need state)
- 0002: Async runtime via tokio feature flag
- 0003: Stdio uses `&mut self` (matches std::io)
