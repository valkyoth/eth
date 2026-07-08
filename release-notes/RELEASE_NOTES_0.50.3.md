# eth 0.50.3 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.50.3` adds the validated BN254 pairing tuple streaming domain required by
the Miller-loop releases. The release still keeps EIP-197 non-empty pairing
execution fail-closed, but the fail-closed path now feeds the Fp12 tower
accumulator from typed, validated `(G1, G2)` tuples instead of from a count-only
placeholder.

This release still does not claim line-function correctness, Miller-loop
correctness, final exponentiation, or non-empty BN254 pairing results. Those
remain split into later release slices so each arithmetic layer can be tested
and pentested independently.

## Added

- Internal `Bn254PairingTuple` domain carrying validated G1 and G2 points.
- Allocation-free `for_each_valid_pairing_tuple` streaming helper.
- Typed tower accumulation over validated tuple data before the existing
  non-empty fail-closed return.
- Regression tests proving tuple streaming order, first-invalid-tuple stop
  behavior, and tower accumulation over the validated tuple stream.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.16.0`.
- `eth` publishes as `0.50.3` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.16.0`.
- The roadmap now splits the remaining BN254 pairing work into smaller
  reviewed passes: line-function foundation, Miller-loop accumulation, and
  final exponentiation/non-empty execution.
- The release gate is available as `scripts/release_0_50_3_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, allocator, or pairing backend dependency is
  added.
- Tuple streaming validates input size, tuple length, G1 point validity, G2
  field elements, G2 curve membership, and G2 subgroup membership before a
  tuple reaches the tower accumulator.
- Streaming stops at the first invalid tuple and does not visit later tuples.
- Non-empty pairing execution still returns `PrecompileBackendUnavailable`.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core bn254_pairing`
- `cargo test -p eth-valkyoth-evm-core bn254_tower`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`

## Pentest

- Pending. Permanent report will be added at `security/pentest/v0.50.3.md`
  after the external pentest and retest.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.16.0`.
- `eth` publishes as `0.50.3`.
- Other support crates are unchanged and are not republished.
