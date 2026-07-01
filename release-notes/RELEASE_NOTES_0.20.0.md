# eth 0.20.0 Release Notes

Status: implementation ready; pentest pending

## Summary

`0.20.0` adds digest-level Ethereum sender recovery. Callers can recover an
`Address` from an already constructed 32-byte Ethereum signing digest,
`r/s/y_parity` signature parts, and a caller-provided Keccak-256 hasher.

This release does not construct transaction signing hashes from decoded
transactions. Replay-domain checks, transaction preimage construction,
fork/fee/account validation, and proof binding remain separate steps.

## Added

- Added `EthereumSignature` in `eth-valkyoth-verify`.
- Added `recover_sender_from_digest` for secp256k1 ECDSA public-key recovery and
  Ethereum address derivation.
- Added low-s rejection for EIP-2 signature malleability policy.
- Added explicit y-parity recovery policy: only `0` and `1` are accepted.
- Added digest-level valid and invalid sender-recovery tests.
- Added `docs/dependency-admission-k256.md`.
- Added `scripts/release_0_20_gate.sh`.

## Changed

- `eth-valkyoth-verify` publishes as `0.9.0` under the independent
  support-crate versioning policy.
- The facade crate publishes as `eth` `0.20.0` and re-exports the sender
  recovery APIs through `eth::verify`.
- `eth-valkyoth-verify` now depends on `eth-valkyoth-hash` and `k256`.

## Security Notes

- `k256` `0.13.4` is admitted with default features disabled and `ecdsa`
  enabled. `0.14.0-rc.15` was not selected because it is a release candidate.
- Sender recovery rejects malformed scalar values, high-s signatures, and
  non-Ethereum recovery IDs.
- Address derivation hashes the recovered uncompressed public key payload
  through the caller-provided `Keccak256` trait boundary. No concrete Keccak
  backend is admitted in this release.
- The hasher used for sender recovery should have an explicit state-clearing
  policy. The optional sanitization bridge remains the preferred place to
  enforce `SecureSanitize` for concrete stateful hashers.

## Release Gate

- Pentest must be completed before tagging.
- Permanent pentest report will be `security/pentest/v0.20.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-verify -p eth --all-features
cargo clippy -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_20_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
