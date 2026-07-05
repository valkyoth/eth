# eth-valkyoth-evm

Support crate for `eth`: explicit EVM adapter boundary and REVM dependency
admission review.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.37"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.8.0` support-crate release, shipped with `eth` `0.37.0`, records the
REVM dependency review result. REVM is not admitted yet: the latest line
requires Rust `1.91.0`, and the newest Rust `1.90.0`-compatible candidates do
not pass this repository's cargo-deny policy.
