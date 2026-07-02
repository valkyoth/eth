# eth 0.24.2 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.24.2` adds the EIP-7702 set-code transaction validity gate that was
scheduled after the syntactic decoder and signing validation releases.

The protocol crate now exposes a non-cryptographic context validator that keeps
decode permissive but rejects set-code transactions that are not valid for a
caller-supplied execution context. The gate checks Prague/Pectra fork
activation, outer transaction chain binding, fee order, optional caller-computed
minimum gas, non-empty authorization lists, authorization chain and nonce
policy, recovered authority availability, authority nonce equality, and
authority code classification.

The gate intentionally does not recover signatures itself. Callers should obtain
authorization authorities through the `eth-valkyoth-verify` authorization
signature API and then pass those authorities into the protocol validity gate.

Official EIP-7702 source was refreshed on 2026-07-02 before implementation.

## Added

- Added `SetCodeTransactionValidationContext`.
- Added `validate_set_code_transaction_context`.
- Added `ValidSetCodeTransaction`.
- Added `SetCodeAuthorizationAuthority` and `SetCodeAuthorizationAuthorityView`.
- Added `SetCodeAuthorityAccount`, `SetCodeAuthorityCode`, and
  `SetCodeAuthorityStateView`.
- Added `SetCodeTransactionValidityError` and stable validity error categories.
- Added `EIP_7702_DELEGATION_INDICATOR_PREFIX`.
- Added regression tests for empty authorization lists, wrong authorization
  chain, max authorization nonce, inactive fork, pre-Prague fork context, fee
  order, gas policy integration, nonce mismatch, and invalid authority code.

## Security Notes

- The syntactic decoder still accepts empty authorization lists so callers can
  inspect malformed or invalid transactions. The validity gate rejects them.
- Authorization chain ID `0` remains universal. Nonzero authorization chain IDs
  must match the outer transaction chain ID.
- Authorization nonce `u64::MAX` is rejected before authority-state checks
  because EIP-7702 increments authority nonces after a valid tuple is applied.
- Authority account code must be empty or already classified by the caller as
  an EIP-7702 delegation indicator. Non-delegation code is rejected.
- The optional `minimum_gas_limit` field lets callers bind their own intrinsic
  gas calculation to this validation boundary without introducing a node or RPC
  dependency.

## Versioning

- `eth-valkyoth-protocol` publishes as `0.22.0`.
- `eth-valkyoth-verify` publishes as `0.14.1` because its protocol dependency
  must align with `eth-valkyoth-protocol 0.22.0`.
- The facade crate publishes as `eth` `0.24.2`.
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
