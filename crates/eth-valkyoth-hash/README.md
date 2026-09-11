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

The `no_std` Keccak-256 backend boundary for
[`eth`](https://crates.io/crates/eth). Default builds expose traits, digest
domains and conformance helpers without selecting a hash implementation.

Opt into the reviewed software backend through the facade:

```sh
cargo add eth --features keccak-tiny
```

## Example

```rust
use eth::hash::{KECCAK256_ABC, TinyKeccak256, hash_one};

let digest = hash_one(TinyKeccak256::default(), b"abc");
assert_eq!(<[u8; 32]>::from(digest), KECCAK256_ABC);
```

Ethereum uses Keccak-256, not SHA3-256. Backend admission requires known-answer,
chunking and differential tests. The optional `tiny-keccak` backend is not
enabled by default and does not promise sponge-state zeroization. Secret
hashing paths that need state clearing must supply a backend with an explicit
sanitization contract.

See the [hash boundary](https://github.com/valkyoth/eth/blob/main/docs/keccak-boundary.md).

## License

MIT OR Apache-2.0, at your option.
