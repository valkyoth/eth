# Release Notes - eth v0.52.6

Status: implementation complete; pentest required before release.

## Summary

This release adds bounded hash-addressed MPT resolution and snapshot-bound
multiproof orchestration without weakening the allocation-free proof kernel.

## Added

- `MptNodeResolver` with strict hash ordering, duplicate rejection, complete
  pre-hash syntax admission, and digest validation.
- `MptSnapshotAnchor` binding every resolver batch to one trusted root.
- `MptBatchQuery` and `verify_mpt_multiproof` with query and cumulative output
  byte limits.
- Canonical empty branch terminals are treated as absence in both planned and
  executed ordered/resolver-backed proof walks.
- Optional `alloc` `MptOwnedNodeArena` with pre-hash raw-node limits,
  decode/hash/allocation session preflight, exact-size payload ownership,
  retained payload-and-metadata accounting, and identical-node deduplication.
- Deterministic `MptBatchSchedule` and cooperative `MptCancellation`.
- Shared/reordered, missing, duplicate, unrelated snapshot, output overflow,
  empty-terminal absence, malformed and over-budget pre-hash, deduplication,
  retained-capacity, and cancellation tests.
- Resolver fuzz admission and traversal coverage.

## Security Properties

- All encoded nodes are locally canonical before attacker bytes reach Keccak.
- Claimed hashes are verified once at resolver admission.
- Snapshot mismatch fails before traversal.
- Sorted unique entries provide logarithmic lookup without allocation.
- Node, query, hash-work, decode-work, retained-byte, and verified-output limits
  are explicit.
- Host scheduling does not alter deterministic merge order or consensus proof
  semantics.

## Versioning

- `eth-valkyoth-verify` advances from `0.26.0` to `0.27.0`.
- `eth` advances from `0.52.5` to `0.52.6`.
- All other support crates remain unchanged.

## Pentest

The permanent exact-commit report will be recorded at
`security/pentest/v0.52.6.md` before tagging.
