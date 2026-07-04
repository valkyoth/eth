# eth 0.32.0 Release Notes

Status: implementation ready for pentest.

`0.32.0` adds transaction and receipt Merkle Patricia Trie inclusion proof
verification in `eth-valkyoth-verify`.

## Added

- `verify_transaction_inclusion` for byte-exact transaction trie inclusion at
  `rlp(transaction_index)`.
- `verify_receipt_inclusion` for byte-exact receipt trie inclusion at
  `rlp(transaction_index)`.
- Distinct proof root domains:
  - `MptProofRoot`
  - `TransactionTrieRoot`
  - `ReceiptTrieRoot`
- Successful proof result types:
  - `VerifiedTransactionInclusion`
  - `VerifiedReceiptInclusion`
- Stable proof verification errors and categories that distinguish:
  - malformed or incomplete proof inputs;
  - absence at the requested key;
  - wrong root, hash-reference mismatch, trailing nodes, or value mismatch.
- `MAX_PROOF_WALK_DEPTH` and `ProofTooDeep` enforcement for stack-safe proof
  traversal independent of caller-selected proof-node budgets.
- Proof walking over the `eth-valkyoth-hash::Keccak256` trait boundary via a
  caller-provided hasher factory.
- Regression tests for transaction inclusion, receipt inclusion through a
  branch child, absent keys, wrong roots, value mismatches, missing child nodes,
  trailing proof nodes, and stable error categories.

## Security Notes

- The inclusion APIs verify exact trie membership of caller-provided encoded
  transaction or receipt bytes. They intentionally do not decode, execute, or
  consensus-validate those values.
- Proof node decoding is performed while walking the proof with one shared
  `DecodeAccumulator`, so consumed hashed proof nodes are not pre-decoded and
  decoded again by the verifier.
- Proof walking is iterative, not mutually recursive, and fails closed at
  `MAX_PROOF_WALK_DEPTH` before adversarial proof chains can grow native stack
  usage.
- Hashed child references are checked by hashing the next encoded proof node
  with the caller-provided Keccak implementation. Inline child references remain
  bounded by the v0.31.0 inline-size and inline-depth decoder rules.
- Inline child proof traversal is covered by direct success-path regression
  tests.
- Extra proof nodes after a successful match are rejected.
- Account and storage proof verification remain scheduled for `v0.33.0`.

## Versioning

- `eth-valkyoth-verify` publishes as `0.19.0` because it adds public proof
  verification APIs.
- The facade crate publishes as `eth` `0.32.0`.
- Unchanged support crates are not republished.
