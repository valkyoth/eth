# eth 0.9.1 Release Notes

Status: release candidate ready; final GitHub checks required before tag

## Summary

`0.9.1` removes duplicated Ethereum RLP integer canonicality logic between the
codec and primitive crates.

The codec crate is now the single implementation point for canonical integer
payload validation and conversion. Primitive constructors keep their existing
domain error type, but delegate canonical payload parsing to codec helpers.

## Included So Far

- Added public payload-level codec helpers:
  `validate_rlp_integer_payload`, `rlp_integer_payload_to_u64`,
  `rlp_integer_payload_to_u128`, and `rlp_integer_payload_to_u256_bytes`.
- Updated `RlpInteger` conversion methods to use the same payload helpers.
- Made `eth-valkyoth-primitives` depend on `eth-valkyoth-codec` and removed its
  duplicated integer radix, width, leading-zero, and right-alignment logic.
- Added primitive tests that compare constructor behavior against codec helper
  behavior for accepted and rejected integer payloads.
- Addressed v0.9.1 pentest findings by updating fuzz dependencies to `0.9.1`,
  fuzzing the new payload helpers directly, adding primitive delegation fuzz
  coverage, documenting the primitive error-mapping contract, preserving
  `ExactSizeIterator` accounting on list iterator error paths, and hardening
  payload-helper/deployment-policy documentation.
- Updated independent crate release metadata so only `eth-valkyoth-codec`,
  `eth-valkyoth-primitives`, and `eth` publish for this release.

## Still Required Before Tag

- GitHub checks must pass on the final release commit.

## Verification

```bash
cargo test -p eth-valkyoth-codec -p eth-valkyoth-primitives
cargo check --manifest-path fuzz/Cargo.toml
scripts/checks.sh
scripts/release_0_9_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --dry-run --skip-checks --yes
```
