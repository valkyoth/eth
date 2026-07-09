# Release Notes - eth v0.51.0

Status: implementation complete; awaiting pentest.

## Summary

This release adds first-party dependency-free EIP-152 BLAKE2F precompile
execution to `eth-valkyoth-evm-core`.

BLAKE2F now uses the existing fork-aware precompile descriptor, exact 213-byte
input policy, output-length metadata, and round-count gas policy, then executes
the compression function without adding default crypto, allocator, or backend
dependencies.

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

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.23.0` to `0.24.0`.
- `eth` is bumped from `0.50.10` to `0.51.0`.
- BLAKE2F descriptors now report `NativeBlake2F` instead of a backend-required
  placeholder.
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
- KZG and BLS precompile execution remains fail-closed until later audited
  release slices.

## Verification

- `cargo test -p eth-valkyoth-evm-core blake2f`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin blake2f_frame -- -D warnings`
- `cargo fmt --all --check`
- `scripts/validate-release-metadata.sh`
- `scripts/release_crates.py --check`

## Pentest

Pentest is required before tagging. The final report must be committed at
`security/pentest/v0.51.0.md` with `Status: PASS`.
