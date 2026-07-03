# eth 0.31.0 Release Notes

Status: pentest passed; waiting for final GitHub checks before tagging.

`0.31.0` adds bounded syntactic Merkle Patricia Trie node decoding in
`eth-valkyoth-verify`.

## Added

- `decode_mpt_node` for canonical RLP MPT branch, extension, and leaf nodes.
- `decode_mpt_proof_nodes` for caller-provided proof-node lists with cumulative
  proof-node count and encoded-byte accounting.
- Borrowed MPT node/reference types:
  - `MptNode`
  - `MptBranchNode`
  - `MptExtensionNode`
  - `MptLeafNode`
  - `MptCompactPath`
  - `MptNodeReference`
  - `MptProofNodes`
- Stable MPT node decode error codes, categories, and facade re-exports.
- `mpt_node` fuzz target plus committed seed corpus for valid branch, leaf,
  extension, inline-reference, malformed field count, bad compact path, and
  invalid child-reference inputs.
- `docs/mpt-nodes.md` describing the syntactic decode boundary and the deferred
  proof-verification work.

## Security Notes

- The decoder does not allocate decoded node structures.
- `decode_mpt_proof_nodes` accounts every encoded node against
  `max_proof_nodes` and `max_total_allocation`.
- Compact-path flags and padding are checked before a two-field node is exposed
  as extension or leaf.
- Scalar child references are accepted only as empty branch children or 32-byte
  hash references. Inline child nodes must be RLP lists.
- Inline child lists are shape-checked eagerly under
  `MPT_INLINE_REFERENCE_DEPTH_LIMIT` and rejected at
  `MPT_MAX_INLINE_REFERENCE_BYTES` or larger.
- Branch nodes store decoded child references and value bytes after the eager
  validation pass so common accessors do not reparse all branch children.
- This release does not verify trie roots. Transaction, receipt, account,
  storage, and withdrawal proof verification remain scheduled for later
  releases.

## Versioning

- `eth-valkyoth-verify` publishes as `0.18.0` because it adds public MPT decode
  APIs.
- The facade crate publishes as `eth` `0.31.0`.
- Unchanged support crates are not republished.
