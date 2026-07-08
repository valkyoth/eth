# eth 0.50.3 Release Notes

Status: release-ready; pentest passed after remediation and retest.

`0.50.3` adds the validated BN254 pairing tuple streaming domain required by
the Miller-loop releases. The release still keeps EIP-197 non-empty pairing
execution fail-closed, but the fail-closed path now feeds the Fp12 tower
accumulator from typed, validated `(G1, G2)` tuples instead of from a count-only
placeholder. The plan-level BN254 pairing execution path now accepts the gas
meter directly and charges the plan cost on every call before validation work is
reachable, so there is no reusable gas-payment token to replay.

This release still does not claim line-function correctness, Miller-loop
correctness, final exponentiation, or non-empty BN254 pairing results. Those
remain split into later release slices so each arithmetic layer can be tested
and pentested independently.

## Added

- Internal `Bn254PairingTuple` domain carrying validated G1 and G2 points.
- Allocation-free `for_each_valid_pairing_tuple` streaming helper.
- Atomic charge-and-execute BN254 pairing plan method.
- Typed tower accumulation over validated tuple data before the existing
  non-empty fail-closed return.
- Regression tests proving tuple streaming order, first-invalid-tuple stop
  behavior, tower accumulation over the validated tuple stream, and repeated
  plan execution charging on every call.

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
- BN254 pairing plan execution charges the supplied gas meter before the parser,
  G2 subgroup check, or tower exerciser are reachable through the
  dispatcher-facing method. It does not expose a reusable payment object.
- Non-empty pairing execution still returns `PrecompileBackendUnavailable`.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core bn254_pairing`
- `cargo test -p eth-valkyoth-evm-core bn254_tower`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`

## Pentest

- Initial pentest passed with no Critical/High/Medium findings.
- A Low future-integration finding requested a type-enforced gas-charge boundary
  before BN254 pairing validation is reachable from dispatcher-style code.
- Retest found that the first token-based remediation was replayable because the
  token was `Copy`. The token design was removed. `EvmPrecompilePlan` now
  charges the supplied `EvmGasMeter` inside `execute_bn254_pairing` on every
  call before it reaches pairing validation work.
- Permanent report: `security/pentest/v0.50.3.md`.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.16.0`.
- `eth` publishes as `0.50.3`.
- Other support crates are unchanged and are not republished.
