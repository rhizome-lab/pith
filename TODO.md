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

WASI Phase 1/2 interfaces to consider:

- [ ] **pith-timezone** - timezone handling (WASI Phase 2)
- [ ] **pith-keyvalue** - key-value store (WASI Phase 1)
- [ ] **pith-logging** - structured logging (WASI Phase 1)
- [ ] **pith-config** - runtime configuration (WASI Phase 1)
- [ ] **pith-url** - URL parsing (WASI Phase 1)
- [ ] **pith-messaging** - message queues (WASI Phase 1)
- [ ] **pith-observe** - observability/telemetry (WASI Phase 1)
- [ ] **pith-blobstore** - blob storage (WASI Phase 1)

### Unstable / Deferred

APIs likely to change or too platform-specific:

- **pith-nn** - neural network inference (WASI Phase 2, ML APIs evolving fast)
- **pith-gfx** - graphics (WASI Phase 2, graphics APIs notoriously unstable)
- **pith-threads** - threading (WASI Phase 1, complex semantics)
- **pith-i2c / pith-spi / pith-usb** - hardware interfaces (niche, platform-specific)
- **pith-distributed-lock** - distributed locking (niche)

## Future Considerations

- **`spore-pith`**: Lua bindings (belongs in Spore, not here)

Potential interface improvements to consider later:

- **Filesystem seek**: Add `Seek` trait for random access file operations
- **Zero-copy reads**: Add `read_into(&mut self, buf: &mut [u8])` to `InputStream`

## ADRs

- 0001: `InsecureRandom` uses `&mut self` (PRNGs need state)
- 0002: Async runtime via tokio feature flag
- 0003: Stdio uses `&mut self` (matches std::io)
