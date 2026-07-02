# eth 0.26.0 Release Notes

Status: implementation complete; external pentest findings remediated; clean
retest complete; waiting for final GitHub checks before tagging

## Summary

`0.26.0` adds the first-party EIP-712 typed-data hashing pipeline.

The verify crate can now compute canonical `encodeType`, bounded `encodeData`,
`hashStruct`, EIP-712 domain separators, and final
`keccak256("\x19\x01" || domainSeparator || hashStruct(message))` digests from
caller-provided borrowed descriptors and values. The implementation remains
`no_std`, no-allocation, and keeps concrete Keccak backends outside the default
dependency graph.

This release intentionally does not parse JSON typed-data documents. Callers
must parse and review JSON-RPC payloads at their own boundary, then pass bounded
descriptors and values into the encoder.

## Added

- Added `Eip712StructType`, `Eip712Field`, `Eip712Value`, and
  `Eip712ValueKind`.
- Added `Eip712DomainData` for admitted EIP-712 domain separator fields.
- Added `Eip712EncodeError`.
- Added `encode_eip712_type`.
- Added `encode_eip712_data`.
- Added `eip712_type_hash`.
- Added `eip712_hash_struct`.
- Added `eip712_domain_separator`.
- Added `eip712_typed_data_signing_digest`.
- Added official EIP-712 Ether Mail signer recovery coverage.
- Added adversarial coverage for missing values, type mismatches, arrays, and
  fixed bytes.
- Added `eip712_typed` fuzz-target build coverage for bounded type graphs and
  value hashing.

## Fixed During Pentest

- Reserved EIP-712 atomic type names such as `address`, `bool`, `bytes`,
  `string`, `uintN`, `intN`, and `bytesN` are now rejected as custom struct
  names before type-graph traversal.
- Array dimensionality is now counted against the same recursion limit as
  struct nesting.
- The recursion-limit boundary now rejects at the documented maximum instead of
  admitting one extra level.
- `Eip712DomainData::chain_id` now documents that callers requiring EIP-712
  replay protection should reject `ChainId(0)` before computing a domain
  separator.

## Security Notes

- The typed-data encoder accepts borrowed descriptors only; it does not parse
  JSON or allocate.
- All recursive type and value hashing is bounded by an explicit recursion
  limit.
- Dynamic `bytes` and `string` values are hashed before insertion into
  `encodeData`, matching EIP-712.
- Arrays are hashed as concatenated encoded element words.
- Domain separators are computed from admitted optional domain fields in the
  EIP-712 field order.
- Concrete Keccak-256 implementations remain caller-provided and must compute
  Ethereum Keccak-256, not FIPS SHA3-256.

## Versioning

- `eth-valkyoth-verify` publishes as `0.15.0`.
- The facade crate publishes as `eth` `0.26.0`.
- Unchanged support crates are not republished.

## Release Gate

- External pentest must pass before tagging.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-verify --all-features
cargo clippy -p eth-valkyoth-verify --all-targets --all-features -- -D warnings
scripts/release_0_26_gate.sh
```
