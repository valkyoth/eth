<p align="center">
  <b>bounded no_std Ethereum wire codec policy for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-codec">Docs.rs</a>
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

# eth-valkyoth-codec

Canonical Ethereum RLP and cumulative decode budgets for
[`eth`](https://crates.io/crates/eth). This lower-level `no_std` crate
parses borrowed scalars, lists and integers, and encodes into caller-owned
buffers. Most applications should use the facade:

```sh
cargo add eth
```

## Example

```rust
use eth::codec::{DecodeLimits, decode_rlp_u64, encode_rlp_integer};

let limits = DecodeLimits::reviewed_policy(32, 4, 4, 32, 4, 4);
let mut output = [0_u8; 3];
let written = encode_rlp_integer(&[4, 0], &mut output)?;
assert_eq!(decode_rlp_u64(&output[..written], limits)?, 1024);
# Ok::<(), eth::error::DecodeError>(())
```

## Security Contract

For nested untrusted input, retain one non-copyable `DecodeSession` and use
the `*_in_session` APIs for structural parsing and every subsequent traversal.
Legacy iterators are only suitable for trusted or independently bounded data.
Review every policy field against concurrency, memory and protocol needs;
changing one template value does not establish a safe deployment policy.

Budgets cover item visits, nesting, proof nodes, compact-path nibbles, trie
values and actual hash work. Hash-capacity preflight does not debit the ledger;
each actual hash must still be charged. Payload-only integer helpers validate
canonicality, not wire framing or an operation-wide budget.

`RlpEncode` and `RlpDecode` support optional derives. Discard the output buffer
on any encoding error: aggregate encoders can have written an earlier prefix.

See [decode sessions](https://github.com/valkyoth/eth/blob/main/docs/decode-session.md)
and [fuzzing](https://github.com/valkyoth/eth/blob/main/docs/fuzzing.md).

## License

MIT OR Apache-2.0, at your option.
