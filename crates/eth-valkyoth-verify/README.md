<p align="center">
  <b>no_std Ethereum verification boundaries for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-verify">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/eth">
    <img src="https://raw.githubusercontent.com/valkyoth/eth/main/.github/images/eth.webp" alt="eth Rust crate overview">
  </a>
</p>

# eth-valkyoth-verify

Support crate for `eth`: `no_std` Ethereum verification boundaries.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.52.3"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.24.0` release adds shared-session MPT node and proof-node syntax entry
points. They charge structural and borrowed semantic passes to one ledger;
complete reject-before-hash proof preflight remains assigned to `eth` `0.52.4`.

The `0.23.0` support-crate release, shipped with `eth` `0.52.1`, rejects
malformed EIP-712 struct and field identifiers, duplicate borrowed type,
field, and value names, and clears partial encode-data output on failure.
It also validates fully unwrapped array member types before hashing and adds
default plus caller-configurable cumulative dynamic-byte work limits across
domain and message hashing. These are signing-boundary security contract
changes; use `0.23` rather than a `0.22` compatibility requirement.

The previous `0.22.0` support-crate release, shipped with `eth` `0.52.0`, applies a
64-type ceiling to both EIP-712 schema paths, traverses shared dependency DAGs
once, redacts signing-value formatting, and removes `Copy` and `Clone` from
borrowed signing values. These are intentional public compatibility changes;
use `0.22` rather than a `0.21` compatibility requirement.

The `0.20.0` support-crate release, shipped with `eth` `0.33.0`, adds account
and storage MPT inclusion proof verification. The new
`verify_account_inclusion` and `verify_storage_inclusion` APIs verify exact
encoded account or storage value bytes at `keccak256(address)` or
`keccak256(slot_key)` under distinct `AccountTrieRoot` and `StorageTrieRoot`
domains. They prove byte-exact trie membership only; they do not decode account
fields, prove that a storage root belongs to a specific account, or interpret
the included storage scalar.

The previous `0.19.0` support-crate release, shipped with `eth` `0.32.0`, adds
transaction and receipt MPT inclusion proof verification. The new
`verify_transaction_inclusion` and `verify_receipt_inclusion` APIs verify exact
encoded transaction or receipt bytes at `rlp(transaction_index)` under distinct
`TransactionTrieRoot` and `ReceiptTrieRoot` domains. They use the
`eth-valkyoth-hash::Keccak256` trait boundary and distinguish malformed,
absent, and wrong-root/value-mismatch proofs.

The previous `0.18.0` support-crate release, shipped with `eth` `0.31.0`, adds bounded
syntactic Merkle Patricia Trie node decoding. It exposes borrowed branch,
extension, leaf, compact-path, child-reference, and proof-node-list types, and
enforces cumulative proof-node and encoded-byte budgets. This is not trie-root
or key-membership verification.

The previous `0.17.3` support-crate release, shipped with `eth` `0.30.0`, updates the
published dependency range for `eth-valkyoth-protocol 0.25.0`. No verification
API changes are introduced by this patch release.

The previous `0.17.2` support-crate release, shipped with `eth` `0.29.0`, updates the
published dependency range for `eth-valkyoth-protocol 0.24.0`. No verification
API changes are introduced by this patch release.

The previous `0.17.1` support-crate release, shipped with `eth` `0.28.0`, updates the
published dependency range for `eth-valkyoth-protocol 0.23.0`. No verification
API changes are introduced by this patch release.

The previous `0.17.0` support-crate release, shipped with `eth` `0.27.0`, adds EIP-712
JSON parser fuzz coverage and a raw JSON structural-depth regression test. The
`json` feature continues to rely on `serde_json`'s default recursion guard and
must not be built with `serde_json/unbounded_depth`.

The previous `0.16.0` support-crate release, shipped with `eth` `0.26.1`, adds an
optional `json` feature for bounded EIP-712 JSON-RPC typed-data parsing. The
feature depends on current `serde`/`serde_json`, requires `std`, rejects
duplicate JSON object keys, enforces explicit parser limits, and remains
disabled by default.

The previous `0.15.0` support-crate release, shipped with `eth` `0.26.0`, adds a
no-allocation EIP-712 typed-data encoder over caller-provided borrowed
descriptors. It supports canonical `encodeType`, bounded `encodeData`,
`hashStruct`, domain separator construction, and final `0x1901` signing digest
construction without adding a concrete Keccak backend or JSON parser.

The previous `0.14.2` support-crate release aligned the published codec,
primitive, hash, and protocol dependency ranges for the public RLP derive
surface.

The previous `0.14.1` support-crate release aligned the protocol dependency
with the EIP-7702 set-code transaction validity gate.

The previous `0.14.0` release added EIP-7702 set-code transaction signing
hashes, decoded set-code transaction signature validation, and authorization
tuple signing-hash plus signer recovery helpers. The transaction signature
domain and authorization signature domain use distinct APIs and hash newtypes.

The crate also provides decoded transaction signature validation helpers for
legacy EIP-155, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 transaction
domains. Use raw digest recovery only after constructing the correct Ethereum
signing digest and checking the transaction, authorization, or structured-data
domain.

Decoded transaction signature validation is still not full execution
validation. It does not itself prove fork validity, enforce fee rules, validate
account state, enforce EIP-7702 authorization chain/nonce/account-state policy,
or validate blob/KZG commitments. Use the protocol validity gate for the
non-cryptographic set-code transaction checks.

EIP-712 helpers require the caller to provide both `chainId` and
`verifyingContract`, then check them against the expected execution context
before sender recovery. The typed-data encoder now computes domain separators
and message hashes from borrowed descriptors. JSON-RPC typed-data parsing is
available only through the opt-in `json` feature and does not affect default
`no_std` builds. Both paths enforce the 64-type `EIP712_MAX_TYPES` ceiling.
Borrowed schemas additionally cap each struct at
`EIP712_MAX_FIELDS_PER_TYPE` (64) fields and
`EIP712_MAX_VALUES_PER_STRUCT` (64) named values. Every borrowed array
dimension is capped at `EIP712_MAX_ARRAY_ITEMS` (256), and every borrowed or
JSON operation is capped at `EIP712_MAX_VALUE_NODES` (4096) recursive value
visits, including repeated traversal through shared borrowed slices.
Borrowed operations also cap cumulative dynamic `bytes`, string, domain-name,
and domain-version hashing at `EIP712_MAX_DYNAMIC_VALUE_BYTES` (1 MiB) by
default. `Eip712Limits` and the `*_with_limits` entry points allow a stricter
deployment policy. Unsupported atomic-looking spellings and undefined array
base types are rejected during schema validation even when the supplied array
is empty. Schema validation runs once per public operation, dependency
discovery visits reachable types once before canonical ordering, and recursive
hashing reuses a fixed 64-entry type-hash cache.
Borrowed signing values are neither `Copy` nor `Clone`, and their manual
`Debug` implementations redact all payload contents.
