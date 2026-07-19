<p align="center">
  <b>optional derive macros for audited eth support crates.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-derive">Docs.rs</a>
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

# eth-valkyoth-derive

Optional derive macros for `eth` support crates.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.52.4"
```

Crates.io: <https://crates.io/crates/eth>

This package is only for users who explicitly opt into derive ergonomics. It
provides sanitization derives and reviewed public RLP derives.

```toml
[dependencies]
eth-valkyoth-sanitization = { version = "0.7", features = ["derive"] }
```

The `0.17` series exports `RlpEncode` and `RlpDecode` derives for reviewed
simple structs. Generated decoders require `DecodeLimits`, encode structs as
RLP lists in Rust declaration order, reject generics/enums/unions, and require
skipped fields to use `#[eth_rlp(skip, default, reason = "...")]`.

Supported field attribute:

```rust
#[eth_sanitization(skip, reason = "non-secret label")]
```

RLP skipped-field attribute:

```rust
#[eth_rlp(skip, default, reason = "derived cache")]
```

Supported container attribute:

```rust
#[eth_sanitization(crate = "::my_sanitization_path")]
```

RLP container attribute:

```rust
#[eth_rlp(crate = "::my_codec_path")]
```

Enum derives are rejected because inactive variant backing bytes may retain
secret material after variant changes. Use a struct wrapper for secret-bearing
state.
