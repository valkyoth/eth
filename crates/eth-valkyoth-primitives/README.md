# eth-valkyoth-primitives

Support crate for `eth`: core `no_std` Ethereum protocol primitives.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.25"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.11.0` release implements the public codec `RlpEncode` and `RlpDecode`
traits for primitive domain types so reviewed structs can derive RLP without
duplicating primitive canonicality logic.
