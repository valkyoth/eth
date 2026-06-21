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
- Addressed pentest findings by rejecting enum sanitization derives, adding
  hardened-mode evidence, pinning official Ethereum source revisions, renaming
  the production decode-limit template, expanding fuzz coverage, documenting
  hash timing caveats, removing the dead-code typestate constructor from normal
  builds, marking public error enums non-exhaustive, documenting
  `TransactionType::try_from`, and fixing skipped-field generic derive bounds.
- Addressed follow-up pentest findings by rejecting enum and union
  `SecureSanitizeOnDrop` derives, documenting the conservative derive-bound
  fallback, and adding a downstream `HARDENED_MODE` assertion example.
- Marked `eth-valkyoth-codec`, `eth-valkyoth-primitives`,
  `eth-valkyoth-protocol`, `eth-valkyoth-verify`, `eth-valkyoth-derive`,
  `eth-valkyoth-sanitization`, and `eth` for `0.5.0` publication.
- Marked `eth-valkyoth-signer` for dependency-only `0.3.2` publication.
- Updated the optional `sanitization` dependency to `1.1.1`.
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
