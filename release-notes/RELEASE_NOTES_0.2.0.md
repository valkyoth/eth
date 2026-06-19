# eth 0.2.0 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.2.0` prepares the workspace for crates.io publication by keeping the facade
crate as `eth` and moving all support crates into the `eth-valkyoth-*`
namespace.

## Included

- Rename support crates to `eth-valkyoth-*`.
- Keep `eth` as the public facade crate.
- Add `scripts/release_crates.py` for crates.io publish order.
- Add a release-gate check that keeps the publish script synchronized with
  workspace metadata.
- Add negative tests for release-readiness refusal paths.

## Verification

```bash
scripts/checks.sh
scripts/release_0_2_gate.sh
scripts/release_crates.py --check
scripts/test-release-readiness.sh
```
