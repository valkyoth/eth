# eth-valkyoth-hash

Support crate for `eth`: `no_std` Keccak-256 hashing boundary traits.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.36"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. The `0.11.0` release adds the optional
`tiny-keccak` feature and `TinyKeccak256` backend for applications that want a
reviewed software Keccak-256 implementation without changing the default
dependency graph. The backend is covered by empty-input, `abc`, and chunking
known-answer tests.

```toml
[dependencies]
eth = { version = "0.27", features = ["keccak-tiny"] }
```

The default build still defines only the trait boundary and conformance
helpers. `tiny-keccak` does not expose a documented sponge-state zeroization
contract, so deployments that require state clearing should provide a custom
hasher with an explicit sanitization contract.
