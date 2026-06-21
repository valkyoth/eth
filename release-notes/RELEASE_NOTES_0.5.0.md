# eth 0.5.0 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.5.0` strengthens the decode-budget model before the first real RLP parser
lands. The codec crate now has explicit proof-node and cumulative decoded item
budgets in addition to byte, list, nesting, and allocation limits.

## Included

- Added `max_proof_nodes` and `max_total_items` to `DecodeLimits`.
- Added `check_proof_node_count` and `check_item_count` for single-count
  validation.
- Added `DecodeAccumulator::account_proof_nodes` and
  `DecodeAccumulator::account_items` for cumulative DoS accounting.
- Added checked length and range helpers for decoder offset arithmetic:
  `checked_len_add`, `checked_range_end`, and `require_range_in_bounds`.
- Added stable decode/resource errors for proof-node, item-count, overflow,
  and out-of-bounds failures.
- Expanded the fuzz target to exercise the new budget APIs.
- Split the codec crate into focused modules so source files remain below the
  500-line limit.
- Marked `eth-valkyoth-codec` and `eth` for `0.5.0` publication.
- Updated the optional `sanitization` dependency to `1.1.1` and marked
  `eth-valkyoth-sanitization` for dependency-only `0.4.1` publication.
- Left all other support crates on their previous published versions.

## Verification

```bash
scripts/checks.sh
scripts/release_0_5_gate.sh
scripts/release_crates.py --check
cargo test -p eth-valkyoth-codec
cargo deny check
cargo audit
```
