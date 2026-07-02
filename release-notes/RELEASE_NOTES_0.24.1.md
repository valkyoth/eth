# eth 0.24.1 Release Notes

Status: ready for external pentest

## Summary

`0.24.1` adds the cryptographic EIP-7702 set-code signing layer that was
intentionally left out of the `0.24.0` syntactic decoder.

The protocol crate now exposes no-allocation signing-preimage encoders for the
outer set-code transaction domain and for authorization-list tuples. The verify
crate exposes distinct hash newtypes and validation APIs for the transaction
sender signature and the authorization signer signature, so the `0x04`
transaction domain cannot be silently substituted for the `0x05` authorization
domain.

This is still not full EIP-7702 execution validity. Empty authorization-list
rejection, authorization chain policy, nonce/account-state checks, delegation
indicator checks, fee checks, and fork activation remain scheduled for
`v0.24.2`.

## Added

- Added `SET_CODE_AUTHORIZATION_MAGIC`.
- Added `encoded_set_code_signing_preimage_len` and
  `encode_set_code_signing_preimage`.
- Added `encoded_set_code_authorization_signing_preimage_len` and
  `encode_set_code_authorization_signing_preimage`.
- Added `set_code_transaction_signing_hash`.
- Added `SetCodeAuthorizationSigningHash`.
- Added `set_code_authorization_signing_hash`.
- Added `validate_set_code_transaction_signature`.
- Added `validate_set_code_authorization_signature`.
- Added `ValidatedSetCodeAuthorization` and stable authorization validation
  error/category types.
- Extended the unified decoded transaction signature validator so
  `UnvalidatedTransaction::SetCode` uses the EIP-7702 transaction signing
  domain instead of returning `UnsupportedTransactionType`.
- Added fuzz coverage for set-code authorization signing-hash construction and
  tuple signature validation with input-selected scratch-buffer lengths.

## Security Notes

- EIP-7702 has two separate signature domains:
  `keccak256(0x04 || rlp(unsigned_set_code_transaction_payload))` for the
  transaction sender, and `keccak256(0x05 || rlp([chain_id, address, nonce]))`
  for each authorization tuple.
- Authorization validation recovers the tuple authority and applies low-s,
  scalar, and y-parity policy through the same secp256k1 recovery boundary used
  for transaction signatures.
- Authorization chain-ID policy is not enforced in this release. Universal
  authorization chain ID `0` versus expected chain binding is part of the
  `v0.24.2` transaction-validity gate.
- Callers still provide concrete Keccak-256 implementations. Hashers used on
  key-adjacent recovery paths should clear sponge state on drop.

## Versioning

- `eth-valkyoth-protocol` publishes as `0.21.0`.
- `eth-valkyoth-verify` publishes as `0.14.0`.
- The facade crate publishes as `eth` `0.24.1`.
- Unchanged support crates are not republished.

## Release Gate

- External pentest must pass before tagging.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_24_gate.sh
```
