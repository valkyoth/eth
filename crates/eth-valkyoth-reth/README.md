<p align="center">
  <b>future Reth integration boundary for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-reth">Docs.rs</a>
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

# eth-valkyoth-reth

An opt-in integration marker for
[`eth`](https://crates.io/crates/eth). This package keeps the future Reth
adapter separate from the default `no_std` protocol graph.

```sh
cargo add eth --features reth
```

## Example

The currently available surface is only a marker:

```rust
let boundary = eth::reth::RethAdapterBoundary;
assert_eq!(boundary, eth::reth::RethAdapterBoundary);
```

No Reth dependency, RPC connection, execution engine, database adapter or node
is provided. Enabling this feature does not install or run Reth. Actual
integration requires future implementation and dependency admission under the
[release plan](https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md).

## License

MIT OR Apache-2.0, at your option.
