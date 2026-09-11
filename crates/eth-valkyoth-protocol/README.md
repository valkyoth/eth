<p align="center">
  <b>fork-aware no_std protocol validation for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-protocol">Docs.rs</a>
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

# eth-valkyoth-protocol

Borrowed Ethereum transaction, header, receipt and withdrawal models, canonical
encoding, signing preimages and explicit fork contexts for
[`eth`](https://crates.io/crates/eth). This is a `no_std` protocol library,
not a node or a complete state-transition validator.

```sh
cargo add eth
```

## Example

Classify a typed envelope under a reviewed byte and item policy:

```rust
use eth::codec::DecodeLimits;
use eth::protocol::{TransactionEnvelope, decode_transaction_envelope};

let limits = DecodeLimits::reviewed_policy(32, 4, 4, 32, 4, 4);
let envelope = decode_transaction_envelope(&[0x02, 0xc0], limits)?;
assert!(matches!(envelope, TransactionEnvelope::Typed(_)));
# Ok::<(), eth::error::TransactionEnvelopeError>(())
```

Classification of this empty type-2 payload does not make it a valid EIP-1559
transaction. Use the specific decoder and the relevant validity checks next.

## Current Scope

- Legacy, EIP-2930, EIP-1559, EIP-4844 and EIP-7702 decode/encode and signing preimages.
- Shared-session envelope and transaction decoding, including nested access lists,
  storage keys, blob hashes and authorization tuples.
- Syntactic legacy through Prague headers, receipts and EIP-4895 withdrawals.
- Caller-reviewed chain/fork tables and explicit validation-state transitions.
- EIP-7702 context checks, including nonempty authorizations, fee order and
  caller-supplied gas/account policy; invalid individual authorizations are skipped.

## Security Contract

Untrusted traversal must use `*_in_session` APIs with the same cumulative
ledger. An `Unvalidated*` result proves syntax, not signatures, sender identity,
state validity, fee/blob-gas policy or KZG correctness. Blob transactions still
need fork-aware nonempty/hash-version checks. Header hashes do not authenticate
the header, receipt syntax does not prove a receipt root, and withdrawal syntax
does not prove consensus dequeue correctness or apply balances.

Use `ChainSpec::try_new` for dynamic or generated fork tables;
`ChainSpec::new` is only for hand-audited static tables. Caller context must
come from an independently trusted chain view. Recovery and proof checking live
in the verification layer; transaction execution lives in the EVM layers.

See the [specification matrix](https://github.com/valkyoth/eth/blob/main/docs/SPEC_MATRIX.md)
for exact implemented boundaries and versioned completion work.

## License

MIT OR Apache-2.0, at your option.
