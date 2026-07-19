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
until every structural limit is reviewed, and cross-limit validation rejects a
component ceiling larger than the aggregate work ceiling.

## Parsing Model

The session-aware RLP list decoder performs one iterative structural walk. A
nested list contributes its header at the parent position and its payload as
the walk descends, so each encoded byte is charged once during structural
validation. Borrowed zero-copy models can require later semantic reparsing;
`RlpListItems::next_in_session` and `RlpList::items_in_session` charge those
actual additional passes to the same ledger.

Session-aware transaction entry points cover legacy, EIP-2930, EIP-1559,
EIP-4844, and EIP-7702 envelopes and their nested access lists, storage keys,
blob hashes, and authorization tuples. Session-aware MPT entry points cover
single nodes and proof-node syntax. The older `DecodeLimits` entry points stay
available for source compatibility; callers composing multiple untrusted
stages should use the `_in_session` APIs.

No current session-aware transaction or MPT parser allocates. Their borrowed
models therefore leave `allocation_capacity` at zero. A future owned conversion
must call `account_allocation_capacity(requested_capacity)` before reserving or
allocating; charging input length after allocation does not satisfy the
contract.

## Verification

- Exact structural counter oracles cover nested RLP.
- Composite transaction tests prove nested access-list work remains in one
  session and allocation capacity remains zero for borrowed decoding.
- MPT tests debit structural and semantic passes plus proof-node count.
- The `decode_limits` fuzz target drives every session counter, atomic RLP
  reparses, overflow edges, and post-call policy invariants.
- Strict clippy, workspace tests, `no_std` checks, and the release gate cover
  the public APIs on the supported Rust range.

`v0.52.4` builds on this ledger to preflight complete proof work and reject
before each Keccak operation.
