# eth 0.50.1 Release Notes

Status: release candidate; pentest and retest complete.

`0.50.1` adds BN254 pairing G2 subgroup validation to
`eth-valkyoth-evm-core`. The release keeps the `v0.50.0` bounded EIP-197 frame
boundary and empty-input execution, adds a first-party scalar-multiplication
subgroup gate for G2 points, and precomputes the twist curve coefficient used
by G2 curve validation.

This release still does not claim non-empty BN254 pairing algebra. Non-empty
pairing inputs are parsed, checked for G1 validity and G2 curve/subgroup
membership, and then fail closed with `PrecompileBackendUnavailable` until the
Miller-loop and final-exponentiation releases are admitted.

## Added

- Private dependency-free G2 subgroup validation for BN254 pairing frames.
- `PrecompilePointNotInSubgroup` with the stable error code
  `precompile_point_not_in_subgroup`.
- A deterministic valid-twist, invalid-subgroup regression fixture.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.14.0`.
- `eth` publishes as `0.50.1` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.14.0`.
- The BN254 pairing frame parser now rejects valid twist points that are not in
  the admitted subgroup.
- G2 curve validation uses precomputed Montgomery constants for the twist
  coefficient instead of recomputing `3 / (9 + i)` during tuple validation.
- README, crate version matrix, EVM fork matrix, spec matrix, implementation
  plan, and release plan now document G2 subgroup validation.
- The release gate is available as `scripts/release_0_50_1_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, allocator, or pairing backend dependency is
  added.
- The subgroup check uses first-party Fp2 projective scalar multiplication by
  the BN254 group order and accepts only points where `[r]P` is infinity.
- The subgroup check is variable-time over public calldata and the fixed public
  BN254 group order; it is not a secret-scalar primitive.
- Infinity remains accepted as the subgroup identity.
- Field range, curve membership, input length, input ceiling, output size, and
  fail-closed non-empty execution behavior from `v0.50.0` remain enforced.
- Future EVM dispatch integration must charge the already planned precompile
  gas before running pairing frame validation or execution.
- Non-empty pairing algebra, BLAKE2F, KZG, and BLS12-381 precompiles still fail
  closed until their release slices are admitted.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core bn254_pairing`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo check --manifest-path fuzz/Cargo.toml`
- `cargo +1.90.0 check -p eth --features evm-core`
- `cargo +1.96.1 check -p eth --features evm-core`
- `cargo package -p eth-valkyoth-evm-core --allow-dirty`
- `cargo package -p eth --allow-dirty --config 'patch.crates-io.eth-valkyoth-evm-core.path="crates/eth-valkyoth-evm-core"'`
- `cargo deny check`
- `cargo audit`
- `scripts/release_crates.py --check`
- `python3 scripts/test-release-metadata.py`
- `scripts/release_0_50_1_gate.sh`

## Pentest

- Permanent report: `security/pentest/v0.50.1.md`.
- The release-scope review and retest found no exploitable findings. Two
  informational notes were documented before the final release gate.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.14.0`.
- `eth` publishes as `0.50.1`.
- Other support crates are unchanged and are not republished.
