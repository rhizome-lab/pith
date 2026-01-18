//! # Pith
//!
//! A standard library of interfaces for Rust.
//!
//! Pith provides capability-based interfaces that enable portability across
//! platforms (native, WASM, embedded) while reducing decision fatigue in
//! contested ecosystem domains.
//!
//! ## Philosophy
//!
//! - **Portability over power** - simpler interfaces that work everywhere
//! - **Reduce decision fatigue** - blessed choices in contested domains
//! - **Consistent APIs** - uniform patterns across all pith crates
//!
//! ## Interface Categories
//!
//! ### Primitives (High Value)
//!
//! Fundamental capabilities with genuinely different implementations per platform:
//!
//! | Crate | Domain |
//! |-------|--------|
//! | [`rhizome-pith-clocks`](https://docs.rs/rhizome-pith-clocks) | Time and timestamps |
//! | [`rhizome-pith-filesystem`](https://docs.rs/rhizome-pith-filesystem) | File I/O |
//! | [`rhizome-pith-io`](https://docs.rs/rhizome-pith-io) | Streams and polling |
//! | [`rhizome-pith-random`](https://docs.rs/rhizome-pith-random) | Randomness |
//! | [`rhizome-pith-sockets`](https://docs.rs/rhizome-pith-sockets) | Raw networking |
//!
//! ### Contested Domains (Medium Value)
//!
//! Areas where the ecosystem has multiple viable options:
//!
//! | Crate | Domain | Ecosystem Alternatives |
//! |-------|--------|----------------------|
//! | [`rhizome-pith-http`](https://docs.rs/rhizome-pith-http) | HTTP client/server | reqwest, ureq, hyper |
//! | [`rhizome-pith-sql`](https://docs.rs/rhizome-pith-sql) | SQL databases | rusqlite, sqlx, diesel |
//! | [`rhizome-pith-cache`](https://docs.rs/rhizome-pith-cache) | Caching with TTL | moka, cached, etc. |
//! | [`rhizome-pith-crypto`](https://docs.rs/rhizome-pith-crypto) | Cryptography | ring, rustcrypto |
//! | [`rhizome-pith-logging`](https://docs.rs/rhizome-pith-logging) | Logging | log, tracing |
//! | [`rhizome-pith-markdown`](https://docs.rs/rhizome-pith-markdown) | Markdown | pulldown-cmark, comrak |
//! | [`rhizome-pith-config`](https://docs.rs/rhizome-pith-config) | Configuration | figment, config |
//! | [`rhizome-pith-websocket`](https://docs.rs/rhizome-pith-websocket) | WebSocket | tungstenite, etc. |
//!
//! ## Solved Domains (Use Directly)
//!
//! For these domains, the Rust ecosystem has clear winners.
//! **Don't use pith wrappers - use these directly:**
//!
//! | Domain | Recommended Crate | Why |
//! |--------|------------------|-----|
//! | Serialization | [`serde`](https://docs.rs/serde) | Universal standard |
//! | JSON | [`serde_json`](https://docs.rs/serde_json) | De facto standard |
//! | CLI parsing | [`clap`](https://docs.rs/clap) | Dominant, excellent DX |
//! | URL parsing | [`url`](https://docs.rs/url) | WHATWG compliant |
//! | UUID | [`uuid`](https://docs.rs/uuid) | Full RFC 4122 support |
//! | Regex | [`regex`](https://docs.rs/regex) | Fast, safe, dominant |
//! | Async runtime | [`tokio`](https://docs.rs/tokio) | Ecosystem standard |
//! | Multi-pattern matching | [`aho-corasick`](https://docs.rs/aho-corasick) | Used by regex internally |
//! | Library error types | [`thiserror`](https://docs.rs/thiserror) | Dominant derive macro |
//! | App error handling | [`anyhow`](https://docs.rs/anyhow) | Dominant for apps |
//! | Incremental parsing | [`tree-sitter`](https://docs.rs/tree-sitter) | Dominant in tooling |
//!
//! Creating pith wrappers for these would add friction without benefit.
//!
//! ## What Pith Is Not
//!
//! Pith does not try to:
//! - **Wrap solved domains** - use serde, clap, regex directly
//! - **Abstract stylistic choices** - error handling, parser combinators
//! - **Replace the ecosystem** - we complement it, not compete
//! - **Be a framework** - pith is à la carte
//!
//! ## Usage
//!
//! Add the specific pith crates you need:
//!
//! ```toml
//! [dependencies]
//! rhizome-pith-filesystem = "0.1"
//! rhizome-pith-http = "0.1"
//! rhizome-pith-clocks = "0.1"
//!
//! # Native backends
//! rhizome-pith-filesystem-native = "0.1"
//! rhizome-pith-http-native = "0.1"
//! rhizome-pith-clocks-native = "0.1"
//! ```
//!
//! ## Backends
//!
//! Each interface has one or more backend implementations:
//!
//! - `*-native` - Native OS implementation
//! - `*-wasm` - WebAssembly implementation (partial)
//! - `*-mock` - Testing mocks
//!
//! ## Links
//!
//! - [GitHub Repository](https://github.com/rhizome-lab/pith)
//! - [Design Guidelines](https://github.com/rhizome-lab/pith/blob/main/DESIGN.md)

#![no_std]

// This crate is documentation-only.
// Use the specific pith-* crates for functionality.
