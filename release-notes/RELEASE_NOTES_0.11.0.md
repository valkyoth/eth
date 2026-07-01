# eth 0.11.0 Release Notes

Status: release candidate ready; waiting for final GitHub checks before tag

## Summary

`0.11.0` starts the transaction-envelope phase with a small EIP-2718 shell.
The release classifies legacy transaction bytes and typed transaction bytes
without decoding transaction fields, signatures, sender recovery, or fork
validity.

## Included So Far

- Added `eth_valkyoth_protocol::decode_transaction_envelope`.
- Added `TransactionEnvelope` with `Legacy` and `Typed` variants.
- Added `TypedTransactionEnvelope` with a typed transaction byte and opaque
  payload slice.
- Added stable `TransactionEnvelopeError` codes, messages, categories,
  formatting, and `std::error::Error` support behind the `std` feature.
- Legacy transaction envelopes must be exactly one bounded RLP list.
- Typed transaction envelopes enforce the input byte budget before exposing the
  opaque payload.
- Rejected RLP scalar prefixes `0x80..=0xbf` as malformed transaction
  envelopes.
- Rejected the EIP-2718 reserved `0xff` prefix with a dedicated error.
- Rejected typed prefix `0x00` as unsupported by this shell because this crate
  keeps `TransactionType::LEGACY` as the legacy domain value.
- Added a dedicated `transaction_envelope` fuzz target and committed seed
  corpus cases for typed, legacy, scalar-prefix, typed-zero, reserved-prefix,
  and trailing-legacy inputs.
- Addressed pentest findings by tying the protocol typed-prefix maximum to
  `TransactionType::MAX_TYPED`, documenting that unknown nonzero typed prefixes
  are accepted only as opaque envelopes, and recursing through nested legacy
  list payloads in the transaction-envelope fuzz target.
- Refreshed `spec-lock.toml` and `docs/SPEC_MATRIX.md` with official EIP and
  execution-spec revisions checked on 2026-07-01.
- Updated release metadata so `eth-valkyoth-protocol` and `eth` publish for
  this release.
- Added `scripts/release_0_11_gate.sh`.

## Known Limitations

- Typed transaction payloads are opaque in this release.
- Legacy transaction fields are not decoded yet.
- No transaction signature, sender-recovery, gas, fee, nonce, access-list, blob,
  chain, or fork validation is performed.
- No transaction encoding is added in this release.

## Release Gate

- Initial pentest findings were remediated and committed.
- Maintainer retest found no remaining findings for the release candidate.
- Permanent report: `security/pentest/v0.11.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth --all-targets --all-features -- -D warnings
scripts/materialize_fuzz_seeds.py --check
cargo check --manifest-path fuzz/Cargo.toml
scripts/checks.sh
scripts/release_0_11_gate.sh
cargo deny check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
