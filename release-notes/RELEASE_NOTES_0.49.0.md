# eth 0.49.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.49.0` adds dependency-free native BN254 point addition and scalar
multiplication precompile execution to `eth-valkyoth-evm-core`. The release
keeps the EVM core `no_std` and allocator-free by using fixed-size first-party
field arithmetic for the EIP-196 `alt_bn128` G1 precompile domain.

This release does not add BN254 pairing execution. Pairing remains a bounded
plan and is still scheduled for `0.50.0`, where batch behavior and pairing
costs can be reviewed separately.

## Added

- `EVM_BN254_POINT_BYTES`, `EVM_BN254_ADD_INPUT_BYTES`, and
  `EVM_BN254_MUL_INPUT_BYTES`.
- `execute_bn254_add` for precompile address `0x06`.
- `execute_bn254_mul` for precompile address `0x07`.
- `EvmPrecompilePlan::execute_bn254_add` and
  `EvmPrecompilePlan::execute_bn254_mul`.
- First-party canonical BN254 field-element parsing and range checks.
- Point-at-infinity handling for `(0, 0)`.
- Invalid field and invalid point rejection.
- Tests for independently computed generator doubling, generator scalar
  multiplication, empty input, full-width scalar acceptance, output-buffer
  behavior, wrong-plan dispatch, and Byzantium/Istanbul gas.
- `fuzz/fuzz_targets/bn254_frame.rs` for BN254 add/mul frame fuzz coverage and
  a `P + P == 2 * P` invariant for valid fuzzed points.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.12.0`.
- `eth` publishes as `0.49.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.12.0`.
- The precompile registry now reports BN254 add/mul as native implementations.
- README, crate version matrix, EVM fork matrix, spec matrix, implementation
  plan, and release plan now document native BN254 add/mul execution.
- The release gate is available as `scripts/release_0_49_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, or allocator dependency is added.
- Field elements are rejected when they are equal to or greater than the BN254
  field modulus.
- The field modulus is the BN254 base-field prime used for point coordinates,
  not the scalar-field order.
- Field multiplication uses fixed-modulus Montgomery multiplication rather than
  bit-serial long division.
- Points are rejected unless they are `(0, 0)` or satisfy `y^2 = x^3 + 3`.
- Out-of-range field elements and off-curve points return distinct named error
  variants for diagnostics while preserving precompile reject behavior.
- Short inputs use EIP-196 virtual zero padding, and surplus bytes are ignored
  by the fixed precompile frame parser.
- Output buffers are checked before execution writes any result bytes.
- `execute_bn254_add` and `execute_bn254_mul` are documented as public EVM
  calldata arithmetic, not constant-time secret-key arithmetic.
- Scalar multiplication treats the scalar as an arbitrary 256-bit integer, as
  specified by EIP-196.
- BN254 pairing, BLAKE2F, KZG, and BLS12-381 precompiles still fail closed with
  `PrecompileBackendUnavailable` until their release slices are admitted.

## Spec References

- EIP-196 defines BN254 add/mul precompile addresses, encoding, padding,
  invalid-input behavior, and Byzantium gas:
  <https://eips.ethereum.org/EIPS/eip-196>.
- EIP-1108 defines Istanbul BN254 gas reductions:
  <https://eips.ethereum.org/EIPS/eip-1108>.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo check --manifest-path fuzz/Cargo.toml --bin bn254_frame`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_frame -- -D warnings`
- `cargo check -p eth --features evm-core`
- `cargo test -p eth --all-features --doc`
- `scripts/release_crates.py --check`
- `python3 scripts/test-release-metadata.py`

## Pentest

- Pending. Permanent report will be added as `security/pentest/v0.49.0.md`
  after the release-scope pentest and retest are complete.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.12.0`.
- `eth` publishes as `0.49.0`.
- Other support crates are unchanged and are not republished.
