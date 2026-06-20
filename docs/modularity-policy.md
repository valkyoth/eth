# eth Modularity Policy

`eth` must not become a monolithic source tree.

Rules:

- Main crate `eth` is a facade, not the implementation home.
- Parsing, validation, verification, execution, RPC, signing, sanitization,
  derive macros, Reth, and tests live in separate crates.
- Keep `lib.rs` as module wiring and public API shape.
- Non-generated Rust source files must stay under 500 lines.
- A file approaching 300 lines should be reviewed for splitting.
- Adapter crates may depend inward on stable protocol crates; core crates must
  not depend outward on adapters.
- Feature flags must not silently enable networking, signing, Reth, P2P, or
  local key storage.

The local gate is:

```bash
scripts/validate-modularity-policy.sh check
```
