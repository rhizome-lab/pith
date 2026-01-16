# CLAUDE.md

Pith is a standard library of interfaces, inspired by WASI.

## Structure

```
crates/
├── interfaces/     # Trait definitions (what you can do)
├── protocols/      # Wire format implementations (how it's encoded)
└── backends/       # Platform implementations (how it runs)
    ├── native/     # Native OS implementations
    └── wasm/       # WASM implementations
```

### Interfaces (`interfaces/`)

**What:** Traits defining capabilities. No implementations, just contracts.

**Examples:**
- `rhizome-pith-http` → `HttpClient`, `HttpHandler` traits
- `rhizome-pith-websocket` → `WebSocketClient`, `WebSocketServer` traits
- `rhizome-pith-dns` → `Resolver` trait

**Rule:** If it's "what can I do?" (client, server, read, write), it's an interface.

### Protocols (`protocols/`)

**What:** Wire format parsing/serialization. Pure Rust, no platform deps.

**Examples:**
- `rhizome-pith-http1` → HTTP/1.1 request/response parsing

**Rule:** If it's "how is data encoded on the wire?", it's a protocol. These are shared across backends - both native and wasm can use the same HTTP parser.

### Backends (`backends/<target>/`)

**What:** Platform-specific implementations of interface traits. Wraps libraries.

**Examples:**
- `rhizome-pith-websocket-native` → implements `WebSocketClient` using tungstenite
- `rhizome-pith-dns-native` → implements `Resolver` using hickory-resolver

**Rule:** If it wraps a library or uses platform APIs, it's a backend.

## Behavioral Patterns

From ecosystem-wide session analysis:

- **Question scope early:** Before implementing, ask whether it belongs in this crate/module
- **Check consistency:** Look at how similar things are done elsewhere in the codebase
- **Implement fully:** No silent arbitrary caps, incomplete pagination, or unexposed trait methods
- **Name for purpose:** Avoid names that describe one consumer
- **Verify before stating:** Don't assert API behavior or codebase facts without checking

## Design

- Interfaces define traits, backends provide implementations
- Capability-based (no global/ambient access)
- Async-first where blocking is possible
- Mirror WASI structure but diverge for ergonomics where sensible
- Prefer portability over power: simpler interfaces that work everywhere beat feature-rich ones that only work on some platforms

See [DESIGN.md](DESIGN.md) for detailed API design guidelines.

## Documentation Maintenance

When making scope decisions (adding/removing interfaces, deferring to ecosystem crates, etc.), update these files:

| Decision | Files to Update |
|----------|-----------------|
| Add new interface | `Cargo.toml` (workspace), `DESIGN.md` (category tables) |
| Remove interface / defer to ecosystem | `docs/RECOMMENDATIONS.md` (solved domains table), `crates/pith/src/lib.rs` (docs) |
| Add to "watching" list | `docs/RECOMMENDATIONS.md` (contested domains table) |
| Change design guidelines | `DESIGN.md` |
| Change scope/philosophy | `docs/RECOMMENDATIONS.md` ("What Pith Is/Is Not"), `crates/pith/src/lib.rs` |

The `rhizome-pith` meta-crate (`crates/pith/src/lib.rs`) is the public-facing docs.rs documentation - keep it in sync with recommendations.
