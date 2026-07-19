<p align="center">
  <b>no_std Keccak-256 boundary traits for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-hash">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/eth">
    <img src="https://raw.githubusercontent.com/valkyoth/eth/main/.github/images/eth.webp" alt="eth Rust crate overview">
  </a>
</p>

# eth-valkyoth-hash

Support crate for `eth`: `no_std` Keccak-256 hashing boundary traits.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.52.3"
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
