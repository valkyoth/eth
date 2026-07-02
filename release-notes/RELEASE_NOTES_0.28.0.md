# eth 0.28.0 Release Notes

Status: pentest passed; waiting for final GitHub checks before tagging

## Summary

`0.28.0` adds syntactic execution block header decoding and block header
hashing.

The decoder is intentionally not a full header-validity engine. It admits one
canonical RLP header under explicit `DecodeLimits`, checks the selected fork
field count, decodes fixed-width fields into domain types, and preserves the
exact canonical RLP bytes for hashing.

## Added

- Added `HeaderFieldSet` for legacy, London, Shanghai, Cancun, and Prague
  header layouts.
- Added `decode_block_header`.
- Added `UnvalidatedBlockHeader`.
- Added fixed-width `LogsBloom`.
- Added `BlockHash`, a domain newtype around `B256`.
- Added `UnvalidatedBlockHeader::hash_with` using the caller-provided
  `Keccak256` trait boundary.
- Added malformed header tests for field-count and fixed-width failures.
- Added a resource-exhaustion regression for oversized header input.
- Added hash consistency tests that verify hashing uses the exact canonical RLP
  header bytes that were decoded.
- Added `fuzz/fuzz_targets/header.rs` to drive `decode_block_header` across all
  current `HeaderFieldSet` variants under fixture and deployment limits.
- Added `docs/block-headers.md`.

## Security Notes

- Header decoding remains syntactic. It does not prove ancestry, fork
  activation, state root, transaction root, receipt root, logs-bloom
  correctness, gas accounting, base-fee calculation, withdrawals root, blob gas
  accounting, parent beacon root, requests hash, or consensus-layer
  commitments.
- Header hashing returns `BlockHash`, not raw `B256`, so later proof APIs can
  keep block-hash domains distinct from transaction hashes, receipt roots, and
  trie roots.
- The crate does not infer fork layout from block number or timestamp. Callers
  must pass the reviewed `HeaderFieldSet` for the context they are decoding.
- `extra_data` is decoded as borrowed bytes and is not checked against a
  network's consensus-specific length limit, such as the 32-byte mainnet cap.

## Specification Notes

- EIP-4895 defines the `withdrawals_root` header extension.
- EIP-4844 defines `blob_gas_used` and `excess_blob_gas`.
- EIP-4788 defines `parent_beacon_block_root`.
- EIP-7685 defines `requests_hash`.

## Versioning

- `eth-valkyoth-protocol` publishes as `0.23.0`.
- `eth-valkyoth-verify` publishes as `0.17.1` because its published protocol
  dependency range changes.
- The facade crate publishes as `eth` `0.28.0`.
- Other unchanged support crates are not republished.

## Release Gate

- External pentest passed for
  `c2989eab13fad32ac1cc2133452e8eaf1ee3d695`.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify --all-features
cargo clippy -p eth-valkyoth-protocol -p eth-valkyoth-verify -p eth --all-targets --all-features -- -D warnings
scripts/release_0_28_gate.sh
```
