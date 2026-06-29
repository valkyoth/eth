# eth 0.8.0 Release Notes

Status: implementation ready for external pentest

## Summary

`0.8.0` is the canonical RLP integer milestone. It extends the bounded RLP
decoder from canonical scalars and lists to Ethereum integer interpretation:
zero is the empty byte array, positive values are shortest-form unsigned
big-endian bytes, and leading-zero payloads are rejected before protocol
validation.

## Included So Far

- Added `RlpInteger` as a validated wrapper around decoded scalar payloads.
- Added exact and partial integer decoders:
  `decode_rlp_integer` and `decode_rlp_integer_partial`.
- Added bounded conversion helpers:
  `decode_rlp_u64`, `decode_rlp_u128`, and `decode_rlp_u256_bytes`.
- Added `RlpInteger::to_u64`, `RlpInteger::to_u128`, and
  `RlpInteger::to_be_bytes32`.
- Rejected noncanonical integer zero encoded as a single byte `0x00`.
- Rejected leading-zero integer payloads such as `0x82 0x00 0x01`.
- Added canonical RLP integer payload constructors for `ChainId`,
  `BlockNumber`, `Gas`, `Nonce`, `UnixTimestamp`, and `Wei`.
- Added fuzz coverage for exact and partial RLP integer decoding plus bounded
  conversion helpers.
- Split primitive tests into a separate module to keep implementation files
  below the 500-line limit.
- Refreshed the pinned EIPs revision after checking official Ethereum sources
  on 2026-06-29.

## Still Required Before Tag

- External pentest for the exact implementation commit.
- Permanent `security/pentest/v0.8.0.md` report after findings are resolved.
- GitHub checks must pass on the final release commit.

## Verification

```bash
scripts/checks.sh
scripts/release_0_8_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
```
