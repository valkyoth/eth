# BN254 Pairing Economics

Status: `v0.50.6` evidence for sparse Miller-loop multiplication.

## Scope

This document tracks the gas-vs-CPU evidence for the first-party BN254
pairing implementation in `eth-valkyoth-evm-core`.

`v0.50.6` still does not admit non-empty EIP-197 pairing success. Non-empty
inputs validate and exercise the internal Miller accumulator, then fail closed
with `PrecompileBackendUnavailable` until final exponentiation and official
known-answer vectors are admitted in a later release.

## Sparse Line-Factor Rule

Miller line factors are represented as an Fp6 operand inside Fp12 rather than
as a dense Fp12 value with a zero second Fp6 lane. The production Miller loop
therefore calls `Fp12::mul_by_fp6` for line factors.

The regression test `sparse_line_factor_multiplication_matches_dense_carrier`
checks that the sparse multiplication path matches the previous dense carrier
for generator doubling and addition lines.

## Benchmark Harness

The release evidence harness is an ignored unit test:

```bash
cargo test --release -p eth-valkyoth-evm-core \
  miller_loop_wall_time_budget_smoke -- --ignored --nocapture
```

The test runs `miller_loop_tuple` over the EIP-197 generator tuple and prints
the total and average nanoseconds. It is intentionally dependency-free so the
core crate does not admit a benchmark framework.

## v0.50.6 Evidence

Local release-mode evidence on 2026-07-09:

```text
bn254_miller_loop_tuple iterations=3 total_ns=2493510 average_ns=831170
```

The v0.50.6 release budget is a reviewed evidence budget, not a consensus
claim: the average release-mode generator tuple run should stay below
2,000,000 ns on the maintainer release host before final exponentiation work is
allowed to proceed. If a later machine produces materially slower numbers, the
release must document the hardware context or keep non-empty pairing
fail-closed.

## Next Gate

Before non-empty pairing execution can be admitted, the final exponentiation
release must add:

- official EIP-197 positive and negative vectors;
- independent differential vectors from an admitted reference source;
- final-exponentiation edge-case KATs;
- updated release-mode benchmark evidence for complete pairing execution;
- a pentest report covering both correctness and gas-vs-CPU behavior.
