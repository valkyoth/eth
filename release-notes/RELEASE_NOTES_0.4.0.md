# eth 0.4.0 Release Notes

Status: implementation complete; pending external pentest input

## Summary

`0.4.0` establishes the stable error model for protocol operations and starts
independent crate versioning for crates.io releases.

## Included

- Added stable `code()`, `message()`, category, and `Display` behavior for
  codec, protocol, fork, feature, resource-exhaustion, and verification errors.
- Added optional `std::error::Error` implementations behind existing `std`
  features.
- Kept error payloads free of input bytes, keys, signatures, and other
  secret-bearing data.
- Added tests for error stability and formatting.
- Re-exported stable error types through `eth::error`.
- Replaced lockstep support-crate publishing with per-crate release planning.
- Addressed v0.4.0 pentest findings for typestate token forgery, address
  comparison timing, allocation-budget API misuse, sanitization skip
  acknowledgement, decode-limit naming, typed-envelope transaction
  classification, best-effort sanitization visibility, and fuzz bootstrap.
- Marked `eth-valkyoth-codec`, `eth-valkyoth-primitives`,
  `eth-valkyoth-protocol`, `eth-valkyoth-verify`, `eth-valkyoth-derive`,
  `eth-valkyoth-sanitization`, and `eth` for `0.4.0` publication.
- Marked `eth-valkyoth-signer` for dependency-only `0.3.1` publication.
- Left unchanged EVM, RPC, Reth, and testkit support crates on `0.3.0`.

## Verification

```bash
scripts/checks.sh
scripts/release_0_4_gate.sh
scripts/release_crates.py --check
cargo deny check
cargo audit
```
