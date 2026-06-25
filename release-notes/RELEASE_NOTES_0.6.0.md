# eth 0.6.0 Release Notes

Status: in development

## Summary

`0.6.0` is the RLP scalar decoder milestone. The first implementation pass
refreshes dependencies and release tooling evidence before parser code lands.

## Included So Far

- Updated `quote` from `1.0.45` to `1.0.46`.
- Updated optional `sanitization` support from `1.1.1` to `1.2.2`.
- Confirmed GitHub tooling is current with `scripts/check_latest_tools.sh`;
  no workflow pin changes were required.
- Added the `scripts/release_0_6_gate.sh` release gate.
- Marked `eth-valkyoth-derive`, `eth-valkyoth-sanitization`, and `eth` for
  `0.6.0` publication.
- Left codec, primitive, protocol, verification, signer, EVM, RPC, Reth, and
  testkit packages on their previously published versions for this first pass.

## Still Required Before Tag

- RLP scalar item model.
- Short and long string handling.
- Exact-consumption and trailing-data rejection.
- Malformed length tests.
- External pentest and permanent `security/pentest/v0.6.0.md` report.

## Verification

```bash
scripts/checks.sh
scripts/release_0_6_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo audit
```
