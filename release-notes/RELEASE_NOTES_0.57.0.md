# eth v0.57.0

Status: implementation checks passed; exact-commit pentest pending.
Publication: DEFERRED TO v0.60.0

## Scope

Adds first-party public-input G1 affine and Jacobian types on the v0.56.0
BLS12-381 base field. Includes canonical on-curve parsing, infinity, conversions,
negation, complete addition/doubling and geometric projective equality.
Existing `EvmBls12381G1Point` parsing remains wire-only and API-compatible.

## Security And Verification

- Private coordinates preserve the on-curve invariant. Canonical wire failures
  are rejected, never reduced. Curve membership is not subgroup membership.
- No new runtime dependency, heap allocation, unsafe code or default feature.
  Public data only; no constant-time, signing or gas-authorization claim.
- All nine pinned official addition vectors, an independent BigUint affine
  oracle, exceptional/group-law/scale tests, raw-wire differential fuzzing,
  fixed-work benchmarks and the implementation gate cover this slice.
- Charged addition remains v0.58.0; subgroup checks v0.62.0. No G2, pairing,
  MSM, map-to-curve or BLS precompile dispatch is enabled by these operations.
- Updated optional `sanitization` to 2.1.0 and dev-only `trybuild` to 1.0.121
  after source review. Neither adds a dependency to the default core runtime.

See the [scope, resources and provenance](../docs/bls12-g1-arithmetic.md).
The Rust 1.98.1 implementation gate, 1.90.0-through-1.98.0 compatibility
checks and nine additional no-default-feature cross-compilation targets passed.
The final differential fuzz smoke completed 15,212 inputs without failure.
The next step is exact-commit pentest. GitHub approval and explicit maintainer
authorization still precede a signed internal tag. Crates.io examples stay on
published `eth 0.55.0`; cumulative publication remains v0.60.0.
