# MPT Node Decoding And Inclusion Proofs

Status: v0.33.0 adds account and storage inclusion proof verification on top
of the v0.32.0 transaction and receipt proof verifier.

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

`verify_transaction_inclusion` and `verify_receipt_inclusion` verify that
caller-provided encoded transaction or receipt bytes are present at
`rlp(transaction_index)` under a trusted root. The APIs use distinct
`TransactionTrieRoot` and `ReceiptTrieRoot` newtypes so those domains cannot be
silently substituted for raw `B256` values. Proof walking hashes each consumed
encoded proof node through the caller-provided `Keccak256` trait boundary and
uses one shared `DecodeAccumulator` for the traversal.

`verify_account_inclusion` and `verify_storage_inclusion` verify that
caller-provided encoded account or storage value bytes are present at
`keccak256(address)` or `keccak256(slot_key)` under a trusted root. The APIs
use distinct `AccountTrieRoot`, `StorageTrieRoot`, and `StorageSlotKey`
newtypes. They prove byte-exact trie membership only; they do not decode
account fields, prove that a storage root belongs to a specific account, or
interpret the included storage scalar.

The proof APIs distinguish malformed or incomplete proof inputs from
well-formed absence proofs and wrong-root/value-mismatch proofs. They reject
unused trailing proof nodes after a successful match. The proof walker is
iterative and additionally capped by `MAX_PROOF_WALK_DEPTH`, independent of
caller-selected `DecodeLimits`, so large `max_proof_nodes` deployments cannot
turn proof validation into unbounded native stack growth.

This release verifies trie inclusion only. It does not prove that a trusted
root came from a canonical header, decode or execute the included transaction,
validate receipt semantics, decode account state, or compose account and
storage proofs into full JSON-RPC `eth_getProof` semantics.

Source trail:

- `spec-lock.toml` pins `ethereum/execution-specs` at
  `26f47861dfbbd6b33d6a050ece5dae0ee4611285`.
- The pinned `src/ethereum/merkle_patricia_trie.py` source describes
  `LeafNode`, `ExtensionNode`, `BranchNode`, hex-prefix compact paths, and the
  hash-or-inline child-reference boundary.
