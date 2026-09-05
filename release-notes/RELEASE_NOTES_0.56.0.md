# eth v0.56.0

Status: pentest retest clean; workflow update awaiting fresh GitHub checks and tag approval.
Publication: DEFERRED TO v0.60.0

## Scope

First-party BLS12-381 base-field arithmetic for public inputs, through the
existing `EvmBls12381Fp` type: conversion, wide reduction, addition,
subtraction, multiplication, square, negation, inversion and square root.

The arithmetic kernel uses fixed-width Montgomery limbs without allocation,
runtime dependencies or unsafe code. Canonical wire decoding is unchanged and
does not reduce malformed field encodings. Nonsquares and zero inversion
return `None`; square roots choose the smaller canonical root.

## Security And Limits

The initial pentest identified three release-control findings, now addressed
with signed-history and effective-Cargo-contract regression tests. See the
[remediation record](../docs/release-control-remediation-0.56.0.md). Field
arithmetic was unchanged. A follow-up caller-identity finding now binds current
tag admission and baseline comparisons to captured authenticated commits,
with conflicting/moving-reference regressions. The maintainer confirmed the
external retest clean on 2026-09-05.

The maintainer requested a simpler release loop: implementation tests first,
pentest/fixes with regression tests and report updates, commit and GitHub wait,
then explicit permission to tag/push. The active gate now uses
`--implementation` for portable verification and default/`--tag` for final
metadata/report admission. Signed identities, exact report lineage, prior
train evidence and cumulative publication checks are unchanged. The workflow
change has local regression coverage; it is not a new external retest claim.

- Public inputs only; no constant-time or secret-key safety claim.
- No G1/G2 operations, subgroup checks, map-to-curve, pairing or signature
  verification are enabled. Charged BLS precompile execution remains closed.
- Fixed-size scratch and loop bounds; standalone field methods have no gas
  authorization and hosts remain responsible for aggregate workload admission.
- Independent BigUint differential tests, boundary/carry tests, algebraic
  fuzzing and fixed-work benchmarks cover the new field layer.
- Clean pentest does not waive portable implementation checks or hosted CI/CodeQL.
- The existing external-client ModExp regression is still host-blocked by
  missing Podman CPU/memory delegation. It is a separate integration evidence
  run, not a tag-stage prerequisite for this field-only milestone. No client
  result is claimed and the runner's isolation checks remain unchanged.

See the [scope, sources and resource contract](../docs/bls12-381-base-field.md).
The retained support-crate versions are internal source snapshots, not new
crates.io publications; all package changes are reviewed cumulatively at
`v0.60.0`. Published dependency examples remain on `eth 0.55.0`.
