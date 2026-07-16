# BLS12-381 Wire Encodings

Status: implemented in `v0.52.1`; arithmetic and curve/subgroup validation
remain fail closed.

## Scope

`eth-valkyoth-evm-core` provides dependency-free, allocation-free parsers for
the EIP-2537 inputs at precompile addresses `0x0b..=0x11`. This boundary proves
only that the bytes use the canonical Ethereum wire representation admitted by
this release.

It does not prove that a non-infinity point lies on the BLS12-381 curve, belongs
to the required subgroup, or is valid for arithmetic. Those checks and the
precompile implementations are assigned to `v0.52.2..=v0.52.9`.

## Canonical Rules

- Fp is exactly 64 big-endian bytes. The first 16 bytes must be zero and the
  remaining 48-byte integer must be strictly less than the BLS12-381 base-field
  modulus.
- Fr is exactly 32 big-endian bytes and must be strictly less than the scalar
  field modulus.
- EIP-2537 multiplication scalars are a separate exact 32-byte domain. Every
  256-bit value is accepted, including values at or above the subgroup order.
- Fp2 is encoded as `c0 || c1`, with each coefficient using canonical Fp
  encoding.
- G1 is `x || y`, for 128 bytes total. G2 is `x || y`, for 256 bytes total,
  where each coordinate is Fp2.
- Infinity is represented only by the all-zero point encoding. Non-zero top
  padding and out-of-range coordinates are rejected before an infinity value
  can be constructed.
- Fixed frames must have their exact EIP-2537 length. MSM and pairing frames
  must be non-empty exact multiples of their item length and remain subject to
  the release-wide precompile input cap.

## Security Boundary

The variable-frame parsers validate every item before returning a borrowed
frame. Iterators reparse already-validated bounded items so the public type can
remain allocation-free and borrow the caller's input. Callers may therefore
treat successful iteration as deterministic, but must not treat a parsed G1 or
G2 value as curve-valid until a later validation API returns an explicit proof
state.

Public precompile inputs are not secret material. If these field types are ever
reused for secret-bearing key operations, that separate API must define a
sanitization and constant-time contract rather than inheriting this wire-parser
boundary implicitly.

## Source And Verification

The implementation was checked against the current official
[EIP-2537](https://eips.ethereum.org/EIPS/eip-2537) encoding and input rules on
2026-07-16.

Verification includes boundaries for zero, `p - 1`, `p`, non-zero top padding,
`q - 1`, `q`, full-width multiplication scalars, coefficient order, unique
infinity encodings, exact and partial frames, malformed later items, canonical
round trips, iterator behavior, and fuzzing across all seven input parsers.
