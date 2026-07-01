# eth 0.11.0 Release Notes

Status: implementation complete; pentest required before tag

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

## Still Required Before Tag

- Maintainer pentest must be run for the exact implementation commit.
- Any pentest findings must be fixed and retested.
- A permanent report must be written at `security/pentest/v0.11.0.md`.
- GitHub checks must pass on the final release report commit.

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
