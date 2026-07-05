# eth 0.37.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.37.0` performs the REVM dependency admission review for the optional EVM
adapter boundary. The result is a deliberate non-admission: current REVM
candidates do not pass this repository's dependency policy, so no REVM crate is
added to the published dependency graph.

## Added

- `eth-valkyoth-evm` now publishes `RevmDependencyReview` and
  `revm_dependency_review()` so the reviewed REVM admission result is visible
  from code.
- `docs/revm-dependency-review.md` records the checked REVM versions, MSRV
  constraints, cargo-deny failures, and the recheck rule before execution work.
- `scripts/release_0_37_gate.sh` validates the v0.37.0 release slice.
- `scripts/checks.sh` now patches all optional facade support crates during
  local package verification so pre-publish checks can validate a support-crate
  bump before that crate exists in the crates.io index.
- `docs/RELEASE_PLAN.md`, `docs/IMPLEMENTATION_PLAN.md`, `docs/SCOPE.md`, and
  `docs/SPEC_MATRIX.md` now make the first-party-core goal explicit: current
  cryptographic implementation dependencies must be audited behind boundaries,
  native EVM execution is followed by full execution state/block validity, and
  RPC/Reth/node work moves after full execution fixture admission.

## Security Notes

- `revm 41.0.0` was current during review, but it requires Rust `1.91.0`; this
  workspace still supports Rust `1.90.0`.
- `revm 36.0.0` is the newest reviewed `revm` line compatible with Rust
  `1.90.0`, but it fails the dependency gate due to duplicate crypto/hash
  lines, an unmaintained `paste` advisory, and a `CC0-1.0` transitive license
  through the precompile graph.
- `revm-primitives 22.1.0` was checked as a narrower candidate, but it still
  fails the dependency gate due to duplicate crypto/hash lines and the
  unmaintained `paste` advisory.
- No REVM dependency is admitted in v0.37.0, and the default facade graph
  remains unchanged.

## Versioning

- `eth-valkyoth-evm` publishes as `0.8.0` for the new dependency-review API.
- The facade crate publishes as `eth` `0.37.0`.
- All other support crates remain on their v0.36.0 published versions.
