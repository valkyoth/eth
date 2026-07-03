# MPT Node Decoding

Status: v0.31.0 adds bounded syntactic Merkle Patricia Trie node decoding.

`eth-valkyoth-verify` exposes `decode_mpt_node` for one canonical RLP trie
node and `decode_mpt_proof_nodes` for a caller-provided list of encoded proof
nodes. Both APIs require explicit `DecodeLimits`.

The decoder supports the three Ethereum trie node forms described by the
pinned `execution-specs` `src/ethereum/merkle_patricia_trie.py` source:

- branch nodes: sixteen child references plus one scalar value slot;
- extension nodes: compact hex-prefix path plus one required child reference;
- leaf nodes: compact hex-prefix path plus one scalar value.

The compact path decoder checks that the flag byte is present, that the high
nibble uses only the extension/leaf and odd/even bits, and that even paths use
zero padding. The child-reference decoder accepts empty branch slots, 32-byte
hash references, and inline RLP list references. Required extension children
must not be empty. Inline child lists are shape-checked eagerly under
`MPT_INLINE_REFERENCE_DEPTH_LIMIT` and must be shorter than
`MPT_MAX_INLINE_REFERENCE_BYTES` encoded bytes, matching Ethereum's canonical
hash-or-inline boundary. A decoded parent therefore cannot hide a malformed or
noncanonical embedded node behind a lazy accessor.

`decode_mpt_proof_nodes` is intentionally allocation-free, but it still accounts
each proof node against the cumulative proof-node budget and each encoded node
byte length against the cumulative allocation budget. This keeps malformed
proof inputs from bypassing the same byte, item, nesting, and proof-count
limits used elsewhere in the crate.

This release is not proof verification. It does not compute Keccak-256 trie
roots, compare roots against headers, verify key-path membership or absence,
select the transaction/receipt/account/storage/withdrawal trie domain, or
interpret account/storage values. Those are scheduled as separate proof
milestones in `docs/RELEASE_PLAN.md`.

Source trail:

- `spec-lock.toml` pins `ethereum/execution-specs` at
  `26f47861dfbbd6b33d6a050ece5dae0ee4611285`.
- The pinned `src/ethereum/merkle_patricia_trie.py` source describes
  `LeafNode`, `ExtensionNode`, `BranchNode`, hex-prefix compact paths, and the
  hash-or-inline child-reference boundary.
