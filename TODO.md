# TODO

## Roadmap

1. [x] Diverge from WASI where ergonomics warrant
2. [x] Add more interfaces (crypto, encoding, sql)
3. [x] Reference implementations for native targets
4. [ ] `spore-pith` - Lua bindings for Spore

## Future Considerations

Potential interface improvements to consider later:

- **Filesystem seek**: Add `Seek` trait for random access file operations
- **Zero-copy reads**: Add `read_into(&mut self, buf: &mut [u8])` to `InputStream`
- **Unified streams**: Have filesystem return `pith-io` streams instead of its own `Read`/`Write` traits (more composable, but adds crate coupling)

## ADRs

- 0001: `InsecureRandom` uses `&mut self` (PRNGs need state)
- 0002: Async runtime via tokio feature flag
- 0003: Stdio uses `&mut self` (matches std::io)
