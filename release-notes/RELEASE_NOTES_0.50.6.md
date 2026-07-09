# eth 0.50.6 Release Notes

Status: release candidate; pentest clean.

`0.50.6` adds sparse BN254 Miller-loop line-factor multiplication evidence.
The Miller loop now multiplies line factors through the dedicated
`Fp12::mul_by_fp6` path instead of constructing a dense Fp12 carrier with a
zero Fp6 lane and calling generic `Fp12::mul`.

This release still does not claim final exponentiation or non-empty BN254
pairing results. Non-empty EIP-197 pairing execution remains fail-closed with
`PrecompileBackendUnavailable`.

## Added

- Sparse Fp12-by-Fp6 multiplication path for BN254 Miller line factors.
- Regression test proving sparse line-factor multiplication matches the dense
  carrier for generator doubling and addition lines.
- Ignored release evidence benchmark for `miller_loop_tuple`.
- `docs/bn254-pairing-economics.md` with the release-mode benchmark command,
  local evidence, and the next gate before non-empty pairing execution.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.19.0`.
- `eth` publishes as `0.50.6` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.19.0`.
- README and specification docs now describe sparse Miller-loop multiplication
  as implemented and leave final exponentiation for `v0.50.7`.
- The release gate is available as `scripts/release_0_50_6_gate.sh`.

## Security Notes

- No default BN254, bigint, crypto, allocator, or pairing backend dependency is
  added.
- The new arithmetic remains ordinary variable-time arithmetic over public EVM
  calldata-derived values. It must not be reused for secret scalar or private
  key operations.
- Non-empty pairing execution still returns `PrecompileBackendUnavailable`.
- Complete non-empty pairing success remains blocked on final exponentiation,
  official vectors, differential vectors, updated performance evidence, and a
  pentest gate.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core bn254_miller`
- `cargo test --release -p eth-valkyoth-evm-core miller_loop_wall_time_budget_smoke -- --ignored --nocapture`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`

## Pentest

- Initial review received in root `PENTEST.md`; no blocking findings were
  reported. The permanent release report is recorded at
  `security/pentest/v0.50.6.md`.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.19.0`.
- `eth` publishes as `0.50.6`.
- Other support crates are unchanged and are not republished.
