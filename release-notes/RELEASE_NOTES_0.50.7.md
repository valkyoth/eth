# Release Notes - eth v0.50.7

Status: release candidate accepted after pentest and retest; awaiting GitHub
CI/CodeQL before tagging.

## Summary

This release adds the first-party BN254 final-exponentiation foundation to
`eth-valkyoth-evm-core` while deliberately keeping non-empty EIP-197 pairing
execution fail-closed.

During implementation, the final exponentiation work exposed that the current
Miller accumulator still needs the optimal-ate post-loop Frobenius/addition
line terms before public non-empty pairing results can be admitted. The roadmap
now tracks that explicitly in `v0.50.8`, with final result admission in
`v0.50.9`.

## Added

- `eth-valkyoth-evm-core` now has a dedicated `bn254_final` module for bounded
  final exponentiation over the existing Fp12 tower.
- The exponent is fixed at `(p^12 - 1) / q`, encoded as a constant little-endian
  limb schedule so the loop count is statically bounded.
- The non-empty BN254 pairing path now validates the frame, runs the Miller
  accumulator, exercises final exponentiation, and still returns
  `PrecompileBackendUnavailable` without writing output.
- Pairing tests now cover final exponentiation of `Fp12::ONE` and an admitted
  inverse-batch Miller accumulator that maps to one.
- The BN254 economics harness now has an ignored release-mode benchmark for the
  final-exponentiation path that v0.50.7 executes before failing closed.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.19.0` to `0.20.0`.
- `eth` is bumped from `0.50.6` to `0.50.7`.
- The release plan splits the remaining BN254 pairing work into:
  - `v0.50.8`: optimal-ate post-loop Frobenius/addition line terms;
  - `v0.50.9`: public non-empty EIP-197 result admission with official and
    differential vectors.
- Documentation now says final exponentiation is implemented, but non-empty
  pairing success is not claimed yet.
- `docs/bn254-pairing-economics.md` records v0.50.7 final-exponentiation timing
  evidence alongside the existing v0.50.6 Miller-loop timing.

## Security Notes

- Non-empty pairing execution remains fail-closed after bounded algebra work.
- No default crypto, allocator, bigint, or pairing backend dependency was added.
- The final exponentiation path is fixed-size and does not introduce an
  unbounded CPU path independent of input length, gas, or release limits.

## Verification

- `cargo test -p eth-valkyoth-evm-core bn254_pairing`
- `cargo test -p eth-valkyoth-evm-core bn254_miller`
- `cargo test --release -p eth-valkyoth-evm-core bn254_pairing_final_exponentiation_wall_time_budget_smoke -- --ignored --nocapture`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`

## Pentest

The permanent report is tracked at `security/pentest/v0.50.7.md`. The initial
review found one Low evidence gap: final exponentiation was live in the
fail-closed path but not represented in the release-mode benchmark evidence.
The remediation added an ignored release benchmark and recorded the timing in
`docs/bn254-pairing-economics.md`. Retest passed.
