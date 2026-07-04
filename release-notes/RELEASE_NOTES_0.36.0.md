# eth 0.36.0 Release Notes

Status: pentest passed; waiting for final GitHub checks before tagging.

`0.36.0` adds the first differential test harness. The release compares
`eth-valkyoth-codec` structural RLP behavior against the independent
`alloy-rlp` implementation through a dev-only dependency.

## Added

- `scripts/run_differential_tests.py` validates and runs the differential
  harness.
- `eth-valkyoth-codec` now has a `differential_rlp_reference` integration test
  comparing curated valid/invalid RLP cases against `alloy-rlp`.
- `docs/differential-test-harness.md` documents the reference path, command,
  scope, and mismatch reporting.
- `docs/differential-test-report.md` records the v0.36.0 claimed pass set.
- `scripts/release_0_36_gate.sh` includes the differential runner.

## Security Notes

- `alloy-rlp` is dev-only for the codec crate and is not pulled into the
  production `no_std` facade.
- Checked current registry versions on 2026-07-04: `alloy-rlp` `0.3.16`,
  transitive `bytes` `1.12.0`, `serde` `1.0.228`, and `serde_json` `1.0.150`
  are current.
- The differential claim is structural RLP coverage. Ethereum integer-domain
  canonicality remains covered by local codec, primitive bridge, and fuzz tests.
- The test accumulates all mismatches before failing so a pentest or CI failure
  can report every observed divergence in one run.

## Fixed During Pentest

- `scripts/run_differential_tests.py --check` now compiles the actual
  differential integration test with `--no-run` instead of printing a constant
  success message.
- Added `fuzz/fuzz_targets/rlp_differential.rs` and committed seed cases so
  arbitrary byte inputs can be compared against `alloy-rlp`, reducing reliance
  on the hand-curated integration-test corpus.

## Pentest

- External pentest passed after remediation of the differential check-gate and
  randomized differential fuzz-coverage findings.
- Permanent report: `security/pentest/v0.36.0.md`.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Versioning

- `eth-valkyoth-codec` publishes as `0.19.0` for the new differential test
  package surface.
- Downstream crates with only refreshed dependency requirements use patch
  bumps.
- The facade crate publishes as `eth` `0.36.0`.
