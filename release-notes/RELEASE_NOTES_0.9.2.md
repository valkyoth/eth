# eth 0.9.2 Release Notes

Status: implementation in progress; pentest required before tag

## Summary

`0.9.2` adds a primitive RLP bridge so common Ethereum domain types can encode
to canonical RLP and decode from bounded RLP without callers writing repeated
codec glue.

This release keeps the single canonicality source introduced in `0.9.1`:
integer primitive decoders delegate through `eth-valkyoth-codec`, while address
and hash domains enforce fixed-width scalar payloads at the primitive boundary.

## Included So Far

- Added no-allocation `encoded_rlp_len`, `encode_rlp`, and `try_from_rlp`
  helpers for `ChainId`, `BlockNumber`, `Gas`, `Nonce`, and `UnixTimestamp`.
- Added the same bridge helpers for `Wei` using canonical U256 payload
  trimming and codec-backed U256 decode.
- Added fixed-width RLP scalar encode/decode helpers for `Address` and `B256`.
- Added `PrimitiveRlpError` to keep codec and primitive errors explicit without
  losing source category information.
- Added table-driven round-trip tests for integer domains, fixed-width byte
  domains, and wei values.
- Added malformed-input tests for non-canonical integers, wrong fixed-width
  scalars, and too-small caller-provided output buffers.
- Updated README examples and independent crate release metadata for publishing
  only `eth-valkyoth-primitives` and the `eth` facade in this release.

## Still Required Before Tag

- Maintainer pentest must be run for the exact implementation commit.
- Any pentest findings must be fixed and retested.
- A permanent report must be written at `security/pentest/v0.9.2.md`.
- GitHub checks must pass on the final release report commit.

## Verification

```bash
cargo test -p eth-valkyoth-codec -p eth-valkyoth-primitives -p eth
cargo check --manifest-path fuzz/Cargo.toml
scripts/checks.sh
scripts/release_0_9_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
