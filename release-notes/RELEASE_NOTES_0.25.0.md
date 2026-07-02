# eth 0.25.0 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.25.0` publishes the first reviewed public RLP derive surface.

The codec crate now defines public `RlpEncode` and `RlpDecode` traits for
derive-generated and hand-written domains. The derive crate exports
`#[derive(RlpEncode)]` and `#[derive(RlpDecode)]` for reviewed structs, and the
primitive crate implements the traits for the core primitive domain types.

This release intentionally keeps the derive surface narrow. Generated encoders
encode structs as RLP lists in Rust declaration order. Generated decoders
require explicit `DecodeLimits`. Generics, enums, unions, transaction derives,
and implicit skipped fields remain rejected.

## Added

- Added `eth_valkyoth_codec::RlpEncode`.
- Added `eth_valkyoth_codec::RlpDecode`.
- Added `eth_valkyoth_codec::RlpDeriveError`.
- Added `eth_valkyoth_codec::checked_encoded_len_add`.
- Added public `RlpEncode` and `RlpDecode` derive macros.
- Added primitive trait implementations for chain IDs, block numbers, gas,
  nonces, timestamps, wei, addresses, hashes, `u64`, `u128`, and fixed byte
  arrays.
- Added trybuild compile-fail coverage for unsupported enum, generic, and
  ambiguous skipped-field derive inputs.
- Added round-trip tests for named, tuple, and unit structs.

## Security Notes

- Decode derives require caller-provided `DecodeLimits`; generated code does
  not bypass the bounded codec contract.
- The generated RLP encoder now treats inconsistent field encoder length
  reporting as a runtime error instead of relying on a debug-only assertion.
- Encode callers must discard output buffers after any returned error; aggregate
  derived encoders can leave a partial prefix when a later field fails.
- Skipped fields must be explicit:
  `#[eth_rlp(skip, default, reason = "...")]`.
- Transaction structs still use hand-written encoders and decoders. Public RLP
  derives do not imply fork validity, signature validity, sender recovery, or
  Ethereum transaction-state promotion.
- The EIP-7702 set-code validity gate counts repeated recovered authorities as
  sequential nonce applications. Duplicate-authority tuples with a reused nonce
  are skipped instead of over-counted.
- Generic structs, enums, and unions are rejected until their generated trait
  bounds, layout behavior, and domain semantics are explicitly designed.
- `trybuild` is admitted as a dev-only dependency for compiler diagnostic
  coverage.

## Versioning

- `eth-valkyoth-codec` publishes as `0.17.0`.
- `eth-valkyoth-primitives` publishes as `0.11.0`.
- `eth-valkyoth-derive` publishes as `0.17.0`.
- `eth-valkyoth-hash` publishes as `0.10.1`.
- `eth-valkyoth-protocol` publishes as `0.22.1`.
- `eth-valkyoth-verify` publishes as `0.14.2`.
- `eth-valkyoth-sanitization` publishes as `0.7.2`.
- `eth-valkyoth-signer` publishes as `0.7.1`.
- The facade crate publishes as `eth` `0.25.0`.
- Unchanged support crates are not republished.

## Release Gate

- External pentest must pass before tagging.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-derive
cargo test -p eth-valkyoth-codec -p eth-valkyoth-primitives -p eth-valkyoth-derive --all-features
cargo clippy -p eth-valkyoth-codec -p eth-valkyoth-primitives -p eth-valkyoth-derive --all-targets --all-features -- -D warnings
scripts/release_0_25_gate.sh
```
