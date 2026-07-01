# eth 0.15.0 Release Notes

Status: release candidate ready; pentest passed

## Summary

`0.15.0` adds unvalidated EIP-4844 blob transaction field decoding for typed
transaction byte `0x03`. The decoder accepts
`0x03 || rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas,
gas_limit, to, value, data, access_list, max_fee_per_blob_gas,
blob_versioned_hashes, y_parity, r, s])` and returns a borrowed field model.

The model is intentionally syntactic. It does not validate signatures, recover
senders, enforce chain binding, check fee ordering, check blob base-fee
adequacy, verify KZG proofs, prove data availability, enforce blob-hash version
policy, apply block blob-gas accounting, validate account state, or prove fork
validity.

## Added

- Added `eth_valkyoth_protocol::decode_blob_transaction`.
- Added `UnvalidatedBlobTransaction` with chain ID, nonce, max priority fee per
  gas, max fee per gas, gas limit, required call target, value, input data,
  access list, max fee per blob gas, blob versioned hashes, y parity, and raw
  canonical U256 signature words.
- Added borrowed `BlobVersionedHashes` and `BlobVersionedHashItems`.
- Added `BlobTransactionField`, `BlobTransactionDecodeError`, and stable error
  categories/codes for wrong type, wrong field count, malformed fields, invalid
  target length, invalid access-list shape, invalid blob versioned hash length,
  invalid y parity, and resource exhaustion.
- Re-exported the blob transaction decode errors from `eth::error`.
- Extended the transaction-envelope fuzz target to also drive EIP-4844 blob
  transaction decoding.

## Security Notes

- Blob transaction field decoding is syntactic and bounded only.
- The `to` field must be exactly a 20-byte address; blob transactions cannot be
  contract creation transactions.
- Blob versioned hashes are eager-validated as 32-byte scalars before returning
  the borrowed model. Iterating hashes later re-parses the same bounded bytes.
- Empty blob versioned hash lists are accepted by this syntactic decoder.
  EIP-4844 execution validation requires at least one blob; that belongs in a
  later fork-aware validation state.
- Blob hash version bytes are not checked in this release. EIP-4844 execution
  validation requires KZG versioned hashes; that policy belongs in a later
  validation state with fork context.
- KZG commitments, proofs, sidecars, and data availability are not represented
  or validated in this release.
- `max_fee_per_blob_gas` is decoded as a canonical unsigned U256 word but is
  not checked against the current blob base fee.
- Access lists use the same borrowed EIP-2930 access-list model and eager
  validation behavior introduced in v0.13.0.
- Signature `r` and `s` are decoded as canonical unsigned U256 words but are not
  checked for secp256k1 scalar validity or low-s policy.
- The borrowed input field is checked against the active allocation limit even
  though the decoder does not allocate, so callers have one policy knob for
  exposed calldata size.

## Specification Evidence

- EIP-4844 defines transaction type `0x03` and the fourteen-field blob
  transaction payload shape.
- EIP-4844 states that `to` must always be a 20-byte address.
- EIP-4844 defines `max_fee_per_blob_gas` as `uint256` and
  `blob_versioned_hashes` as a list of versioned hash outputs.
- Official EIP sources were checked on 2026-07-01 while preparing this release.

## Release Gate

- Pentest passed with informational notes only.
- Permanent report path: `security/pentest/v0.15.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-protocol -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth --all-targets --all-features -- -D warnings
cargo check --manifest-path fuzz/Cargo.toml
scripts/release_0_15_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
