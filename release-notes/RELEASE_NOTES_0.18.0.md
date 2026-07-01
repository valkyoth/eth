# eth 0.18.0 Release Notes

Status: implementation ready for pentest

## Summary

`0.18.0` makes transaction validation state transitions proof-gated. Decoded
transaction tokens can no longer be promoted to canonical, fork-valid, or
sender-recovered states through infallible public methods.

This release still does not implement replay-domain checks, signature recovery,
or full transaction validity. Public proof constructors remain intentionally
deferred until the validators that can create those proofs exist.

## Added

- Added `CanonicalValidationProof`, `ForkValidationProof`, and
  `SenderRecoveryProof` proof tokens.
- Added `Transaction<Decoded>::try_into_canonical`.
- Added `Transaction<Canonical>::try_into_fork_validated`.
- Added `Transaction<ForkValidated>::try_into_sender_recovered`.
- Moved transaction typestate code into a focused module with dedicated tests.
- Added tests proving failed validation does not consume or mutate the previous
  transaction state token.
- Added `scripts/release_0_18_gate.sh`.

## Changed

- Removed infallible transaction state promotion methods from the public API.
  Promotion now requires a validation proof result and returns
  `Result<Transaction<_>, ProtocolError>`.

## Security Notes

- The proof token fields are private. External callers cannot fabricate a
  canonical, fork-valid, or sender-recovered transition proof in this release.
- Failed proof checks borrow the source state token and return an error without
  consuming or mutating the previous typestate.
- Replay-domain validation, sender recovery, and concrete signature checks stay
  deferred to later releases.

## Release Gate

- Pentest is required before the release report commit.
- Permanent report path after pentest: `security/pentest/v0.18.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-protocol -p eth --all-features
scripts/release_0_18_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
