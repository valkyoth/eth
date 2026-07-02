# eth 0.24.0 Release Notes

Status: tagged and released

## Summary

`0.24.0` adds unvalidated EIP-7702 set-code transaction decoding and
no-allocation canonical encoding. The protocol crate now admits type byte
`0x04`, checks the thirteen transaction fields, validates the required
20-byte destination shape, and exposes bounded authorization-list iteration for
tuples shaped `[chain_id, address, nonce, y_parity, r, s]`.

This is still syntactic transaction handling. Signature validation, set-code
authorization validation, fork activation, fee/account-state checks, and
execution validity remain deferred to later verification layers.

## Added

- Added `decode_set_code_transaction`.
- Added `UnvalidatedSetCodeTransaction`.
- Added `SetCodeAuthorizationList`, `SetCodeAuthorizationItems`,
  `SetCodeAuthorization`, `SetCodeAuthorizationChainId`, and
  `SetCodeAuthorizationField`.
- Added stable set-code decode errors and categories.
- Added `encoded_set_code_transaction_len` and
  `encode_set_code_transaction`.
- Added the unified `UnvalidatedTransaction::SetCode` variant.
- Added `require_set_code_replay_domain`.
- Added fail-closed decoded signature validation handling for set-code
  transactions through `TransactionSignatureValidationError::UnsupportedTransactionType`.
- Extended transaction-envelope fuzz coverage and seed corpus entries for
  set-code transactions.
- Added `scripts/release_0_24_gate.sh`.

## Security Notes

- The decoder enforces EIP-7702 transaction type byte `0x04`.
- The transaction payload must contain exactly thirteen RLP fields.
- The destination field must be a 20-byte address; contract-creation set-code
  transactions are rejected at decode time.
- Authorization tuples must contain exactly chain ID, address, nonce, y parity,
  `r`, and `s`.
- Authorization addresses must be 20-byte scalars and y parity must be `0` or
  `1`.
- Authorization chain ID is modeled separately because EIP-7702 authorization
  tuples may use chain ID zero as a universal authorization domain.
- The EIP-7702 authorization signing magic byte is not exported in this
  release because no reviewed authorization signing-hash helper exists until
  `v0.24.1`.
- Empty authorization-list rejection is intentionally deferred to later
  validation, matching the existing syntactic decode boundary for other
  transaction families. The explicit follow-up milestones are `v0.24.1` for
  set-code signing and authorization validation, and `v0.24.2` for the
  transaction-validity gate.
- Set-code transaction signing-hash construction and authorization-signature
  validation are not part of this release. The decoded signature validation
  helper rejects set-code transactions with an explicit unsupported-type error.

## Versioning

- `eth-valkyoth-protocol` publishes as `0.20.0`.
- `eth-valkyoth-verify` publishes as `0.13.0`.
- The facade crate publishes as `eth` `0.24.0`.
- Unchanged support crates are not republished.

## Release Gate

- External pentest passed and the permanent report is available under
  `security/pentest/v0.24.0.md`.
- Final GitHub checks passed before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_24_gate.sh
```
