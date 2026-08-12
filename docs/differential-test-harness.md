# Differential Test Harness

Status: `v0.55.0` covers structural RLP and arbitrary-length ModExp through
dev-only independent reference paths.

## Scope

| Area | Local path | Independent reference | Claim |
| --- | --- | --- | --- |
| Structural RLP | `eth-valkyoth-codec::differential_rlp_reference` | `alloy-rlp` `0.3.16` | Valid/invalid structural decisions and exact accepted re-encoding match for the curated corpus. |
| ModExp arithmetic | `eth-valkyoth-evm-core::modexp_differential` | `num-bigint` `0.5.1` | Exact output matches from 1 through 256-byte widths plus leading-zero, even, zero, unequal-width, sparse, truncated, and right-padded operands. |

Structural RLP comparison cannot distinguish every Ethereum integer-domain
rule from ordinary byte-string validity. Codec integer tests, primitive bridge
tests, and fuzz targets cover those semantic domains separately.

## Commands

Validate that both integration tests compile:

```sh
scripts/run_differential_tests.py --check
```

Run both reference paths:

```sh
scripts/run_differential_tests.py
```

The runner executes:

```sh
cargo test -p eth-valkyoth-codec --test differential_rlp_reference --features testing
cargo test -p eth-valkyoth-evm-core --test modexp_differential
```

The fuzz workspace also includes `rlp_differential` and `modexp_frame`.
`modexp_frame` drives 256-bit length parsing, both fork gas formulas, bounded
workspace admission, and atomic execution. Every payable frame within harness
capacity must execute successfully. Its execution allocation is harness-capped;
wider declarations still reach parsing and gas calculation.

## Mismatch Reporting

The RLP test accumulates all mismatches before failing and reports each corpus
case and mismatch class. ModExp failures report the operand shape whose local
output differs from the independent BigUint result.
