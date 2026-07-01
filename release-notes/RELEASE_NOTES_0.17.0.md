# eth 0.17.0 Release Notes

Status: pentest passed; final GitHub checks pending before tag

## Summary

`0.17.0` adds explicit chain and fork specification APIs to the protocol crate.
Consensus-sensitive operations can now require a caller-reviewed `ChainSpec`,
selected `ForkSpec`, hardfork identity, block number, and timestamp before
later validation layers treat fork context as available.

The release does not hardcode mainnet validation rules. Callers provide their
own chain spec entries from reviewed upstream data.

## Added

- Added `Hardfork` identities for execution-layer fork selection.
- Added `ChainSpec` as a caller-reviewed list of admitted fork specs.
- Added `ChainSpec::try_new` for dynamic or generated chain spec entries that
  need eager runtime validation.
- Extended `ForkSpec` with an explicit hardfork identity.
- Added `ForkSpec::is_active_at`.
- Added `ChainSpec::fork_spec`, `ChainSpec::validation_context`, and
  `ChainSpec::active_fork`.
- Added `ForkError::ChainMismatch`, `ForkError::DuplicateFork`, and
  `ForkError::NonMonotonicForkOrder`.
- Added tests for block-only activation, block/timestamp activation,
  unsupported forks, chain mismatch, duplicate forks, non-monotonic fork order,
  non-monotonic activation thresholds, and active-fork boundary selection.
- Added `scripts/release_0_17_gate.sh`.

## Security Notes

- Fork context is explicit and caller-provided; this crate still does not claim
  transaction, header, receipt, account-state, gas, fee, or sender validity.
- Missing fork entries produce `ForkError::Unsupported`.
- Fork entries with the wrong chain ID produce `ForkError::ChainMismatch`.
- Duplicate fork entries produce `ForkError::DuplicateFork`.
- Non-monotonic hardfork or activation ordering produces
  `ForkError::NonMonotonicForkOrder`.
- `ChainSpec::new` remains the `const` constructor for hand-audited static
  tables; use `ChainSpec::try_new` for generated config, external chain-spec
  data, or merged fork lists.
- Timestamp-based forks require both block and timestamp thresholds.
- Chain specs are intentionally chain-agnostic: a timestamp-based fork may be
  followed by a block-only fork on non-mainnet chains when block thresholds
  remain monotonic.

## Release Gate

- Pentest passed and permanent report is
  `security/pentest/v0.17.0.md`.
- Final GitHub checks must pass on the release report commit before tagging.

## Verification

Expected local release checks:

```bash
cargo test -p eth-valkyoth-protocol -p eth --all-features
scripts/release_0_17_gate.sh
scripts/validate-release-metadata.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
