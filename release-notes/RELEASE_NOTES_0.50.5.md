# eth 0.50.5 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.50.5` adds the first-party BN254 Miller-loop accumulator over validated
EIP-197 pairing tuples. The accumulator consumes the existing G2 subgroup
checks, validated tuple stream, Fp6/Fp12 tower, and line-function helpers.

This release still does not claim final exponentiation or non-empty BN254
pairing results. Non-empty EIP-197 pairing execution remains fail-closed with
`PrecompileBackendUnavailable`.

## Added

- Internal BN254 ate-loop accumulator over validated pairing tuples.
- Fixed public BN254 ate-loop bit schedule.
- Miller-loop unit tests for empty input, infinity neutrality, deterministic
  generator accumulation, tuple-stream consistency, and batch multiplication
  shape.
- Fuzz harness coverage for valid pairing-frame inputs reaching the Miller
  accumulator through the `testing` feature.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.18.0`.
- `eth` publishes as `0.50.5` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.18.0`.
- The fail-closed BN254 pairing path now exercises the Miller accumulator
  instead of only the tower/line-function carrier.
- The release gate is available as `scripts/release_0_50_5_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, allocator, or pairing backend dependency is
  added.
- The new arithmetic remains ordinary variable-time arithmetic over public EVM
  calldata-derived values. It must not be reused for secret scalar or private
  key operations.
- The Miller accumulator is internal and does not expose or claim a public
  pairing result.
- Non-empty pairing execution still returns `PrecompileBackendUnavailable`
  until final exponentiation is reviewed and admitted.
- The low-level free functions remain public unmetered primitives for
  standalone tests and fuzzing. Interpreter dispatch must use the gas-gated
  `EvmPrecompilePlan` methods instead.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`

## Pentest

- Pending. Permanent report will be added at `security/pentest/v0.50.5.md`
  after the external pentest and retest.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.18.0`.
- `eth` publishes as `0.50.5`.
- Other support crates are unchanged and are not republished.
