# eth 0.21.0 Release Notes

Status: implementation ready for pentest

## Summary

`0.21.0` adds EIP-712 domain-safety helpers. Structured-data signing paths can
now require `chainId` and `verifyingContract`, check both against the expected
execution context, build the EIP-191/EIP-712 signing digest, and recover a
sender only after the domain gate passes.

This release does not implement a full EIP-712 typed-data encoder. That work is
now scheduled for `v0.26.0 - EIP-712 Typed-Data Encoder`. Until then, callers
must still compute `domainSeparator` and `hashStruct(message)` with a
conformant encoder before using these helpers.

## Added

- Added `Eip712Domain` in `eth-valkyoth-verify`.
- Added `require_eip712_domain` for expected `chainId` and
  `verifyingContract` checks.
- Added `EIP712_SIGNING_PREFIX` and `eip712_signing_digest` for
  `keccak256("\x19\x01" || domainSeparator || hashStruct(message))`.
- Added `recover_eip712_sender`, which checks the EIP-712 domain before
  digest-level sender recovery.
- Added stable verification errors for missing EIP-712 `chainId`, missing
  `verifyingContract`, and wrong verifying contract.
- Added unit tests for missing fields, wrong fields, digest preimage
  composition, and domain-before-signature ordering.
- Added an independent Keccak/k256 EIP-712 sender-recovery vector for the full
  domain-check, digest-construction, and sender-recovery pipeline.
- Added `scripts/release_0_21_gate.sh`.

## Changed

- `eth-valkyoth-verify` publishes as `0.10.0` under the independent
  support-crate versioning policy.
- The facade crate publishes as `eth` `0.21.0` and re-exports the EIP-712
  domain-safety APIs through `eth::verify`.
- `v0.20.0` is now marked tagged in the release plan.

## Security Notes

- EIP-712 domains are accepted by the safety gate only when both replay-critical
  fields are present and match the expected context.
- `recover_eip712_sender` checks the structured-data domain before invoking
  signature recovery, so malformed or wrong-domain inputs fail before raw digest
  recovery is trusted.
- The digest helper uses the EIP-191 `0x1901` prefix required by EIP-712, but
  it does not validate that the supplied domain separator or message hash came
  from a conformant typed-data encoder.
- `recover_eip712_sender` documents locally that `domain_separator` is not
  proven to be derived from the checked domain; callers must compute both from
  the same EIP-712 domain model until the full typed-data encoder lands.
- Concrete Keccak-256 backend admission rules from `docs/keccak-boundary.md`
  still apply.

## Release Gate

- External pentest must pass before a permanent report is added under
  `security/pentest/v0.21.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_21_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
