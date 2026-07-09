# BN254 Pairing Economics

Status: `v0.50.9` evidence for sparse Miller-loop multiplication, bounded final exponentiation, Frobenius Q1/-Q2 point mapping, and the projective post-loop line carrier.

## Scope

This document tracks the gas-vs-CPU evidence for the first-party BN254
pairing implementation in `eth-valkyoth-evm-core`.

`v0.50.9` still does not admit non-empty EIP-197 pairing success. Non-empty
inputs validate and exercise the internal Miller accumulator, bounded final
exponentiation, Frobenius Q1/-Q2 point helper, and projective post-loop line
carrier, then fail closed with `PrecompileBackendUnavailable` until final
result admission is reviewed in a later release.

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

## v0.50.8 Evidence

The v0.50.8 release adds the Q1/-Q2 Frobenius point foundation required by the
optimal-ate post-loop terms. During implementation, the project tested the
post-loop points against the current affine line carrier and found that this
combination maps the EIP-197 generator tuple to one after final exponentiation.
That is an invalid result for a single generator tuple, so the post-loop points
are admitted and KAT-backed, but the line multiplication remains disabled until
the v0.50.9 projective/reference-aligned line-carrier release.

## v0.50.9 Evidence

The v0.50.9 release replaces the fail-closed accumulator's affine post-loop
shortcut with a projective line carrier following the reviewed optimal-ate
shape. The completed accumulator now multiplies the Q1 and -Q2 post-loop line
terms before final exponentiation, but still returns
`PrecompileBackendUnavailable` for public non-empty EIP-197 pairing execution.

Regression coverage proves that a single EIP-197 generator tuple no longer maps
to one after final exponentiation, while the generator plus negated-generator
batch still maps to one. The BN254 pairing fuzz target also reaches the
complete fail-closed accumulator for every parsed valid frame.

The v0.50.9 pentest found and blocked an incorrect hand-entered NAF table for
the BN254 `6u+2` optimal-ate scalar. The remediation replaces the table with an
independently derived value and adds two permanent regressions: one reconstructs
the scalar from the table, and one checks `e([2]P, Q) == e(P, Q)^2` using the
existing EIP-196 G1-add implementation to build `[2]P`.

## Next Gate

Before non-empty pairing execution can be admitted, the follow-up optimal-ate
and result-admission releases must add:

- official EIP-197 positive and negative vectors;
- independent differential vectors from an admitted reference source;
- complete result-admission KATs;
- updated release-mode benchmark evidence for complete pairing execution;
- a pentest report covering both correctness and gas-vs-CPU behavior.
