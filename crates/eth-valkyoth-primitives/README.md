# eth-valkyoth-primitives

Support crate for `eth`: core `no_std` Ethereum protocol primitives.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.10"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.10.0` release adds signed-chain ID helpers, release-visible unexpected
codec errors, and unambiguous transaction type constructors. Use the facade
crate documentation for examples and compatibility guidance.
