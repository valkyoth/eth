# eth 0.50.2 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.50.2` adds the dependency-free BN254 Fp6/Fp12 tower foundation required by
the future Miller-loop release. The release keeps EIP-197 non-empty pairing
execution fail-closed, but now compiles and exercises the Fp12 tower arithmetic
through a bounded internal accumulation shape tied to the already validated
pairing tuple count.

This release still does not claim non-empty BN254 pairing algebra. Non-empty
pairing inputs are parsed, checked for G1 validity and G2 curve/subgroup
membership, run through the bounded tower-accumulation shape, and then fail
closed with `PrecompileBackendUnavailable` until the Miller-loop and
final-exponentiation releases are admitted.

## Added

- Private dependency-free `Fp6` arithmetic over `Fp2[v] / (v^3 - (9 + i))`.
- Private dependency-free `Fp12` arithmetic over `Fp6[w] / (w^2 - v)`.
- Bounded internal tower-accumulation shape reached from non-empty pairing
  execution before the existing fail-closed return.
- Algebraic tests for `v^3 = 9 + i`, `w^2 = v`, identity/zero behavior,
  squaring, and distributivity.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.15.0`.
- `eth` publishes as `0.50.2` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.15.0`.
- The `v0.50.2` roadmap scope is narrowed to the tower foundation. Miller-loop
  line functions move to `v0.50.3`, and final exponentiation/non-empty pairing
  execution moves to `v0.50.4`.
- README, crate version matrix, EVM fork matrix, spec matrix, implementation
  plan, and release plan now document the tower foundation and updated pairing
  schedule.
- The release gate is available as `scripts/release_0_50_2_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, allocator, or pairing backend dependency is
  added.
- The tower operations are reached only after the existing input-size,
  tuple-length, G1, G2 field, G2 curve, and G2 subgroup validation boundaries.
- The tower-accumulation shape is bounded by `pairs = input.len() / 192`, and
  `input.len()` remains capped by `EVM_PRECOMPILE_INPUT_LIMIT`.
- Non-empty pairing algebra, BLAKE2F, KZG, and BLS12-381 precompiles still fail
  closed until their release slices are admitted.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core bn254_tower`
- `cargo test -p eth-valkyoth-evm-core bn254_pairing`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`

## Pentest

- Pending. Permanent report will be added at `security/pentest/v0.50.2.md`
  after the external pentest and retest.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.15.0`.
- `eth` publishes as `0.50.2`.
- Other support crates are unchanged and are not republished.
