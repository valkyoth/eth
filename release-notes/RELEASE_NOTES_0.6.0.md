# eth 0.6.0 Release Notes

Status: in development

## Summary

`0.6.0` is the RLP scalar decoder milestone. It starts with dependency and
tooling refreshes, then adds canonical scalar RLP byte-string decoding.

## Included So Far

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
- Added a fuzz target for exact and prefix RLP scalar decoding paths.
- Refreshed pinned official Ethereum source revisions for v0.6.0 parser work.
- Added the `scripts/release_0_6_gate.sh` release gate.
- Marked `eth-valkyoth-codec`, `eth-valkyoth-derive`,
  `eth-valkyoth-sanitization`, and `eth` for `0.6.0` publication.
- Left primitive, protocol, verification, signer, EVM, RPC, Reth, and testkit
  packages on their previously published versions for this pass.

## Still Required Before Tag

- External pentest and permanent `security/pentest/v0.6.0.md` report.

## Verification

```bash
scripts/checks.sh
scripts/release_0_6_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo audit
```
