# eth-valkyoth-codec

Support crate for `eth`: bounded `no_std` Ethereum wire codec policy.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.16"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The RLP parser surface is covered by the workspace fuzz harness. See the
project fuzzing guide for target names, committed seed corpus handling, and
crash reproduction:

<https://github.com/valkyoth/eth/blob/main/docs/fuzzing.md>

The `0.16.0` release adds a list-header encoding helper for no-allocation
streaming encoders that compute an RLP list payload length before writing the
payload bytes.
