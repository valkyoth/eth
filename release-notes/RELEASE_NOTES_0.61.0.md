# eth v0.61.0

Status: implementation checks passed; ready for pentest.
Publication: DEFERRED TO v0.65.0

## Charged G2 Addition

- Prague `0x0d` executes dependency-free EIP-2537 G2ADD using the v0.60.0
  public-input arithmetic: exact 512-byte input, 600 gas and 256-byte output.
- `EvmBls12G2Add` and `authorize_and_execute_bls12_g2_add` use the existing
  sealed quote/paid-capability boundary. Admission precedes curve validation,
  addition and inversion. Successful output is written once, after validation.
- Malformed paid calls consume the complete supplied gas and preserve output;
  invalid length, insufficient capacity and insufficient gas fail admission.
- Addition intentionally accepts valid on-curve non-subgroup points. Scalar
  multiplication, subgroup admission, pairing, maps and KZG remain later work.

## Verification And Maintenance

Work counters cover curve checks, additions, inversions and roots. Official
positive/rejection vectors drive paid execution; a BigUint oracle backs fuzzing.
The isolated Geth/Besu/Nethermind runner compares all 16 official G2 outcomes.
See [G2 execution](../docs/bls12-g2-addition.md) for evidence and limitations.

Rust 1.99.0 is the pinned stable gate; 1.90.0 remains the minimum. Both lockfiles
and direct dependencies were refreshed, including syn 3.0.6 and trybuild
1.0.122. Cargo tools and GitHub Actions remain checked for current versions.
See the [maintenance review](../docs/maintenance-review-2026-10-09.md).

No package is selected for crates.io publication. The facade source version is
0.61.0; support versions remain unchanged until the v0.65.0 checkpoint.
Stop for the normal incremental pentest, remediate and retest any findings,
record the permanent report, then wait for GitHub and explicit tag approval.
