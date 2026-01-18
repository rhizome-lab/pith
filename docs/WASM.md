# WASM Backend Strategy

WASM backends implement pith interfaces for browser and WASI environments. This document tracks implementation status and priority.

## Implementation Tiers

### Tier 1: Straightforward (web APIs map well)

These interfaces have direct browser API equivalents.

| Interface | WASM Strategy | Status |
|-----------|---------------|--------|
| `pith-clocks` | `js-sys` Date, `performance.now()`, `gloo-timers` | Done |
| `pith-random` | `getrandom` with `wasm_js` feature | Done |
| `pith-http` | Fetch API via `gloo-net` | Done |
| `pith-websocket` | WebSocket API via `gloo-net` | Done |
| `pith-logging` | `console.*` via `web-sys` | Done |

### Portable (works on native and WASM)

These are in `crates/backends/portable/` and work everywhere.

| Interface | Backend | Notes |
|-----------|---------|-------|
| `pith-encoding` | `pith-encoding-portable` | `base64` crate is pure Rust |
| `pith-cron` | `pith-cron-portable` | Pure Rust parsing |

### May work in WASM (untested)

These native backends might compile to WASM but haven't been tested.

| Interface | Notes |
|-----------|-------|
| `pith-nanoid` | Uses `nanoid` crate (needs WASM-compatible randomness) |
| `pith-snowflake` | Uses `SystemTime` (needs clock injection for WASM) |

### Tier 2: Needs design decisions

These require architectural choices about how to handle WASM limitations.

| Interface | Challenge | Potential Strategy |
|-----------|-----------|-------------------|
| `pith-filesystem` | No real FS in browser | IndexedDB, in-memory, or OPFS |
| `pith-cache` | Persistence options vary | LocalStorage, IndexedDB, or in-memory |
| `pith-keyvalue` | Multiple storage backends | LocalStorage, IndexedDB |
| `pith-crypto` | SubtleCrypto is async-only | Wrap SubtleCrypto, handle async mismatch |
| `pith-timezone` | Need timezone database | `js-sys` Intl API or bundled tzdata |

### Tier 3: WASM limitations

These interfaces have fundamental limitations in browser WASM.

| Interface | Limitation | Options |
|-----------|------------|---------|
| `pith-sockets` | No raw TCP/UDP in browser | WebSocket-only, or server proxy |
| `pith-dns` | No direct DNS access | Server-backed API, or skip |
| `pith-sql` | No native SQLite | `sql.js` (SQLite compiled to WASM) |
| `pith-blobstore` | Cloud APIs need server | Server proxy, or mock-only |
| `pith-messaging` | Queue systems need server | Server proxy, or mock-only |
| `pith-observe` | Telemetry needs backend | Beacon API, or server proxy |

### Tier 4: Skip or mock-only

Some interfaces don't make sense in browser WASM. Provide mocks for testing only.

| Interface | Reason |
|-----------|--------|
| `pith-config` | Environment variables don't exist in browser |

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
