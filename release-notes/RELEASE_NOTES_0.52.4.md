# Release Notes - eth v0.52.4

Status: implementation complete; pentest pending.

## Summary

This release makes Merkle Patricia Trie inclusion verification fail before
hashing attacker-controlled proof nodes when the complete proof exceeds its
reviewed work policy or contains a locally detectable noncanonical node form.
Transaction, receipt, account, and storage proofs can now consume one public
operation-wide `DecodeSession`.

This remains inclusion verification against caller-trusted roots. Composed
account/storage authority and canonical account decoding are assigned to
`v0.52.5`.

## Changed

- Added compact-path nibble and trie-value byte ceilings to
  `DecodeSessionPolicy` and cumulative counters to `DecodeSession`.
- Added an opaque complete-work snapshot and noncommitting capacity check to
  `DecodeSession`; actual work remains charged immediately before execution.
- Preflight now checks every proof-node count and encoded length, cumulative
  hash bytes, expected-value work, and local syntax, then performs a
  conservative dry traversal to check all remaining parser, hash, nibble,
  value, and aggregate work before the first proof-node Keccak invocation.
- Dry-traversal parsing, path comparison, and value comparison now debit the
  caller's operation-wide session; only the opaque future-work check remains
  noncommitting.
- Added public session-aware transaction, receipt, account, and storage
  inclusion APIs.
- Rejects zero-nibble extensions, empty leaf values, branches with fewer than
  two occupied outcomes, extension children that are not branches, inline
  children at or above 32 encoded bytes, and hashed children below 32 bytes.
- Extended the MPT proof fuzz target to construct a valid root, require
  successful verification, mutate the proof, require rejection, and assert
  session complexity ceilings.

## Security Notes

- A malformed trailing node or insufficient complete hash budget causes zero
  proof hasher invocations.
- Insufficient encoded-byte, header, item, nibble, value, or aggregate-work
  capacity also causes zero proof-node hasher invocations.
- Account and storage key hashing is included in the same hash-count and
  hash-byte capacity preflight as proof-node hashing.
- Preflight parsing, dry planning, and later traversal parsing are all charged
  because each consumes attacker-controlled work.
- Rejected retries cannot repeat the dry traversal against a reset side ledger;
  work already performed remains charged to the caller's session.
- Legacy APIs remain bounded by a conservative compatibility session derived
  from `DecodeLimits`; security-sensitive composite callers should select an
  explicit `DecodeSessionPolicy` and use the `*_in_session` APIs.
- All arithmetic and multi-counter charges use checked, fail-without-partial-
  commit operations.

## Versioning

- `eth-valkyoth-codec` advances to `0.21.0` for public session policy and error
  additions.
- `eth-valkyoth-verify` advances to `0.25.0` for public proof APIs and stricter
  canonical admission.
- Dependent support crates receive dependency-only patch releases so crates.io
  resolves one public type identity.
- The `eth` facade advances to `0.52.4`.

## Verification

- Canonical and noncanonical branch, extension, leaf, inline, and hash-reference
  boundary tests.
- Exact session counter assertions for nodes, hash bytes, hashes, nibbles, and
  values.
- Per-domain hasher-call oracles proving failed preflight cannot invoke the
  proof-node backend.
- Release metadata validation proving the documented MPT execution-spec source
  revision matches `spec-lock.toml`.
- Structure-aware valid-root/mutation MPT proof fuzzing.
- Strict workspace and fuzz Clippy, complete workspace tests, supported-Rust
  checks, dependency policy checks, package verification, and the full release
  gate remain required before tagging.

## Pentest

An independent pentest and clean retest are mandatory before this release can
be tagged. The permanent report will be recorded at
`security/pentest/v0.52.4.md`.
