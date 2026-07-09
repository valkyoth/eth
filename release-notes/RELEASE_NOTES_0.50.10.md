# Release Notes - eth v0.50.10

Status: implementation complete; awaiting pentest.

## Summary

This release admits non-empty EIP-197 BN254 pairing result words in the
dependency-free `eth-valkyoth-evm-core` precompile path.

The pairing precompile now keeps the existing bounded input validation, G1/G2
curve checks, G2 subgroup checks, tuple streaming, Miller accumulation,
projective post-loop line carrier, and final exponentiation, then writes the
canonical 32-byte big-endian zero or one word for valid non-empty frames.

## Added

- Non-empty BN254 pairing output admission for `execute_bn254_pairing`.
- go-ethereum `bn256Pairing.json` positive and negative result vectors:
  `two_point_match_2` and `one_point`.
- Public execution regression proving the generator tuple returns zero.
- Public execution regression proving the generator plus negated-generator
  batch returns one.
- Fuzz assertion that every successfully executed valid frame returns only a
  canonical zero or one output word.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.22.0` to `0.23.0`.
- `eth` is bumped from `0.50.9` to `0.50.10`.
- Documentation now records BN254 pairing result admission instead of a
  fail-closed non-empty result boundary.

## Security Notes

- The release does not add default crypto, allocator, bigint, or pairing
  backend dependencies.
- Gas charging, tuple segmentation, input size limits, G1/G2 validation, and
  G2 subgroup validation are unchanged.
- Output buffers are still written only after successful validation and
  arithmetic.
- Remaining cryptographic precompiles stay fail-closed until their own audited
  release slices.

## Verification

- `cargo test -p eth-valkyoth-evm-core bn254_pairing`
- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`
- `cargo fmt --all --check`
- `scripts/validate-release-metadata.sh`
- `scripts/release_crates.py --check`

## Pentest

Pentest is required before tagging. The final report must be committed at
`security/pentest/v0.50.10.md` with `Status: PASS`.
