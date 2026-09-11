<p align="center">
  <b>explicit RPC trust-policy boundary for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-rpc">Docs.rs</a>
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

# eth-valkyoth-rpc

Explicit RPC trust-policy names for
[`eth`](https://crates.io/crates/eth), isolated behind an optional feature.

```sh
cargo add eth --features rpc
```

## Example

```rust
use eth::rpc::RpcTrustModel;

let policy = RpcTrustModel::trusted_with_explicit_acknowledgment(
    "This application explicitly trusts its configured endpoint",
);
assert!(matches!(policy, RpcTrustModel::Trusted { .. }));
```

This package currently contains policy types only. Selecting `Verified` does
not fetch or verify proofs, and selecting `Quorum` does not query independent
providers. There is no HTTP/WebSocket client, endpoint validation, retry engine
or network activity. Applications must implement and enforce their chosen
policy; a value alone is not evidence.

Full provider and RPC behavior has explicit versions in the
[release plan](https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md).

## License

MIT OR Apache-2.0, at your option.
