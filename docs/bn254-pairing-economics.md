# BN254 Pairing Economics

Status: `v0.50.7` evidence for sparse Miller-loop multiplication plus bounded final exponentiation.

## Scope

This document tracks the gas-vs-CPU evidence for the first-party BN254
pairing implementation in `eth-valkyoth-evm-core`.

`v0.50.7` still does not admit non-empty EIP-197 pairing success. Non-empty
inputs validate and exercise the internal Miller accumulator plus bounded final
exponentiation, then fail closed with `PrecompileBackendUnavailable` until the
optimal-ate post-loop Frobenius/addition line terms and final result admission
are reviewed in later releases.

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

cargo test --release -p eth-valkyoth-evm-core \
  bn254_pairing_final_exponentiation_wall_time_budget_smoke -- --ignored --nocapture
```

The Miller test runs `miller_loop_tuple` over the EIP-197 generator tuple and
prints the total and average nanoseconds. The final-exponentiation test feeds
the same validated tuple through the internal accumulator and times the fixed
EIP-197 exponentiation that v0.50.7 executes before failing closed. Both tests
are intentionally dependency-free so the core crate does not admit a benchmark
framework.

## v0.50.6 Evidence

Local release-mode evidence on 2026-07-09:

```text
bn254_miller_loop_tuple iterations=3 total_ns=2493510 average_ns=831170
```

The v0.50.6 release budget is a reviewed evidence budget, not a consensus
claim: the average release-mode generator tuple run should stay below
2,000,000 ns on the maintainer release host before complete non-empty pairing
result admission is allowed to proceed. If a later machine produces materially
slower numbers, the release must document the hardware context or keep
non-empty pairing fail-closed.

## v0.50.7 Evidence

The v0.50.7 remediation adds an ignored release-mode smoke benchmark for the
new live final-exponentiation path. The final exponentiation still runs only
inside a fail-closed non-empty pairing path; the benchmark is evidence for the
cost already paid before `PrecompileBackendUnavailable`, not a result-admission
claim.

Record the local release-mode output here during release finalization:

```text
bn254_final_exponentiation iterations=3 total_ns=27737043 average_ns=9245681
```

## Next Gate

Before non-empty pairing execution can be admitted, the follow-up optimal-ate
and result-admission releases must add:

- optimal-ate post-loop Frobenius/addition line terms;
- official EIP-197 positive and negative vectors;
- independent differential vectors from an admitted reference source;
- Frobenius and complete-accumulator edge-case KATs;
- updated release-mode benchmark evidence for complete pairing execution;
- a pentest report covering both correctness and gas-vs-CPU behavior.
