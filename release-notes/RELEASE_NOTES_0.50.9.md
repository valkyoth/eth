# Release Notes - eth v0.50.9

Status: implementation complete; awaiting pentest.

## Summary

This release adds the dependency-free BN254 projective post-loop line carrier
required before non-empty EIP-197 pairing results can be admitted.

The internal fail-closed pairing path now runs the reviewed optimal-ate
non-adjacent-form loop, multiplies the Q1 and -Q2 post-loop line terms through
the projective carrier, and then runs bounded final exponentiation. Public
non-empty pairing execution still returns `PrecompileBackendUnavailable`; result
words remain scheduled for `v0.50.10`.

## Added

- Projective G2 line-state carrier for BN254 optimal-ate line multiplication.
- Sparse Fp12 line multiplication for the projective `(a, b, c)` line shape.
- Q1 and -Q2 post-loop line multiplication in the fail-closed accumulator.
- Regression proving a single EIP-197 generator tuple no longer maps to one
  after final exponentiation.
- Regression proving the generator plus negated-generator batch still maps to
  one after final exponentiation.
- Regression reconstructing the hard-coded BN254 `6u+2` NAF table back to the
  exact optimal-ate scalar.
- Regression proving bilinearity over a G1 generator double:
  `e([2]P, Q) == e(P, Q)^2`.
- BN254 pairing fuzz reachability for the complete fail-closed accumulator.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.21.0` to `0.22.0`.
- `eth` is bumped from `0.50.8` to `0.50.9`.
- The older affine BN254 line helpers are now test-only foundation checks. The
  production fail-closed pairing accumulator uses the projective carrier.
- Documentation now records that `v0.50.10` is the remaining BN254 pairing
  result-admission release.

## Security Notes

- Non-empty pairing execution remains fail-closed after bounded algebra work.
- No default crypto, allocator, bigint, or pairing backend dependency was added.
- The complete accumulator runs only over already validated G1/G2 pairing
  tuples, including G2 subgroup validation.
- The release does not claim public non-empty EIP-197 success or failure words.
- Pentest found an incorrect `6u+2` NAF table before tagging. The table was
  replaced from an independent derivation, and scalar-reconstruction plus
  bilinearity regressions now cover that failure class.

## Verification

- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`
- `cargo fmt --all --check`
- `scripts/validate-release-metadata.sh`

## Pentest

Pentest is required before tagging. The final report must be committed at
`security/pentest/v0.50.9.md` with `Status: PASS`.
