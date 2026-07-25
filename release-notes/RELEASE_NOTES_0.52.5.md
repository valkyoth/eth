# Release Notes - eth v0.52.5

Status: implementation complete; pentest required before release.

## Summary

This release makes EIP-1186 storage-proof authority derive from authenticated
account state. Canonical account decoding produces a non-forgeable
`VerifiedAccount`; composed storage verification accepts that capability
instead of an unrelated caller-supplied root.

The complete account-plus-storage workflow remains `no_std`, allocation-free
in the proof kernel, and bounded by the shared `DecodeSession`.

## Added

- `EthereumAccount` for canonical
  `[nonce, balance, storageRoot, codeHash]` state values.
- `VerifiedAccount` for authenticated account inclusion or canonical absence.
- `VerifiedStorageValue` for authenticated nonzero inclusion or canonical zero
  by absence.
- `verify_account_proof` and `verify_account_proof_in_session`.
- `verify_account_storage` and `verify_account_storage_in_session`.
- Canonical empty account/storage trie root constants.
- Stable composed-proof error codes and categories.

## Security Properties

- `VerifiedAccount` fields and constructors are private and the capability is
  neither `Clone` nor `Copy`.
- Storage verification obtains its root only from authenticated account state.
- A valid account proof cannot be combined with storage proven under a
  different root.
- Present account tuples require exactly four scalar fields, canonical U64 and
  U256 integers, and exact 32-byte storage/code hashes.
- Account absence authorizes only the canonical empty storage root.
- Storage path absence is successful zero; an explicitly present zero value is
  rejected as noncanonical Ethereum state.
- Planning discovers and decodes state values before proof-node hashing.
  Cryptographic traversal must reach the same bytes or absence outcome.
- The account proof and all storage proofs can share one non-copyable work
  session.

## Compatibility

The byte-exact `verify_account_inclusion` and
`verify_storage_inclusion` functions remain available. They intentionally
retain their lower-level independently rooted contract. New untrusted
`eth_getProof` consumers should use the composed capability APIs.

## Verification

- Pinned Execution APIs Hive account-plus-storage fixture verified end to end.
- Root and account-value substitution rejection.
- Account and storage path-absence vectors.
- Empty-trie account and storage absence.
- Explicit-zero storage rejection.
- Malformed field count, integer canonicality, and hash-width rejection.
- Hasher-call oracle proving malformed account state is rejected before the
  first proof-node hash.
- Structure-aware fuzzing of valid composed chains and unrelated storage
  proofs.
- Strict workspace/fuzz Clippy, all workspace tests, supported-Rust checks,
  dependency policy, package verification, and the release gate remain
  required before tagging.

## Versioning

- `eth-valkyoth-verify` advances from `0.25.0` to `0.26.0`.
- `eth` advances from `0.52.4` to `0.52.5`.
- All other support-crate versions remain unchanged.

## Pentest

Release is blocked until an independent pentest reviews the exact
implementation commit, all findings are remediated, and a clean retest is
recorded at `security/pentest/v0.52.5.md`.
