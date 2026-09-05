# eth v0.56.0

Status: implementation candidate; awaiting pentest.
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

- Public inputs only; no constant-time or secret-key safety claim.
- No G1/G2 operations, subgroup checks, map-to-curve, pairing or signature
  verification are enabled. Charged BLS precompile execution remains closed.
- Fixed-size scratch and loop bounds; standalone field methods have no gas
  authorization and hosts remain responsible for aggregate workload admission.
- Independent BigUint differential tests, boundary/carry tests, algebraic
  fuzzing and fixed-work benchmarks cover the new field layer.
- Pentest and final release admission are pending, not implied by local tests.
- The existing external-client ModExp regression is still host-blocked by
  missing Podman CPU/memory delegation. It remains a mandatory pre-tag check.

See the [scope, sources and resource contract](../docs/bls12-381-base-field.md).
The retained support-crate versions are internal source snapshots, not new
crates.io publications; all package changes are reviewed cumulatively at
`v0.60.0`. Published dependency examples remain on `eth 0.55.0`.
