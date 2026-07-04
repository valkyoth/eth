# eth 0.33.0 Release Notes

Status: pentest passed; waiting for final GitHub checks before tagging.

`0.33.0` adds account and storage Merkle Patricia Trie inclusion proof
verification in `eth-valkyoth-verify`.

## Added

- `verify_account_inclusion` for byte-exact account trie inclusion at
  `keccak256(address)`.
- `verify_storage_inclusion` for byte-exact storage trie inclusion at
  `keccak256(slot_key)`.
- Distinct state proof domains:
  - `AccountTrieRoot`
  - `StorageTrieRoot`
  - `StorageSlotKey`
- Successful proof result types:
  - `VerifiedAccountInclusion`
  - `VerifiedStorageInclusion`
- Shared internal key-based proof verification that reuses the v0.32.0 bounded
  proof walker for indexed, account, and storage trie proofs.
- Regression tests for account inclusion, storage inclusion, missing account
  proof nodes, storage value mismatches, wrong roots, absent keys, and
  proof-walk depth caps.
- `mpt_proof` fuzz target that drives transaction, receipt, account, and
  storage proof verification through the optional real `TinyKeccak256`
  backend, with committed seed corpus entries.

## Security Notes

- Account keys are derived as `keccak256(address)` through the caller-provided
  `eth-valkyoth-hash::Keccak256` trait boundary.
- Storage keys are derived as `keccak256(slot_key)` through the same boundary.
- The APIs prove byte-exact trie membership only. They do not decode account
  nonce, balance, storage root, or code hash fields.
- Storage proof verification does not prove that the supplied storage root
  belongs to a specific account. Callers must compose account proof results and
  storage proof results at a higher validation layer. The call-site
  documentation now repeats this warning directly on `verify_storage_inclusion`.
- Storage values are compared as encoded bytes. The verifier does not interpret
  the storage scalar.
- Proof traversal keeps the v0.32.0 malformed/absent/wrong-root error
  separation, shared `DecodeAccumulator`, and fixed `MAX_PROOF_WALK_DEPTH`
  stack-safety cap.

## Versioning

- `eth-valkyoth-verify` publishes as `0.20.0` because it adds public account
  and storage proof verification APIs.
- The facade crate publishes as `eth` `0.33.0`.
- Unchanged support crates are not republished.
