# eth-valkyoth-hash

Support crate for `eth`: `no_std` Keccak-256 hashing boundary traits.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.26"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. The `0.10.1` release aligns the primitive
dependency range for `eth` `0.26.0`. The `0.10.0` release defines the trait
boundary and empty-input conformance helpers for both `Default` and explicitly
configured hashers; it does not provide or admit a default Keccak
implementation crate.
