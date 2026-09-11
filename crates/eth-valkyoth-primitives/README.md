<p align="center">
  <b>constant-time no_std Ethereum primitives for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-primitives">Docs.rs</a>
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

# eth-valkyoth-primitives

Explicit `no_std` Ethereum value domains for
[`eth`](https://crates.io/crates/eth): addresses, hashes, chain IDs, currency,
gas, nonces, indexes and transaction types. The types share canonical integer
validation with the codec instead of maintaining a second parser.

Most applications should use the facade:

```sh
cargo add eth
```

## Example

```rust
use eth::primitives::{Address, ChainId, Gas, Nonce, Wei};

let chain = ChainId::new(1);
let recipient = Address::from_bytes([0_u8; 20]);
let value = Wei::from_u128(1_000_000_000);
let gas = Gas::new(21_000);
let nonce = Nonce::new(0);
assert_eq!(chain, ChainId::new(1));
assert_eq!(value, Wei::from_u128(1_000_000_000));
assert_eq!((recipient, gas, nonce), (Address::from_bytes([0_u8; 20]), Gas::new(21_000), Nonce::new(0)));
```

A well-formed domain value is not proof of transaction validity, ownership,
chain membership or sufficient balance. Address and hash bytes are public
identifiers, not secret containers. Primitive RLP helpers enforce canonical
encoding; untrusted compound operations also need the codec's shared session.

See the [facade examples](https://docs.rs/eth) and
[specification matrix](https://github.com/valkyoth/eth/blob/main/docs/SPEC_MATRIX.md).

## License

MIT OR Apache-2.0, at your option.
