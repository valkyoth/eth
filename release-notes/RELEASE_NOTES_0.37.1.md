# eth 0.37.1 Release Notes

Status: pentest passed; waiting for final GitHub checks before tagging.

`0.37.1` adds the REVM and Ethereum upstream advisory checker required before
execution adapter work continues. REVM remains non-admitted.

## Added

- `scripts/check_ethereum_upstream.py` compares reviewed REVM registry data and
  pinned official Ethereum source revisions against upstream metadata.
- `docs/ethereum-upstream-check.md` documents the safe-fetch policy, command
  shape, drift semantics, and maintenance-release decision points.
- `scripts/release_0_37_1_gate.sh` validates the v0.37.1 release slice.

## Security Notes

- The checker fetches metadata only. It does not execute fetched content.
- crates.io requests use an explicit project user agent.
- official Ethereum source checks use `git ls-remote` against allowlisted
  `https://github.com/ethereum/...` repositories only.
- Upstream drift is advisory and does not fail the release merely because a
  remote moved. A moved source must be reviewed in a maintenance release before
  implementation work trusts new fork or fixture behavior.
- On 2026-07-05, `revm` and `revm-primitives` still report latest `41.0.0`
  with Rust `1.91.0`, while the newest Rust `1.90.0`-compatible reviewed lines
  remain `revm 36.0.0` and `revm-primitives 22.1.0`.
- On 2026-07-05, the pinned EIPs repository revision was behind remote `HEAD`.
  No implementation source was updated in this release; the drift is recorded
  as maintenance input.

## Fixed During Pentest

- Empty `git ls-remote` output is now reported as a controlled checker error
  instead of an uncaught `IndexError`.
- crates.io versions with unknown `rust_version` metadata are no longer treated
  as evidence of Rust `1.90.0` compatibility.
- Added focused regression tests for both checker hardening cases.

## Pentest

- External pentest passed after remediation of the upstream-checker robustness
  findings.
- Permanent report: `security/pentest/v0.37.1.md`.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Versioning

- The facade crate publishes as `eth` `0.37.1` for updated package
  documentation.
- Support crates are unchanged and are not republished.
