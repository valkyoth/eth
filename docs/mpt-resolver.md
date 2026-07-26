# MPT Resolver And Multiproof Orchestration

Status: `v0.52.6` release candidate; pentest findings are remediated and the
clean retest passed.

`MptNodeResolver` replaces positional proof-node lookup with bounded lookup by
Keccak hash. Resolver entries are strictly hash sorted, duplicate-free, fully
decoded before the first hash, and digest-verified before use.

Every resolver is permanently bound to one `MptSnapshotAnchor`. Batch
verification rejects a different anchor before traversal, preventing nodes
from independently supplied state snapshots from being mixed.

`verify_mpt_multiproof` verifies multiple expected key/value inclusions while
enforcing node, query, and cumulative verified-value byte limits. Shared nodes
are admitted and hashed once, and query order does not control node lookup.
An empty branch terminal is Ethereum's null value and is always reported as
absence; an empty expected byte slice cannot turn it into an inclusion.

Under the optional `alloc` feature, `MptOwnedNodeArena` bounds the raw node
count and complete retained-memory shape before allocation or hashing. It
decodes every node and preflights the complete hash budget before the first
hash, then allocation-free sorts and deduplicates identical entries. Its
retained-byte limit covers each payload vector's actual capacity and the owned
node vector's capacity; allocator bookkeeping overhead is platform-specific
and excluded. The independent node-count limit also bounds per-allocation
overhead. Fallible reservation is used for arena-owned vector growth, and
construction performs no infallible shrink/reallocation step. The arena exposes
only closure-scoped borrowed resolvers.

`MptBatchSchedule` produces deterministic bounded ranges and checks a
cooperative cancellation boundary before each range. Hosts may schedule ranges
concurrently but must merge results in deterministic range order.

The original ordered, allocation-free proof APIs remain available.
