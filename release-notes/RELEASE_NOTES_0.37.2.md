# eth 0.37.2 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.37.2` adds the core dependency independence audit required before execution
work continues. The release does not admit REVM, RPC, networking, local signing,
or new execution dependencies.

## Added

- `docs/core-independence-audit.md` inventories third-party dependencies that
  can influence core Ethereum behavior.
- The audit classifies dependencies as first-party, optional backend,
  reference-only, compile-time only, reviewed exception, or temporary debt.
- The release plan now includes `v0.37.4` and `v0.37.5` so constant-time
  helpers, reference-only crates, optional parser support, and optional
  sanitization bridges have explicit follow-up releases.
- `scripts/release_0_37_2_gate.sh` validates the v0.37.2 release slice.

## Security Notes

- `k256 0.13.4` remains a default runtime dependency through
  `eth-valkyoth-verify` sender recovery. This is classified as temporary debt
  and assigned to `v0.37.3`.
- `subtle 2.6.1` remains a default runtime dependency through primitive
  constant-time equality. This is classified as a reviewed-exception candidate
  and assigned to `v0.37.4` for a narrower policy decision.
- `tiny-keccak`, `serde`, `serde_json`, and `sanitization` remain opt-in
  feature paths, not default facade dependencies.
- `alloy-rlp` and `sha3` remain dev/fuzz/reference dependencies only.
- REVM remains out of the dependency graph.
- Current-version spot checks were run for the core-impacting external crates.
  The only non-current stable item called out by this release is dev-only
  `sha3 0.10.9`; `v0.37.3` reviews whether it should be replaced by the
  project hash boundary or kept as an independent dev-only oracle.

## Verification

- `cargo tree -p eth -e features --no-default-features`
- `cargo tree -p eth -e features --all-features`
- `cargo tree -e features --workspace`
- `scripts/release_0_37_2_gate.sh`

## Fixed During Pentest

- Removed a duplicate `test-ethereum-upstream.py` invocation from the v0.37.2
  release gate. `scripts/checks.sh` already runs that test.
- Normalized the dependency-audit classification tables to the documented
  taxonomy so future linting can rely on closed labels.
- Updated the upstream-checker user-agent from `0.37.1` to `0.37.2` and added
  release-metadata validation for that string.

## Pentest

- External pentest findings have been remediated; clean retest is required
  before tagging.
- Permanent report path after clean retest:
  `security/pentest/v0.37.2.md`.

## Versioning

- The facade crate publishes as `eth` `0.37.2` for updated package
  documentation.
- Support crates are unchanged and are not republished.
