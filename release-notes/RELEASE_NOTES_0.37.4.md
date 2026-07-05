# eth 0.37.4 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.37.4` closes the constant-time and reference dependency policy slice from
the core dependency independence audit.

## Added

- `docs/constant-time-reference-policy.md` records the reviewed `subtle`
  exception and the quarantine rules for reference-only dependencies.
- `scripts/check_runtime_dependency_policy.py` enforces that reference crates
  and optional backend/parser crates do not enter the default runtime graph.
- `scripts/release_0_37_4_gate.sh` captures default and all-feature cargo-tree
  evidence for the release.

## Changed

- `eth` publishes as `0.37.4` with updated facade documentation.
- `release-crates.toml` marks support crates unchanged and republishes only the
  facade package.
- `docs/core-independence-audit.md`, `docs/CRATE_VERSION_MATRIX.md`, and the
  README files now name `v0.37.4` as the dependency-policy evidence release.

## Security Notes

- `subtle 2.6.1` remains the only reviewed default runtime third-party
  exception in the facade graph. It is accepted only for constant-time
  fixed-width equality composition.
- `alloy-rlp` remains reference-only for dev/fuzz differential checks and must
  not appear in runtime paths.
- Codec fixture `serde_json` use remains dev-only. The separate optional
  EIP-712 JSON parser policy remains scheduled for `v0.37.5`.

## Verification

- `scripts/check_runtime_dependency_policy.py`
- `cargo tree -p eth --no-default-features -e normal`
- `cargo tree -p eth -e features --all-features`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo deny check`
- `cargo audit`
- `scripts/release_0_37_4_gate.sh`

## Pentest

- Run pentest on the implementation commit before tagging.
- Permanent report path after clean retest:
  `security/pentest/v0.37.4.md`.

## Versioning

- `eth` publishes as `0.37.4`.
- Support crates are unchanged and are not republished.
