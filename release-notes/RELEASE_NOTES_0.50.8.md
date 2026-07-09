# Release Notes - eth v0.50.8

Status: release candidate accepted after pentest and retest; awaiting GitHub
CI/CodeQL before tagging.

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
- The BN254 pairing fuzz target now directly exercises the post-loop Frobenius
  point helper on every parsed valid frame.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.20.0` to `0.21.0`.
- `eth` is bumped from `0.50.7` to `0.50.8`.
- The roadmap now splits the remaining BN254 pairing work into:
  - `v0.50.9`: projective/reference-aligned post-loop line carrier;
  - `v0.50.10`: public non-empty EIP-197 result admission.
- Documentation now records the discovered affine line-carrier gap explicitly
  instead of treating post-loop completion as a vague deferred item.
- The tag-triggered GitHub release workflow again runs
  `scripts/validate-release-readiness.sh "${GITHUB_REF_NAME}"`, and metadata
  regression tests now fail if the workflow is downgraded to metadata-only
  validation.

## Security Notes

- Non-empty pairing execution remains fail-closed after bounded algebra work.
- No default crypto, allocator, bigint, or pairing backend dependency was added.
- The Q1/-Q2 helper is fixed-size and operates only on already-validated public
  G2 calldata.
- The implementation deliberately does not admit the post-loop line
  multiplication that failed the generator-tuple regression.
- Tag pushes are protected by the strong pentest-readiness workflow, not only
  by the ordinary metadata check.

## Verification

- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`
- `cargo fmt --all --check`
- `python3 scripts/test-release-metadata.py`

## Pentest

The permanent report is tracked at `security/pentest/v0.50.8.md`. The initial
review found one Critical release-workflow regression and one Low fuzz
reachability gap. The remediation restored tag-time release-readiness
enforcement, added regression coverage for the workflow, and made the BN254
pairing fuzz target exercise the Q1/-Q2 Frobenius helper. Retest passed.
