# Release Notes - eth v0.52.3

Status: implementation complete; independent pentest pending.

## Summary

This release adds a non-copyable shared decode session so one reviewed policy
accounts for all parser work caused by a composite untrusted operation. RLP,
all supported transaction families, nested transaction substructures, and MPT
syntax parsing can now share one cumulative ledger instead of resetting local
budgets at each stage.

The project remains an incrementally built Ethereum implementation. This
release strengthens admission accounting; it does not claim a production node
or complete MPT proof preflight, which is the next planned release.

## Changed

- Added `DecodeSessionPolicy` with checked cross-limit relationships and a
  deployment starting point that must be reviewed before use.
- Added non-copyable `DecodeSession` counters for scanned bytes, RLP headers,
  items, nesting, requested allocation capacity, proof nodes, hashes, hash
  bytes, and total work.
- Added session-aware scalar/list RLP entry points and per-step nested cursors.
- Added session-aware envelope and transaction decoders for legacy, EIP-2930,
  EIP-1559, EIP-4844, and EIP-7702, including access lists, storage keys, blob
  hashes, and authorization tuples.
- Added session-aware MPT node and proof-node syntax decoders.
- Extended the decode-budget fuzz target across every session work domain.
- Clarified the facade README: the project is implementing the complete
  Ethereum stack in audited stages, while incomplete products remain explicit.

## Security Notes

- The session cannot be copied, cloned, or reset by nested consumers.
- Structural RLP validation visits each encoded byte once; actual zero-copy
  semantic reparses are charged separately.
- Composite RLP and hash charges are atomic and use checked arithmetic.
- Borrowed transaction and MPT decode paths perform no allocations, so they do
  not create misleading allocation charges. Future owned conversions must
  debit requested capacity before allocation.
- MPT hashing preflight is intentionally not claimed here; `v0.52.4` requires
  complete proof admission before attacker-controlled bytes are hashed.

## Versioning

- `eth-valkyoth-codec` advances to `0.20.0` for the public session API.
- `eth-valkyoth-protocol` advances to `0.26.0` for public session-aware
  transaction APIs.
- `eth-valkyoth-verify` advances to `0.24.0` for public session-aware MPT APIs.
- Dependent support crates receive dependency-only patch releases so crates.io
  resolves one codec/protocol type identity.
- The `eth` facade advances to `0.52.3`.

## Verification

- Focused counter conservation, no-reset, cross-limit, and atomicity tests.
- Composite access-list transaction and MPT semantic-pass accounting tests.
- `cargo test -p eth-valkyoth-codec -p eth-valkyoth-protocol -p eth-valkyoth-verify --all-features`
- `cargo clippy -p eth-valkyoth-codec -p eth-valkyoth-protocol -p eth-valkyoth-verify --all-targets --all-features -- -D warnings`
- `cargo check --manifest-path fuzz/Cargo.toml --all-targets`
- Full workspace and release-gate verification remains required before tag.

## Pentest

This exact implementation commit must be independently pentested. Findings,
remediation, and clean retest evidence will be recorded in
`security/pentest/v0.52.3.md` before release.
