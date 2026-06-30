# eth 0.9.1 Release Notes

Status: implementation complete; pending external pentest input

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
- Updated independent crate release metadata so only `eth-valkyoth-codec`,
  `eth-valkyoth-primitives`, and `eth` publish for this release.

## Still Required Before Tag

- Maintainer pentest for the exact implementation commit.
- Permanent report at `security/pentest/v0.9.1.md` with `Status: PASS`.
- GitHub checks must pass on the final release commit.

## Verification

```bash
cargo test -p eth-valkyoth-codec -p eth-valkyoth-primitives
scripts/checks.sh
scripts/release_crates.py --dry-run --skip-checks --yes
```
