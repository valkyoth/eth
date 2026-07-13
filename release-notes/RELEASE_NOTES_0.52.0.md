# Release Notes - eth v0.52.0

Status: implementation complete; awaiting pentest.

## Summary

This release replaces placeholder advanced-precompile planning with exact
EIP-4844 and EIP-2537 contracts. KZG and BLS12-381 arithmetic remains fail
closed, but callers can now plan canonical input shapes, fixed output lengths,
and every official gas schedule before a backend is admitted.

The release also commits to first-party dependency-free BLS12-381 and KZG
implementation sequences. External implementations may serve as independent
test oracles or optional accelerators, but not as the sole native consensus
path.

## Added

- `NonEmptyMultipleOf` precompile input policy for EIP-2537 MSM and pairing.
- EIP-2537 G1/G2 MSM and pairing gas policy domains.
- Official 128-entry G1 and G2 MSM discount tables with capped discounts.
- Exact KZG/BLS frame and fixed output metadata for addresses `0x0a..=0x11`.
- Boundary tests for fixed frames, empty/partial variable frames, output sizes,
  fixed gas, MSM discount boundaries, and pairing tuple gas.
- An independent EIP-2537 fixture oracle covering every G1/G2 MSM discount,
  every corresponding gas result, and the 129-item capped-discount boundary.
- `advanced_precompile_plan` fuzz target for all KZG/BLS planning paths.
- Advanced-precompile backend admission and conformance policy.
- Concrete first-party BLS releases `v0.52.1..=v0.52.9` and KZG releases
  `v0.61.0..=v0.61.5`.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.24.0` to `0.25.0`.
- `eth-valkyoth-verify` is patch-bumped to `0.21.1` for `k256 0.14.0`.
- `eth-valkyoth-derive` is patch-bumped to `0.17.3` for `trybuild 1.0.118`.
- `eth-valkyoth-sanitization` is patch-bumped to `0.7.5` for `sanitization
  1.2.4` and the derive dependency update.
- `eth` is bumped from `0.51.0` to `0.52.0`.
- Prague BLS descriptors no longer use `BoundedAny`, unknown output lengths,
  or deferred dynamic gas.
- Advanced-precompile gas arithmetic is checked and split into focused modules
  so every source remains below 500 lines.
- Optional `k256` recovery uses the stable `0.14.0` API while retaining the
  project-owned scalar and low-s validation boundary.

## Security Notes

- No crypto, allocator, bigint, KZG, BLS, or trusted-setup dependency is added.
- All unimplemented KZG/BLS execution still reports backend unavailable.
- Empty BLS MSM and pairing frames are rejected as required by EIP-2537.
- Gas planning uses checked count, multiplication, and addition operations.
- The 1 MiB global precompile input limit remains in force.
- First-party execution cannot be admitted until official vectors,
  differential evidence, fuzzing, dependency review, and pentest pass.
- Every transcribed EIP-2537 discount entry is checked against a separately
  stored official-spec fixture before any later release makes the charge live.
- Public precompile values are not treated as secret material; any future reuse
  for secret-bearing key operations requires a separate sanitization contract.

## Verification

- `cargo test -p eth-valkyoth-evm-core precompile`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin advanced_precompile_plan -- -D warnings`
- `cargo fmt --all --check`
- `scripts/validate-release-metadata.sh`
- `python3 scripts/test-release-metadata.py`
- `scripts/release_crates.py --check`
- `scripts/checks.sh`

## Pentest

The release must not be tagged until its independent pentest, remediation,
retest, and committed `security/pentest/v0.52.0.md` report are complete.
