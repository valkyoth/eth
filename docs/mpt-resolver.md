# MPT Resolver And Multiproof Orchestration

Status: `v0.52.6` implementation complete; awaiting pentest on the exact
implementation commit.

`MptNodeResolver` replaces positional proof-node lookup with bounded lookup by
Keccak hash. Resolver entries are strictly hash sorted, duplicate-free, fully
decoded before the first hash, and digest-verified before use.

Every resolver is permanently bound to one `MptSnapshotAnchor`. Batch
verification rejects a different anchor before traversal, preventing nodes
from independently supplied state snapshots from being mixed.

`verify_mpt_multiproof` verifies multiple expected key/value inclusions while
enforcing node, query, and cumulative verified-value byte limits. Shared nodes
are admitted and hashed once, and query order does not control node lookup.

Under the optional `alloc` feature, `MptOwnedNodeArena` owns encoded nodes,
sorts and deduplicates identical entries, enforces retained-byte limits, and
exposes only closure-scoped borrowed resolvers. `MptBatchSchedule` produces
deterministic bounded ranges and checks a cooperative cancellation boundary
before each range. Hosts may schedule ranges concurrently but must merge
results in deterministic range order.

The original ordered, allocation-free proof APIs remain available.
