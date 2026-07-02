# eth 0.26.1 Release Notes

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

## Summary

`0.26.1` adds an optional EIP-712 JSON-RPC typed-data parser boundary.

The default crate graph remains `no_std` and does not parse JSON. Applications
that want first-party JSON typed-data handling can opt into
`eth = { version = "0.26", features = ["eip712-json"] }`.

## Added

- Added `eth-valkyoth-verify` feature `json`.
- Added facade feature `eip712-json`.
- Added `Eip712JsonLimits`.
- Added `Eip712JsonError`.
- Added `eip712_json_typed_data_signing_digest`.
- Added duplicate JSON object-key rejection before type maps are admitted.
- Added explicit parser limits for input bytes, type count, field count, array
  length, string length, dynamic bytes length, and recursion depth.
- Added Ether Mail JSON fixture coverage.
- Added adversarial fixtures for duplicate keys, missing primary type,
  full-width signed decimal integer handling, parser limits, malformed hex,
  duplicate type fields, fixed-array length mismatch, and domain validation.

## Security Notes

- The JSON parser is disabled by default and requires `std`.
- The parser rejects `ChainId(0)` in EIP-712 domains.
- The parser rejects unknown EIP-712 domain fields.
- The parser rejects duplicate fields inside a type definition.
- The parser validates the shape of `types.EIP712Domain` even though domain
  separator construction is driven by the parsed `domain` object.
- Duplicate-key detection uses a set-backed check with an independent JSON
  object-width guard so raising `max_input_bytes` does not admit unbounded
  wide-object duplicate scans.
- `max_bytes_value` applies to dynamic `bytes`; fixed `bytes1` through
  `bytes32` values remain bounded by their EIP-712 type width.
- Parsed data is fed into the same typed-data hashing boundary introduced in
  `0.26.0`.
- Concrete Keccak-256 implementations remain caller-provided.

## Dependency Review

- `serde 1.0.228` and `serde_json 1.0.150` were checked as current registry
  versions before admission.
- Both dependencies are behind the optional `json`/`eip712-json` feature path.
- The new transitive `zmij 1.0.21` dependency comes through `serde_json`; the
  release review checked its maintainer lineage, license, build script shape,
  and current `cargo deny`/`cargo audit` results.

## Versioning

- `eth-valkyoth-verify` publishes as `0.16.0`.
- The facade crate publishes as `eth` `0.26.1`.
- Unchanged support crates are not republished.

## Release Gate

- External pentest and retest passed before tagging.
- Final GitHub checks must pass on the pentest report commit before tagging.

## Verification

```bash
cargo test -p eth-valkyoth-verify --features json
cargo clippy -p eth-valkyoth-verify --all-targets --all-features -- -D warnings
scripts/release_0_26_1_gate.sh
```
