# eth 0.4.0 Release Notes

Status: in development

## Summary

`0.4.0` establishes the stable error model for protocol operations and starts
independent crate versioning for crates.io releases.

## Planned

- Add non-panicking error categories for codec, protocol, verification, fork,
  feature, and resource-exhaustion boundaries.
- Keep error payloads free of secret-bearing data.
- Add tests for error stability and formatting.
- Replace lockstep support-crate publishing with per-crate release planning.
- Publish only crates that changed, with dependency-only forced bumps using a
  patch version on the existing crate line.

## Verification

```bash
scripts/checks.sh
scripts/release_0_4_gate.sh
scripts/release_crates.py --check
cargo deny check
cargo audit
```
