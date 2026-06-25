# eth 0.6.0 Release Notes

Status: implementation complete; pentest passed; pending GitHub green before tag

## Summary

`0.6.0` is the RLP scalar decoder milestone. It starts with dependency and
tooling refreshes, then adds canonical scalar RLP byte-string decoding.

## Included

- Updated `quote` from `1.0.45` to `1.0.46`.
- Updated optional `sanitization` support from `1.1.1` to `1.2.2`.
- Confirmed GitHub tooling is current with `scripts/check_latest_tools.sh`;
  no workflow pin changes were required.
- Added borrowed RLP scalar item decoding for single-byte, short-string, and
  long-string forms.
- Added exact-consumption enforcement for scalar decoder entry points.
- Added malformed length, trailing-data, list-prefix, canonical-form, and
  budget enforcement tests.
- Added official scalar RLP example fixtures and long-length overflow coverage.
- Added a fuzz target for exact and partial RLP scalar decoding paths.
- Refreshed pinned official Ethereum source revisions for v0.6.0 parser work.
- Addressed v0.6.0 pentest findings by gating codec test fixtures, renaming
  ambiguous decode-limit and partial-decoder APIs, adding hardened-only
  sanitization builds, and requiring explicit trusted-RPC acknowledgment.
- Added the `scripts/release_0_6_gate.sh` release gate.
- Marked `eth-valkyoth-codec`, `eth-valkyoth-derive`,
  `eth-valkyoth-sanitization`, `eth-valkyoth-rpc`, and `eth` for `0.6.0`
  publication.
- Left primitive, protocol, verification, signer, EVM, Reth, and testkit
  packages on their previously published versions for this pass.

## Verification

```bash
scripts/checks.sh
scripts/release_0_6_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo audit
cargo deny --manifest-path fuzz/Cargo.toml check
cargo check --manifest-path fuzz/Cargo.toml
scripts/release_crates.py --dry-run --skip-checks --yes
scripts/validate-release-readiness.sh v0.6.0
```
