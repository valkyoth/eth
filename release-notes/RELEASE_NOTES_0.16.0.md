# eth 0.16.0 Release Notes

Status: pentest passed; final GitHub checks pending before tag

## Summary

`0.16.0` adds canonical no-allocation transaction envelope encoding for the
decoded transaction domains already admitted by the protocol crate:

- legacy transactions;
- EIP-2930 access-list transactions;
- EIP-1559 dynamic-fee transactions;
- EIP-4844 blob transactions.

The encoder writes into caller-provided buffers. It is intended for field
models that were already decoded by this crate or constructed from trusted
domain values. Encoding a transaction does not validate signatures, recover
senders, prove chain binding, check fee ordering, account for gas/blob gas, or
prove fork validity.

## Added

- Added `TransactionEncodeError` and `TransactionEncodeErrorCategory`.
- Added `UnvalidatedTransaction` as a unified admitted transaction domain enum.
- Added encoded-length and encode functions for legacy, access-list,
  dynamic-fee, and blob transactions.
- Added unified `encoded_transaction_len` and `encode_transaction` dispatch.
- Added public type-byte constants for EIP-2930, EIP-1559, and EIP-4844
  transactions.
- Added `eth-valkyoth-codec::encoded_rlp_list_header_len` and
  `eth-valkyoth-codec::encode_rlp_list_header` for no-allocation streaming
  encoders.
- Added failure-path tests that verify transaction envelope type/header bytes
  are not committed when payload writer completion fails.

## Security Notes

- Encoders produce canonical RLP for already admitted transaction field models.
- Unknown typed transaction payloads are intentionally not accepted by the
  unified encoder. Callers that need lossless forwarding of unsupported types
  should carry the original bytes instead of constructing a field model this
  crate cannot validate.
- Access lists and blob versioned hash lists reuse the already validated
  borrowed RLP list bytes from the decoder.
- Signature fields are encoded as canonical unsigned U256 integers, but
  secp256k1 scalar validity, low-s policy, y-parity semantics, and sender
  recovery remain outside this release.
- EIP-1559 fee ordering and EIP-4844 blob fee/hash/KZG/data-availability checks
  remain validation-layer responsibilities.
- First pentest pass fixed transaction encoder envelope writes so list headers
  and typed transaction bytes are committed only after payload writer completion.

## Release Gate

- Pentest completed with remediation and clean retest.
- Permanent report path: `security/pentest/v0.16.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-codec -p eth-valkyoth-protocol -p eth --all-features
cargo clippy -p eth-valkyoth-codec -p eth-valkyoth-protocol -p eth --all-targets --all-features -- -D warnings
cargo check --manifest-path fuzz/Cargo.toml
scripts/release_0_16_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
