# Portals

Standard library interfaces.

Capability-based, async-first interfaces inspired by WASI, designed to be implementable across runtimes.

## Crates

| Crate | Description | WASI Equivalent |
|-------|-------------|-----------------|
| `portals-clocks` | Wall clock, monotonic clock | `wasi:clocks` |
| `portals-cli` | Args, environment, stdio | `wasi:cli` |
| `portals-crypto` | Hashing, HMAC, encryption, signatures | - |
| `portals-encoding` | Base64, hex, URL encoding | - |
| `portals-filesystem` | Files, directories | `wasi:filesystem` |
| `portals-http` | HTTP client/server | `wasi:http` |
| `portals-io` | Streams, polling | `wasi:io` |
| `portals-random` | Secure and insecure RNG | `wasi:random` |
| `portals-sockets` | TCP, UDP, DNS | `wasi:sockets` |
| `portals-sql` | Database connections, queries | - |

## Structure

```
crates/
├── interfaces/     # Trait definitions (portals-*)
└── backends/       # Implementations
    ├── native/     # Native OS implementations
    └── wasm/       # WASM implementations
```

## Design Principles

- **Capability-based**: Access is granted through capability objects, not ambient authority
- **Async-first**: Operations that may block return futures
- **Minimal**: Interfaces define traits, backends provide implementations
- **Portable**: Implementable on native, WASM, and embedded targets
