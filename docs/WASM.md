# WASM Backend Strategy

WASM backends implement portals interfaces for browser and WASI environments. This document tracks implementation status and priority.

## Implementation Tiers

### Tier 1: Straightforward (web APIs map well)

These interfaces have direct browser API equivalents.

| Interface | WASM Strategy | Status |
|-----------|---------------|--------|
| `portals-clocks` | `js-sys` Date, `performance.now()`, `gloo-timers` | Done |
| `portals-random` | `getrandom` with `wasm_js` feature | Done |
| `portals-http` | Fetch API via `gloo-net` | Done |
| `portals-websocket` | WebSocket API via `gloo-net` | Done |
| `portals-logging` | `console.*` via `web-sys` | Done |

### Portable (works on native and WASM)

These are in `crates/backends/portable/` and work everywhere.

| Interface | Backend | Notes |
|-----------|---------|-------|
| `portals-encoding` | `portals-encoding-portable` | `base64` crate is pure Rust |
| `portals-cron` | `portals-cron-portable` | Pure Rust parsing |

### May work in WASM (untested)

These native backends might compile to WASM but haven't been tested.

| Interface | Notes |
|-----------|-------|
| `portals-nanoid` | Uses `nanoid` crate (needs WASM-compatible randomness) |
| `portals-snowflake` | Uses `SystemTime` (needs clock injection for WASM) |

### Tier 2: Needs design decisions

These require architectural choices about how to handle WASM limitations.

| Interface | Challenge | Potential Strategy |
|-----------|-----------|-------------------|
| `portals-filesystem` | No real FS in browser | IndexedDB, in-memory, or OPFS |
| `portals-cache` | Persistence options vary | LocalStorage, IndexedDB, or in-memory |
| `portals-keyvalue` | Multiple storage backends | LocalStorage, IndexedDB |
| `portals-crypto` | SubtleCrypto is async-only | Wrap SubtleCrypto, handle async mismatch |
| `portals-timezone` | Need timezone database | `js-sys` Intl API or bundled tzdata |

### Tier 3: WASM limitations

These interfaces have fundamental limitations in browser WASM.

| Interface | Limitation | Options |
|-----------|------------|---------|
| `portals-sockets` | No raw TCP/UDP in browser | WebSocket-only, or server proxy |
| `portals-dns` | No direct DNS access | Server-backed API, or skip |
| `portals-sql` | No native SQLite | `sql.js` (SQLite compiled to WASM) |
| `portals-blobstore` | Cloud APIs need server | Server proxy, or mock-only |
| `portals-messaging` | Queue systems need server | Server proxy, or mock-only |
| `portals-observe` | Telemetry needs backend | Beacon API, or server proxy |

### Tier 4: Skip or mock-only

Some interfaces don't make sense in browser WASM. Provide mocks for testing only.

| Interface | Reason |
|-----------|--------|
| `portals-config` | Environment variables don't exist in browser |

## WASI vs Browser WASM

Some backends may work differently in WASI (server-side WASM) vs browser:

- **WASI**: Has filesystem, sockets, clocks via WASI APIs
- **Browser**: Uses Web APIs (Fetch, WebSocket, IndexedDB)

Consider whether to:
1. Have separate `-wasm-browser` and `-wasm-wasi` backends
2. Feature-flag within a single `-wasm` backend
3. Let WASI use native backends (many work via wasm32-wasi target)

## Dependencies

Common WASM dependencies:

```toml
[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = [...] }
gloo-net = "0.6"          # Higher-level Fetch/WebSocket
gloo-timers = "0.3"       # setTimeout/setInterval
```

## Testing

WASM backends need browser testing via `wasm-pack test --headless --chrome/firefox`:

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
```

## Progress Tracking

- [x] Tier 1 complete
- [ ] Tier 2 complete
- [ ] Tier 3 evaluated (decide skip vs implement)
