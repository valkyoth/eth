# eth-valkyoth-signer

Support crate for `eth`: future signer isolation boundary.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.26"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. The `0.7.1` release aligns the primitive dependency
range for `eth` `0.26.0`. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.
