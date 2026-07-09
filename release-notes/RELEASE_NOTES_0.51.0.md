# Release Notes - eth v0.51.0

Status: release candidate; pentest remediation and retest complete.

## Summary

This release adds first-party dependency-free EIP-152 BLAKE2F precompile
execution to `eth-valkyoth-evm-core`.

BLAKE2F now uses the existing fork-aware precompile descriptor, exact 213-byte
input policy, output-length metadata, and round-count gas policy, then executes
the compression function without adding default crypto, allocator, or backend
dependencies.

The v0.51.0 pentest also found a pre-existing BN254 final-exponentiation
performance issue in the already-admitted pairing path. This release candidate
now replaces the previous full 3072-bit exponentiation with the standard
easy-part reduction and BN-parameter hard-part chain while keeping a test-only
full-exponent reference check.

## Added

- Native EIP-152 BLAKE2F compression execution.
- `EvmPrecompilePlan::execute_blake2f`, which charges the plan gas before
  execution.
- Direct `execute_blake2f` helper for test and integration use.
- Public `EVM_BLAKE2F_INPUT_BYTES` and `EVM_BLAKE2F_OUTPUT_BYTES` constants.
- EIP-152 zero-round, final-block, non-final-block, one-round, and max-round
  planning vector coverage.
- Fuzz target for BLAKE2F input shape, gas planning, final-flag behavior, and
  bounded-round execution.
- Fp6/Fp12 inversion and Frobenius arithmetic needed by optimized BN254 final
  exponentiation.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.23.0` to `0.24.0`.
- `eth` is bumped from `0.50.10` to `0.51.0`.
- BLAKE2F descriptors now report `NativeBlake2F` instead of a backend-required
  placeholder.
- BN254 final exponentiation now uses the easy part plus hard-part addition
  chain instead of the earlier full-exponent fallback.
- Documentation now records BLAKE2F as an executable Istanbul precompile.

## Security Notes

- The release does not add default crypto, allocator, bigint, or BLAKE2 backend
  dependencies.
- The input contract is exact-length only: every executable BLAKE2F frame must
  be exactly 213 bytes.
- The final-block flag accepts only `0` or `1`.
- The gas policy remains EIP-152 round-count gas using the first four input
  bytes as a big-endian `u32`.
- The fuzz target does not execute arbitrary unbounded round counts; it validates
  shape and gas planning for all frames and executes only bounded-round frames.
- The optimized BN254 final exponentiation is checked against the previous
  full-exponent reference on real Miller-loop accumulator output.
- KZG and BLS precompile execution remains fail-closed until later audited
  release slices.

## Verification

- `cargo test -p eth-valkyoth-evm-core blake2f`
- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin blake2f_frame -- -D warnings`
- `cargo test --release -p eth-valkyoth-evm-core bn254_pairing_final_exponentiation_wall_time_budget_smoke -- --ignored --nocapture`
- `cargo fmt --all --check`
- `scripts/validate-release-metadata.sh`
- `python3 scripts/test-release-metadata.py`
- `scripts/release_crates.py --check`
- `scripts/checks.sh`

## Pentest

Initial pentest reported no issues in the new BLAKE2F implementation and one
Medium pre-existing BN254 final-exponentiation performance issue. The BN254
finding was remediated and retest passed. The final report is committed at
`security/pentest/v0.51.0.md` with `Status: PASS`.
