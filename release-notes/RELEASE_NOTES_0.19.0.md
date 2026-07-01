# eth 0.19.0 Release Notes

Status: implementation ready for pentest

## Summary

`0.19.0` adds replay-domain validation for decoded transaction field models.
Callers can now require a transaction's signed chain domain to match an
expected `ChainId` before accepting future sender-recovery results.

This release still does not implement signature recovery, low-s checks, fork
validity, fee validity, account-state validation, gas accounting, blob/KZG
validation, or full transaction validity.

## Added

- Added `require_legacy_replay_domain` for EIP-155 legacy transaction chain
  binding.
- Added `require_access_list_replay_domain` for EIP-2930 transaction chain
  binding.
- Added `require_dynamic_fee_replay_domain` for EIP-1559 transaction chain
  binding.
- Added `require_blob_replay_domain` for EIP-4844 transaction chain binding.
- Added `require_transaction_replay_domain` for the unified
  `UnvalidatedTransaction` domain.
- Added `VerifyError::MissingReplayDomain` for legacy transactions without an
  EIP-155 chain binding.
- Added `scripts/release_0_19_gate.sh`.

## Changed

- `eth-valkyoth-verify` now depends on `eth-valkyoth-protocol` so replay-domain
  checks can accept decoded transaction field models directly.
- `eth-valkyoth-verify` publishes as `0.8.0` under the independent support-crate
  versioning policy while the facade publishes as `eth` `0.19.0`.
- The facade crate re-exports the new verification APIs through `eth::verify`
  and the new stable error through `eth::error`.

## Security Notes

- Pre-EIP-155 legacy transactions fail replay-domain validation with
  `VerifyError::MissingReplayDomain`.
- Wrong-chain legacy, EIP-2930, EIP-1559, and EIP-4844 transactions fail with
  `VerifyError::WrongChain`.
- Replay-domain validation is intentionally separate from sender recovery. A
  matching chain ID does not imply a valid signature or valid transaction.

## Release Gate

- Pentest is required before the release report commit.
- Permanent report path after pentest: `security/pentest/v0.19.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-verify -p eth-valkyoth-protocol -p eth --all-features
scripts/release_0_19_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
