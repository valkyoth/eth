# eth-valkyoth-hash

Support crate for `eth`: `no_std` Keccak-256 hashing boundary traits.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.9"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. The `0.9.3` release defines the trait boundary only;
it does not provide or admit a default Keccak implementation crate.
