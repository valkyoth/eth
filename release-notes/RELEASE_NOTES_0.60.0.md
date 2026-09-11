# eth v0.60.0

Status: implementation checks passed; per-tag pentest report finalization pending.
Publication: PENDING

## Public Checkpoint

This release publishes the complete v0.55.0-to-v0.60.0 delta. Every signed tag
retains its own incremental pentest report. v0.60.0 uses the same ordinary
v0.59.0-to-candidate pentest; publication adds no cumulative assessment.
The gate still authenticates prior reports and classifies cumulative package
changes from v0.55.0. Signed-history regression tests cover both public and
internal tags, patch predecessors and rejection of invalid or missing reports.

- v0.56.0: first-party public-input BLS12-381 base-field arithmetic.
- v0.57.0: validated G1 affine/projective points and group operations.
- v0.58.0: exact-input, gas-authorized Prague G1ADD with atomic output.
- v0.59.0: public-input Fp2 arithmetic, inversion and checked square roots.
- v0.60.0: validated G2 affine/projective points, conversions, complete
  addition/doubling, negation and geometric equality.

G2 has all nine positive and seven negative pinned official addition fixtures,
an independent BigUint affine oracle, malformed/rescaled/exceptional tests,
differential fuzzing and a fixed-work host benchmark.

## Packages And Documentation

All 14 packages have cumulative changes and are selected for publication.
The facade becomes 0.60.0, EVM core becomes 0.30.0, and the EVM boundary
becomes 0.13.0. Other support crates receive API-compatible patch bumps for
dependency or documentation/test corrections, not lockstep version changes.
See [the complete version matrix](../docs/CRATE_VERSION_MATRIX.md).

Every package README retains the remotely linked WebP logo and now includes
current scope, installation and runnable examples. Installation uses
`cargo add`, which resolves the latest compatible published package and writes
a versioned manifest; no stale hardcoded version or wildcard is recommended.
The GitHub and facade READMEs remain identical. README tests use the documented
features and catch implicit std assumptions. Package verification includes
the complete unpublished local dependency closure, checks README contents in
every archive, and rejects bundled logo bitmaps.

## Security And Remaining Scope

Arithmetic is variable-time and strictly public-input only. G2 curve
membership does not establish subgroup membership or precompile authority.
No new runtime dependency or default feature is introduced in the G2 slice.
Paid G2ADD and fresh cross-client G2 evidence are v0.61.0; subgroup checks,
MSM, mapping and pairing remain assigned through v0.70.0. KZG and complete
state-transition/client/validator behavior remain later work.

The default core remains dependency-free and no_std. Optional allocator-backed
node tracking still uses the explicit sanitization bridge. REVM remains
unadmitted. See [G2 contracts and verification](../docs/bls12-g2-arithmetic.md).

## Release Sequence

Rust 1.98.1 implementation checks passed, including 760 workspace
tests/doctests, 42 README examples, all 14 archives, strict Clippy, independent
vectors/oracles, targeted fuzzing, supply-chain checks, 14 older-Rust checks and
nine no-default cross-target checks. The fresh ModExp client runner could not
start because the host lacks required Podman controller delegation; no client
pass or weakened-isolation fallback is claimed. Full measurements and scope
limitations are retained in the G2 document.

Finish local implementation checks, use the regular pentest of the exact
candidate against v0.59.0, remediate and retest findings, then record the real
permanent report. Commit final evidence and wait for GitHub/CodeQL green and
explicit maintainer approval before creating the signed tag. Publish in the
validated dependency order only after that approval. No publication yet.
