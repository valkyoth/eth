# Shared Decode Session

Status: implemented in `v0.52.3`; stricter MPT proof preflight and pre-hash
charging remain assigned to `v0.52.4`.

## Security Contract

`DecodeSession` is a non-copyable, non-cloneable capability for one untrusted
decode operation. Every nested consumer receives `&mut DecodeSession`; the API
does not expose a reset operation. A reviewed `DecodeSessionPolicy` combines
the structural `DecodeLimits` with cumulative ceilings for encoded bytes, RLP
headers, hashes, hash bytes, and aggregate work.

The session tracks:

- encoded bytes scanned across structural and semantic passes;
- RLP headers and items visited;
- maximum nesting depth;
- requested allocation capacity;
- admitted proof nodes;
- hash operations and hashed bytes;
- aggregate work units across all of those domains.

Counter updates use checked arithmetic and fail closed. Composite hash and RLP
reparse charges are atomic: a rejected charge cannot commit only part of its
component counters. Named deployment starting points are intentionally invalid
until every structural limit is reviewed, and cross-limit validation rejects
item, proof-node, byte, header, depth, allocation, or hash ceilings larger than
the aggregate work ceiling.

## Parsing Model

The session-aware RLP list decoder performs one iterative structural walk. A
nested list contributes its header at the parent position and its payload as
the walk descends, so each encoded byte is charged once during structural
validation. Borrowed zero-copy models can require later semantic reparsing;
`RlpListItems::next_in_session` and `RlpList::items_in_session` charge those
actual additional passes to the same ledger.

When yielding a nested list, the cursor charges the complete outer encoded span
first. It then charges each immediate child item and header before parsing that
child to establish the nested list's compatibility metadata. A list recount can
therefore stop at a component ceiling, but cannot perform an unrecorded scan or
return a model whose observed child visits exceed the ledger.

Session-aware transaction entry points cover legacy, EIP-2930, EIP-1559,
EIP-4844, and EIP-7702 envelopes and their nested access lists, storage keys,
blob hashes, and authorization tuples. Session-aware MPT entry points cover
single nodes and proof-node syntax. The older `DecodeLimits` entry points stay
available for source compatibility; callers composing multiple untrusted
stages should use the `_in_session` APIs.

Borrowed models expose charged traversal at every reparse boundary:

- `AccessList::entries_in_session` and
  `AccessListEntries::next_in_session`;
- `AccessListStorageKeys::keys_in_session` and
  `AccessListStorageKeyItems::next_in_session`;
- `BlobVersionedHashes::hashes_in_session` and
  `BlobVersionedHashItems::next_in_session`;
- `SetCodeAuthorizationList::authorizations_in_session` and
  `SetCodeAuthorizationItems::next_in_session`;
- `MptInlineNode::node_in_session`.

The per-step cursor methods release the mutable session borrow between siblings,
which permits nested storage-key or inline-node traversal under the same
capability. The compatibility `Iterator` implementations remain available for
already trusted or independently bounded data and do not debit a session.

No current session-aware transaction or MPT parser allocates. Their borrowed
models therefore leave `allocation_capacity` at zero. A future owned conversion
must call `account_allocation_capacity(requested_capacity)` before reserving or
allocating; charging input length after allocation does not satisfy the
contract.

## Verification

- Exact structural counter oracles cover nested RLP.
- Boundary tests prove nested compatibility recounts charge every immediate
  item/header visit and fail at the item ceiling.
- Composite transaction tests prove nested access-list work remains in one
  session and allocation capacity remains zero for borrowed decoding.
- An exact EIP-7702 tuple oracle proves charged fields are consumed directly
  without an unaccounted second parse.
- MPT tests debit structural and semantic passes plus proof-node count.
- The `decode_limits` fuzz target drives every session counter, atomic RLP
  reparses, overflow edges, and post-call policy invariants.
- Strict clippy, workspace tests, `no_std` checks, and the release gate cover
  the public APIs on the supported Rust range.

`v0.52.4` adds explicit compact-path nibble and trie-value byte counters,
noncommitting complete hash-capacity checks, and MPT proof preflight before the
first proof-node Keccak operation. Every actual hash is then charged atomically
immediately before the backend call.
