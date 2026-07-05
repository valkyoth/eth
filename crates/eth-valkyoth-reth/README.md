# eth-valkyoth-reth

Support crate for `eth`: future Reth integration boundary.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.37"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.
