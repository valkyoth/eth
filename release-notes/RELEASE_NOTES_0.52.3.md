# Release Notes - eth v0.52.3

Status: implementation complete; pentest findings remediated; clean retest
complete; final release gate and GitHub checks pending.

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
- Nested-list compatibility recounts now charge each immediate child item and
  header before parsing it, including rejection at exact component ceilings.
- Added session-aware envelope and transaction decoders for legacy, EIP-2930,
  EIP-1559, EIP-4844, and EIP-7702, including access lists, storage keys, blob
  hashes, and authorization tuples.
- Added session-aware MPT node and proof-node syntax decoders.
- Added charged borrowed-model traversal for access-list entries and storage
  keys, blob hashes, EIP-7702 authorizations, and inline MPT nodes.
- EIP-7702 authorization decoding consumes the already charged tuple fields
  instead of repeating an unaccounted compatibility parse.
- Extended the decode-budget fuzz target across every session work domain.
- Clarified the facade README: the project is implementing the complete
  Ethereum stack in audited stages, while incomplete products remain explicit.

## Security Notes

- The session cannot be copied, cloned, or reset by nested consumers.
- Structural RLP validation visits each encoded byte once; actual zero-copy
  semantic reparses are charged separately.
- Composite RLP and hash charges are atomic and use checked arithmetic.
- Reviewed policies reject total-item and proof-node ceilings above aggregate
  work, in addition to the existing component relationship checks.
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

- Focused counter conservation, exact nested-recount, no-reset, cross-limit,
  and atomicity tests.
- Composite access-list transaction and MPT semantic-pass accounting tests.
- Exact EIP-7702 tuple-delta and charged inline-MPT reparse tests.
- `cargo test -p eth-valkyoth-codec -p eth-valkyoth-protocol -p eth-valkyoth-verify --all-features`
- `cargo clippy -p eth-valkyoth-codec -p eth-valkyoth-protocol -p eth-valkyoth-verify --all-targets --all-features -- -D warnings`
- `cargo check --manifest-path fuzz/Cargo.toml --all-targets`
- The exact full release gate and green GitHub checks remain required before
  tag.

## Pentest

The independent review found three Medium resource-accounting/capability gaps
and one Low cross-limit policy-validation gap. The remediation pre-charges
nested recount work, consumes already charged EIP-7702 fields, adds charged
borrowed-model traversal, and validates every component ceiling against total
work. The clean retest found no unresolved issue.

The permanent report is recorded at `security/pentest/v0.52.3.md`. External
consumers processing untrusted data must use the `*_in_session` traversal APIs;
the explicit compatibility iterators remain intended only for trusted or
independently bounded data. Compile-time hardening of that integration boundary
is assigned to `v0.54.4`.
