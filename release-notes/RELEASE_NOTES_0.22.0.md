# eth 0.22.0 Release Notes

Status: tagged

## Summary

`0.22.0` adds canonical Ethereum transaction signing-preimage construction for
legacy EIP-155, EIP-2930, EIP-1559, and EIP-4844 transactions. It also adds
verify-layer helpers that hash those preimages through the caller-provided
Keccak-256 boundary and return a `TransactionSigningHash` domain newtype.

This is still not full transaction signature validation. That remains scheduled
for `0.23.0`.

## Added

- Added no-allocation protocol encoders for:
  - legacy EIP-155 signing preimages;
  - EIP-2930 access-list signing preimages;
  - EIP-1559 dynamic-fee signing preimages;
  - EIP-4844 blob signing preimages.
- Added signing-preimage length helpers for caller-sized scratch buffers.
- Added `TransactionSigningHash` in `eth-valkyoth-verify`.
- Added verify helpers for legacy, access-list, dynamic-fee, and blob
  transaction signing hashes.
- Added `TransactionSigningHashError` so missing EIP-155 replay domains and
  encode failures remain distinguishable from signature failures.
- Added `docs/transaction-signing-hashes.md`.
- Added `scripts/release_0_22_gate.sh`.
- Added real `sha3::Keccak256` signing-hash tests for legacy EIP-155,
  EIP-2930, EIP-1559, and EIP-4844, including blob hash API coverage.

## Security Notes

- Legacy signing hashes require an EIP-155 replay domain recovered from `v`.
  Pre-EIP-155 legacy transactions return
  `TransactionSigningHashError::MissingReplayDomain`.
- Typed signing preimages exclude `y_parity`, `r`, and `s`.
- All signing hashes use the caller-provided Keccak-256 trait boundary. The
  crate still does not ship a default hash backend.
- Full sender validation, wrong-sender checks, and decoded transaction
  state-promotion helpers are deferred to `0.23.0`.
- Transaction signing-hash helpers write canonical preimages into caller-owned
  scratch buffers. Callers that reuse scratch for in-flight transactions should
  clear it after hashing.

## Versioning

- `eth-valkyoth-protocol` publishes as `0.19.0`.
- `eth-valkyoth-verify` publishes as `0.11.0`.
- The facade crate publishes as `eth` `0.22.0`.
- Unchanged support crates are not republished.

## Release Gate

- Pentest passed and permanent report is `security/pentest/v0.22.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_22_gate.sh
```
