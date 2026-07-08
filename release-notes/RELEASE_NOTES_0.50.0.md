# eth 0.50.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.50.0` adds the bounded BN254 pairing precompile frame boundary to
`eth-valkyoth-evm-core`. The release implements EIP-197 input segmentation,
G1/G2 field and curve validation, fork-aware gas planning, and the fully
specified empty-input success case without adding default dependencies.

This release does not claim non-empty BN254 pairing algebra yet. Non-empty
pairing inputs are parsed and then fail closed with
`PrecompileBackendUnavailable` until the dedicated subgroup-check and pairing
algebra releases are admitted.

## Added

- `EVM_BN254_PAIRING_ITEM_BYTES` and `EVM_BN254_PAIRING_OUTPUT_BYTES`.
- `parse_bn254_pairing_input` for 192-byte EIP-197 tuple segmentation.
- `execute_bn254_pairing` for the admitted empty-input frame.
- `EvmPrecompilePlan::execute_bn254_pairing`.
- Dependency-free Fp2 parsing and G2 curve-equation validation for pairing
  frames.
- `EvmPrecompileImplementation::NativeBn254PairingFrame` to distinguish this
  audited frame boundary from fully unsupported cryptographic precompiles.
- `fuzz/fuzz_targets/bn254_pairing_frame.rs` for pairing frame segmentation and
  empty-input execution coverage.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.13.0`.
- `eth` publishes as `0.50.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.13.0`.
- The BN254 pairing descriptor now reports a native frame boundary rather than
  a generic backend-only descriptor.
- README, crate version matrix, EVM fork matrix, spec matrix, implementation
  plan, and release plan now document the pairing frame boundary and the
  explicit follow-up releases for non-empty pairing execution.
- The release gate is available as `scripts/release_0_50_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, allocator, or pairing backend dependency is
  added.
- Inputs larger than `EVM_PRECOMPILE_INPUT_LIMIT` are rejected before parsing.
- Non-empty inputs must be a whole number of 192-byte pairing tuples.
- G1 fields and points reuse the BN254 add/mul validation boundary.
- G2 fields are rejected when they are equal to or greater than the BN254 base
  field modulus.
- G2 points are accepted only when they are infinity or satisfy the EIP-197
  twist curve equation.
- G2 subgroup checks are not claimed in this release; therefore non-empty
  pairing execution still fails closed after frame validation.
- Empty input writes the canonical 32-byte word encoding one.
- Output buffers are checked before execution writes any result bytes.
- BN254 pairing algebra, BLAKE2F, KZG, and BLS12-381 precompiles still fail
  closed until their release slices are admitted.

## Spec References

- EIP-197 defines BN254 pairing input tuples, empty-input behavior, invalid
  input handling, and Byzantium gas:
  <https://eips.ethereum.org/EIPS/eip-197>.
- EIP-1108 defines Istanbul BN254 pairing gas reductions:
  <https://eips.ethereum.org/EIPS/eip-1108>.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`
- `cargo check -p eth --features evm-core`
- `cargo check --manifest-path fuzz/Cargo.toml`
- `cargo test -p eth --all-features --doc`
- `scripts/release_crates.py --check`
- `python3 scripts/test-release-metadata.py`

## Pentest

- Pending. Permanent report will be added as `security/pentest/v0.50.0.md`
  after the release-scope pentest and retest are complete.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.13.0`.
- `eth` publishes as `0.50.0`.
- Other support crates are unchanged and are not republished.
