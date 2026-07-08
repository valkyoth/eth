# eth 0.50.4 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.50.4` adds the first-party BN254 line-function foundation required before
the Miller-loop release. The release introduces deterministic G2 line
coefficient helpers for affine doubling and addition, plus G1-to-Fp12 carrier
evaluation wiring consumed by the existing fail-closed BN254 pairing tower
exerciser.

This release still does not claim Miller-loop correctness, final
exponentiation, or non-empty BN254 pairing results. Non-empty EIP-197 pairing
execution remains fail-closed.

## Added

- G2 line coefficient representation for affine BN254 twist lines.
- G2 doubling-line and addition-line helpers over already validated G2 points.
- G1-to-Fp12 line evaluation carrier used by the fail-closed tower exerciser.
- Regression tests proving tangent-line, addition-line, and vertical-line
  relations over admitted BN254 generator fixtures.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.17.0`.
- `eth` publishes as `0.50.4` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.17.0`.
- The fail-closed BN254 pairing tower exerciser now consumes the line-function
  carrier rather than the prior tuple-shape carrier.
- Dispatcher-facing `EvmPrecompilePlan` execution for ModExp and BN254 add/mul
  now requires a mutable gas meter and charges before validation or arithmetic
  is reachable, matching the pairing hardening pattern.
- The release gate is available as `scripts/release_0_50_4_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, allocator, or pairing backend dependency is
  added.
- The new arithmetic remains ordinary variable-time arithmetic over public EVM
  calldata-derived values. It must not be reused for secret scalar or private
  key operations.
- Line helpers are internal and do not expose or claim a public pairing result.
- Non-empty pairing execution still returns `PrecompileBackendUnavailable`.
- The low-level free functions remain public unmetered primitives for
  standalone tests and fuzzing. Interpreter dispatch must use the gas-gated
  `EvmPrecompilePlan` methods instead.
- Initial pentest finding F-01 was remediated by extending gas-gated plan
  execution to BN254 add/mul and ModExp. Finding F-02 was addressed with
  explicit unmetered-helper documentation and release-plan coverage.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core bn254_line`
- `cargo test -p eth-valkyoth-evm-core bn254_pairing`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo test -p eth-valkyoth-evm-core modexp`

## Pentest

- Initial review received in root `PENTEST.md`; remediation implemented.
  Permanent report will be added at `security/pentest/v0.50.4.md` after retest
  passes.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.17.0`.
- `eth` publishes as `0.50.4`.
- Other support crates are unchanged and are not republished.
