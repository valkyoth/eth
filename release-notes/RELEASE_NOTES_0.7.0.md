# eth 0.7.0 Release Notes

Status: in development

## Summary

`0.7.0` is the RLP list decoder milestone. It extends the bounded RLP decoder
from scalar byte strings to canonical short and long lists, including nested
payload traversal under explicit resource limits.

## Included So Far

- Added bounded canonical RLP list decoding for short-list and long-list forms.
- Added exact-consumption and partial list-decoder entry points.
- Added nested list traversal with explicit item-count, nesting-depth, and
  cumulative item-budget enforcement.
- Added no-allocation immediate-child iteration through `RlpList::items`,
  `RlpListItems`, and `RlpItem`.
- Split scalar and list RLP tests into separate modules to keep parser test
  files below the 500-line limit.
- Added official nested-list example coverage and a deeper canonical nesting
  regression test for `NestingTooDeep`.
- Added malformed list tests for missing payloads, missing length bytes,
  leading-zero lengths, long-form lengths for short payloads, and length
  overflow.
- Added fuzz coverage for exact and partial RLP list decoding paths, including
  immediate child iteration on successfully decoded lists.
- Refreshed pinned official Ethereum source revisions for v0.7.0 parser work.
- Confirmed dependency and GitHub tooling currency for the v0.7.0 start slice.
- Corrected the public crate license metadata and repository license files to
  `MIT OR Apache-2.0`, matching the `sanitization` project license model.
- Marked `eth-valkyoth-codec` and `eth` for `0.7.0` publication.
- Left primitives, protocol, verification, derive, sanitization, RPC, signer,
  EVM, Reth, and testkit packages on their previously published versions for
  this pass.

## Still Required Before Tag

- External pentest and permanent `security/pentest/v0.7.0.md` report.

## Verification

```bash
scripts/checks.sh
scripts/release_0_7_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo audit
```
