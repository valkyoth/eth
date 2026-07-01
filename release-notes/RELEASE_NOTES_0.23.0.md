# eth 0.23.0 Release Notes

Status: pentest passed; waiting for final GitHub checks

## Summary

`0.23.0` adds decoded transaction signature validation helpers. The new
verify-layer APIs combine replay-domain checks, signing-hash construction,
low-s/y-parity policy, secp256k1 sender recovery, and optional expected-sender
comparison for legacy EIP-155, EIP-2930, EIP-1559, and EIP-4844 transactions.

This release does not promote protocol typestate tokens yet. Public proof
constructors remain deferred until proofs can be bound to a transaction
identity.

## Added

- Added `ValidatedTransactionSignature`.
- Added `TransactionSignatureValidationError` and category codes.
- Added `validate_transaction_signature` for the unified admitted transaction
  enum.
- Added type-specific validation helpers for:
  - legacy EIP-155 transactions;
  - EIP-2930 access-list transactions;
  - EIP-1559 dynamic-fee transactions;
  - EIP-4844 blob transactions.
- Added real-Keccak tests that sign and recover all supported transaction
  families through the public validation helpers.
- Added external mainnet raw-transaction KATs sourced from
  `ethereum.publicnode.com` for EIP-2930, EIP-1559, and EIP-4844 sender
  recovery.
- Added wrong-chain, wrong-sender, high-s, malformed-scalar, and signing-hash
  construction failure tests.
- Added `docs/transaction-signature-validation.md`.
- Added `scripts/release_0_23_gate.sh`.

## Security Notes

- The validation helpers require an expected chain ID before sender recovery is
  accepted.
- Legacy validation derives y parity from EIP-155 `v`; pre-EIP-155 legacy
  transactions remain rejected as missing a replay domain.
- The helpers reject high-s signatures and malformed secp256k1 scalars through
  the existing sender recovery path.
- `ValidatedTransactionSignature` can only be constructed inside
  `eth-valkyoth-verify`; downstream callers must pass through validation
  helpers to obtain one.
- The caller may pass an expected sender; mismatches return
  `TransactionSignatureValidationError::WrongSender`.
- The helpers still do not validate fork activation, fees, account state,
  blob/KZG commitments, or EIP-4844 blob-hash version policy.

## Versioning

- `eth-valkyoth-verify` publishes as `0.12.0`.
- The facade crate publishes as `eth` `0.23.0`.
- Unchanged support crates are not republished.

## Release Gate

- External pentest passed and the permanent report is available under
  `security/pentest/v0.23.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_23_gate.sh
```
