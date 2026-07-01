# eth 0.16.1 Release Notes

Status: implementation ready for pentest

## Summary

`0.16.1` evaluates the future RLP derive surface without exposing public
`RlpEncode` or `RlpDecode` macros yet.

The release records the derive API decision, adds a private derive-crate
prototype for planning field order and attributes, and documents why public RLP
derives remain deferred until codec traits and transaction validation typestates
are stable.

## Added

- Added `docs/rlp-derive-design.md`.
- Added private `eth-valkyoth-derive` tests for the RLP derive plan:
  declaration-order field handling, explicit skip/default reason handling, and
  rejection of generics, enums, unions, and ambiguous field attributes.
- Added pentest remediation coverage ensuring skip/default reasons are retained
  in the private field plan and duplicate `#[eth_rlp(...)]` attributes are
  rejected.

## Changed

- Bumped `eth-valkyoth-derive` to `0.16.1`.
- Bumped `eth-valkyoth-sanitization` to `0.7.1` so the optional derive feature
  tracks the new derive package.
- Bumped the facade crate to `0.16.1` for packaged documentation and release
  metadata.

## Security Notes

- No public RLP derive macro is exposed in this release.
- The design requires future decode derives to take explicit `DecodeLimits` and
  use the same bounded codec and primitive helpers as hand-written paths.
- The private prototype now carries skip/default reasons forward in its plan so
  future code generation cannot silently drop the audit trail.
- Duplicate `eth_rlp` field attributes are rejected instead of merged.
- Transaction derives remain deferred so generated code cannot bypass fork
  validation or sender-recovery typestates.

## Release Gate

- Pentest is required before the release report commit.
- Permanent report path after pentest: `security/pentest/v0.16.1.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-derive -p eth-valkyoth-codec -p eth-valkyoth-protocol
scripts/release_0_16_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
