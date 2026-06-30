# eth 0.9.0 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.9.0` is the canonical RLP encoding round-trip milestone. It adds
no-allocation encoding helpers for canonical scalar byte strings, Ethereum
integer payloads, list payloads, and already-decoded RLP items.

The release keeps the main crate `no_std` and dependency-free beyond the
existing protocol-core support crates. Encoders write into caller-provided
buffers and return the number of bytes written.

## Included So Far

- Added scalar payload length and encode helpers:
  `encoded_rlp_scalar_len` and `encode_rlp_scalar`.
- Added Ethereum integer payload length and encode helpers:
  `encoded_rlp_integer_len` and `encode_rlp_integer`.
- Added list payload length and encode helpers:
  `encoded_rlp_list_len` and `encode_rlp_list_payload`.
- Raw list-payload helpers validate concatenated child items under explicit
  `DecodeLimits` before returning a length or emitting a list header.
- Addressed v0.9.0 pentest findings by making encode errors leave output
  buffers unchanged, expanding encode fuzz coverage to exact-size output
  buffers, documenting sealed decoded value construction, and hardening
  long-form length invariants.
- Added decoded-value re-encoding helpers:
  `encode_decoded_scalar`, `encode_decoded_integer`, `encode_decoded_list`,
  and `encode_decoded_item`.
- Added decode-then-encode canonicality tests for scalars, integers, lists, and
  child items.
- Added table-driven scalar, integer, empty-list, short-list, long-list,
  noncanonical integer, noncanonical input, and output-buffer regression tests.
- Added fuzz coverage for RLP encoding length helpers, raw payload encoders,
  and decoded-value re-encoding paths.
- Refreshed pinned `execution-apis` and `consensus-specs` revisions after
  checking official Ethereum sources on 2026-06-30. These areas remain
  deferred and no RPC, Engine API, SSZ, or consensus behavior is claimed.
- Updated independent crate release planning so only `eth-valkyoth-codec` and
  `eth` publish for this release.

## Still Required Before Tag

- Maintainer pentest for the exact implementation commit.
- Permanent report at `security/pentest/v0.9.0.md` with `Status: PASS`.
- GitHub checks must pass on the final release commit.

## Verification

```bash
scripts/checks.sh
scripts/release_0_9_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
cargo check --manifest-path fuzz/Cargo.toml
```
