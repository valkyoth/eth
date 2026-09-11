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

Opt-in RLP and sanitization derives for the
[`eth`](https://crates.io/crates/eth) workspace. Procedural macros run on the
build host; generated runtime code can remain `no_std`. They are not default
facade dependencies.

## RLP Example

The generated default path names the codec crate directly:

```sh
cargo add eth-valkyoth-derive eth-valkyoth-codec
```

```rust
use eth_valkyoth_codec::{DecodeLimits, RlpDecode, RlpEncode};

#[derive(Debug, PartialEq, eth_valkyoth_derive::RlpEncode, eth_valkyoth_derive::RlpDecode)]
struct Counter {
    nonce: u64,
}

let value = Counter { nonce: 1 };
let mut output = [0_u8; 8];
let written = value.encode_rlp(&mut output)?;
let limits = DecodeLimits::reviewed_policy(32, 4, 4, 32, 4, 4);
assert_eq!(Counter::decode_rlp(&output[..written], limits)?, value);
# Ok::<(), eth_valkyoth_codec::RlpDeriveError>(())
```

RLP derives encode simple structs as lists in declaration order and reject
generics, enums and unions. Skipped fields require
`#[eth_rlp(skip, default, reason = "derived cache")]`. A custom codec path uses
`#[eth_rlp(crate = "::my_codec_path")]`. Discard output on any error, including
failures after earlier fields were written.

## Sanitization

```sh
cargo add eth-valkyoth-sanitization --features derive
```

`SecureSanitize` implements the field-wise `DropSafeSanitize` contract;
`SecureSanitizeOnDrop` requires `DropSafeSanitize + Unpin`. Enum derives are
rejected because inactive variant storage may retain secrets. Structs still
require ownership/copy/logging review.

A skipped nonsecret field needs
`#[eth_sanitization(skip, reason = "non-secret label")]`; a custom bridge path
uses `#[eth_sanitization(crate = "::my_sanitization_path")]`.
See the [sanitization bridge](https://docs.rs/eth-valkyoth-sanitization).

## License

MIT OR Apache-2.0, at your option.
