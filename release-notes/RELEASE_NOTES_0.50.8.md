# Release Notes - eth v0.50.8

Status: released after pentest, retest, local release gate, GitHub CI, and
CodeQL.

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
- The local release gate continues to run
  `scripts/validate-release-readiness.sh v0.50.8` before tagging.
- After release, the GitHub release workflow was returned to a manual
  metadata-only check because post-tag readiness is expected to fail once the
  tag already exists. The versioned local release gate remains the authoritative
  readiness check.

## Security Notes

- Non-empty pairing execution remains fail-closed after bounded algebra work.
- No default crypto, allocator, bigint, or pairing backend dependency was added.
- The Q1/-Q2 helper is fixed-size and operates only on already-validated public
  G2 calldata.
- The implementation deliberately does not admit the post-loop line
  multiplication that failed the generator-tuple regression.
- Tag creation is protected by the strong local pentest-readiness gate before
  the tag is pushed; GitHub tag workflows are not treated as the release
  authority.

## Verification

- `cargo test -p eth-valkyoth-evm-core bn254`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bn254_pairing_frame -- -D warnings`
- `cargo fmt --all --check`
- `python3 scripts/test-release-metadata.py`

## Pentest

The permanent report is tracked at `security/pentest/v0.50.8.md`. The initial
review found one Critical release-workflow regression and one Low fuzz
reachability gap. The remediation restored local release-readiness enforcement,
added regression coverage for the workflow policy, and made the BN254 pairing
fuzz target exercise the Q1/-Q2 Frobenius helper. Retest passed. The final
workflow policy keeps readiness in the versioned local gate to avoid a known
post-tag false failure.
