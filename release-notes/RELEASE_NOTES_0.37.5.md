# eth 0.37.5 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.37.5` closes the optional parser and sanitization boundary review from the
core dependency independence audit.

## Added

- `docs/optional-parser-sanitization-policy.md` records when `serde`,
  `serde_json`, `eth-valkyoth-sanitization`, and `sanitization` enter the
  dependency graph.
- `scripts/check_optional_boundary_policy.py` enforces facade feature wiring
  and graph separation for JSON parser and sanitization dependencies.
- `scripts/test-optional-boundary-policy.py` locks the checker constants used
  by the gate.
- `scripts/release_0_37_5_gate.sh` captures default, `eip712-json`,
  `sanitization`, and all-feature cargo-tree evidence.

## Changed

- `eth` publishes as `0.37.5` with updated facade documentation.
- `release-crates.toml` marks support crates unchanged and republishes only the
  facade package.
- `docs/core-independence-audit.md`, `docs/CRATE_VERSION_MATRIX.md`, and the
  README files now name `v0.37.5` as the optional-boundary evidence release.

## Security Notes

- The default facade graph must not include `serde`, `serde_json`,
  `eth-valkyoth-sanitization`, or `sanitization`.
- `eip712-json` remains an explicit `std` parser boundary and admits
  `serde`/`serde_json` only through `eth-valkyoth-verify/json`.
- `sanitization` remains an explicit secret-clearing bridge and admits the
  external `sanitization` crate only through `eth-valkyoth-sanitization`.

## Verification

- `scripts/check_optional_boundary_policy.py`
- `python3 scripts/test-optional-boundary-policy.py`
- `cargo tree -p eth -e normal`
- `cargo tree -p eth --no-default-features --features eip712-json -e normal`
- `cargo tree -p eth --no-default-features --features sanitization -e normal`
- `cargo tree -p eth -e features --all-features`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo deny check`
- `cargo audit`
- `scripts/release_0_37_5_gate.sh`

## Pentest

- Run pentest on the implementation commit before tagging.
- Permanent report path after clean retest:
  `security/pentest/v0.37.5.md`.

## Versioning

- `eth` publishes as `0.37.5`.
- Support crates are unchanged and are not republished.
