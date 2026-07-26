# Release Notes - eth v0.52.6

Status: release candidate; pentest passed.

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
  decode/hash/allocation session preflight, actual payload-capacity and metadata
  accounting, allocation-free sorting/deduplication, and no infallible shrink
  path.
- Deterministic `MptBatchSchedule` and cooperative `MptCancellation`.
- Shared/reordered, missing, duplicate, unrelated snapshot, output overflow,
  empty-terminal absence, malformed and over-budget pre-hash, deduplication,
  retained-capacity, and cancellation tests.
- Resolver fuzz admission and traversal coverage.

## Security Properties

- All encoded nodes are locally canonical before attacker bytes reach Keccak.
- Claimed hashes are verified once at resolver admission.
- Hash-addressed short children are rejected when canonical RLP requires an
  inline reference, matching the ordered-proof kernel.
- Empty branch terminals always prove absence, including through hashed and
  inline resolver children.
- Snapshot mismatch fails before traversal.
- Sorted unique entries provide logarithmic lookup without allocation.
- Node, query, hash-work, decode-work, retained-byte, and verified-output limits
  are explicit.
- Owned arenas reject raw node-count and retained-capacity overflow before
  hashing, use fallible reservation, and perform no infallible shrink step.
- Host scheduling does not alter deterministic merge order or consensus proof
  semantics.

## Versioning

- `eth-valkyoth-verify` advances from `0.26.0` to `0.27.0`.
- `eth` advances from `0.52.5` to `0.52.6`.
- All other support crates remain unchanged.

## Pentest

Independent review and iterative retesting found and closed canonical
short-child parity, empty branch inclusion, resolver error-path coverage, raw
arena admission, retained-capacity accounting, and infallible allocation-path
issues. No Critical, High, Medium, or Low finding remains. The permanent
exact-commit report is recorded at `security/pentest/v0.52.6.md`.
