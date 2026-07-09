# Release Notes - eth v0.50.8

Status: implementation ready; awaiting pentest before tagging.

## Summary

This release adds the first-party BN254 G2 Frobenius point foundation required
for Ethereum's optimal-ate post-loop pairing terms while keeping non-empty
EIP-197 pairing execution fail-closed.

During implementation, the post-loop Q1/-Q2 points were tested against the
current affine line-carrier shortcut. That combination incorrectly maps the
single EIP-197 generator tuple to one after final exponentiation. The release
therefore admits only the bounded, KAT-backed Frobenius point helpers and moves
the line-carrier refactor to `v0.50.9`, with non-empty result admission moved
to `v0.50.10`.

## Added

- `eth-valkyoth-evm-core` now has first-party G2 Frobenius helpers for the
  optimal-ate post-loop Q1 and -Q2 points.
- Focused KATs cover generator Q1 and -Q2 point mapping, with expected
  coordinates computed independently from the BN254 field modulus and twist
  factor.
- The fail-closed non-empty Miller path exercises the admitted Q1/-Q2 helper
  without multiplying the post-loop lines into the accumulator.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.20.0` to `0.21.0`.
- `eth` is bumped from `0.50.7` to `0.50.8`.
- The roadmap now splits the remaining BN254 pairing work into:
  - `v0.50.9`: projective/reference-aligned post-loop line carrier;
  - `v0.50.10`: public non-empty EIP-197 result admission.
- Documentation now records the discovered affine line-carrier gap explicitly
  instead of treating post-loop completion as a vague deferred item.

## Security Notes

- Non-empty pairing execution remains fail-closed after bounded algebra work.
- No default crypto, allocator, bigint, or pairing backend dependency was added.
- The Q1/-Q2 helper is fixed-size and operates only on already-validated public
  G2 calldata.
- The implementation deliberately does not admit the post-loop line
  multiplication that failed the generator-tuple regression.

## Verification

- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo fmt --all --check`

## Next

Run the 0.50.8 pentest against this exact implementation commit before the
permanent report is written.
