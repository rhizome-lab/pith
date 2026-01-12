# CLAUDE.md

Pith is a standard library of interfaces, inspired by WASI.

## Structure

```
crates/
├── interfaces/     # Trait definitions (pith-*)
└── backends/       # Implementations
    ├── native/     # Native OS implementations
    └── wasm/       # WASM implementations
```

### Interfaces

Each crate in `interfaces/` defines traits for a capability domain:
- `pith-clocks` - time
- `pith-cli` - command-line environment
- `pith-filesystem` - file I/O
- `pith-http` - HTTP
- `pith-io` - streams and polling
- `pith-random` - randomness
- `pith-sockets` - networking

### Backends

Implementations go in `backends/<target>/`. For example:
- `backends/native/pith-clocks-native` - native clock implementation
- `backends/wasm/pith-clocks-wasm` - WASM clock implementation

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
