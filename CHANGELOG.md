# Changelog

All notable changes to `eth` are documented here.

## Unreleased

- Started `0.9.0` with canonical no-allocation RLP encoding helpers for
  scalar byte strings, Ethereum integer payloads, list payloads, and decoded
  RLP items.
- Added decode-then-encode canonicality tests plus scalar, integer, list,
  long-payload, output-buffer, and noncanonical-input regression coverage.
- Hardened raw list-payload encoding by validating concatenated child items
  under explicit `DecodeLimits` before returning a length or emitting a list
  header.
- Added fuzz coverage for RLP encoding length helpers, raw payload encoders,
  and decoded scalar, integer, list, and item re-encoding paths.
- Refreshed pinned execution-apis and consensus-specs revisions after checking
  official Ethereum sources for v0.9.0 codec work.
- Started `0.8.0` with canonical RLP integer decoding layered on top of
  scalar decoding, including exact and partial entry points.
- Added integer-specific rejection for single-byte zero and leading-zero
  payloads, preserving Ethereum's zero-as-empty-byte-array rule.
- Added bounded `u64`, `u128`, and unsigned 256-bit byte conversion helpers
  for canonical RLP integers.
- Added primitive constructors for canonical RLP integer payloads on the
  integer domain newtypes and `Wei`.
- Added fuzz coverage for exact and partial RLP integer decoding plus bounded
  integer conversion helpers.
- Addressed v0.8.0 pentest findings by documenting duplicated canonical
  integer logic, adding maximum-width integer and Wei boundary tests, clarifying
  U256 copy invariants, cross-referencing duplicated constants, and documenting
  Chain ID 0 domain validation requirements.
- Refreshed the pinned EIPs revision after checking official Ethereum sources
  for v0.8.0 parser work.
- Started `0.7.0` with bounded canonical RLP list decoding, including short
  and long list headers, nested traversal, list item-count enforcement, nesting
  depth enforcement, and adversarial malformed-list tests.
- Added no-allocation immediate-child iteration for decoded RLP lists through
  `RlpList::items`, `RlpListItems`, and `RlpItem`.
- Added `RlpItem::header_len`, `RlpItem::as_scalar`, `RlpItem::as_list`, and
  fused iterator behavior for `RlpListItems`.
- Split scalar and list RLP tests into separate modules and added official
  nested-list fixtures plus deeper canonical nesting budget regression coverage.
- Added fuzz coverage for exact and partial RLP list decoding paths, including
  immediate child iteration on successfully decoded lists.
- Addressed v0.7.0 pentest findings by making nested list iteration
  re-counting use the original decode limits, deepening list iterator fuzz
  coverage, documenting the RLP traversal hard cap and partial-decoder slicing
  contract, and clarifying long-list/string prefix constants.
- Addressed v0.7.0 pentest re-test findings by making `RlpList` equality
  ignore private decode policy, documenting independent iterator recount
  budgets, and simplifying list bounds-check expressions.
- Aligned every public workspace crate to `0.7.0` publication so crates.io
  receives corrected `MIT OR Apache-2.0` license metadata for all packages.
- Refreshed pinned official Ethereum source revisions for v0.7.0 parser work.
- Corrected the public crate license metadata and repository license files to
  `MIT OR Apache-2.0`.
- Started `0.6.0` with a dependency and tooling refresh before RLP scalar
  decoder work: updated `quote` to `1.0.46`, updated optional `sanitization`
  support to `1.2.2`, confirmed GitHub tooling is current, and added the
  v0.6 release gate.
- Added canonical RLP scalar byte-string decoding with exact-consumption,
  malformed length, list-prefix rejection, and budget enforcement tests.
- Added official scalar RLP example fixtures and long-length overflow coverage.
- Added fuzz coverage for exact and partial RLP scalar decoding paths.
- Refreshed pinned official Ethereum source revisions for v0.6.0 parser work.
- Addressed v0.6.0 pentest findings by gating codec test fixtures, renaming
  ambiguous decode-limit and partial-decoder APIs, adding hardened-only
  sanitization builds, and requiring explicit trusted-RPC acknowledgment.
- Started `0.5.0` by extending the decode-budget model with proof-node and
  cumulative item budgets, checked length and range helpers, and adversarial
  tests for overflow and limit rejection.
- Addressed v0.5.0 pentest findings for enum sanitization residual bytes,
  sanitization hardening evidence, spec-source pinning, decode limit naming,
  production-template fuzzing, hash timing documentation, typestate dead code,
  non-exhaustive public errors, TryFrom transaction type documentation, and
  skipped-field generic derive bounds.
- Addressed v0.5.0 follow-up pentest findings by making
  `SecureSanitizeOnDrop` struct-only and documenting downstream
  `HARDENED_MODE` assertion patterns.
- Started `0.4.0` by adding independent support-crate version planning,
  release-plan validation, and a crate version matrix to avoid unnecessary
  crates.io publishes.
- Added stable error codes, messages, categories, and formatting for codec,
  protocol, fork, feature, resource, and verification failures.
- Addressed v0.4.0 pentest findings for typestate token creation, address
  comparison timing, decode-limit API naming, sanitization skip acknowledgement,
  typed-envelope classification, best-effort sanitization visibility, and fuzz
  bootstrap coverage for all decode-budget APIs.
- Added crate-local READMEs for published support crates that point users to
  the `eth` facade crate.
- Added workspace packaging verification to local checks.
- Fixed facade crate docs to include a packaged README instead of a workspace
  root path.
- Initialized the `eth` Rust workspace.
- Added first-party `no_std` crate boundaries.
- Added security, supply-chain, modularity, implementation, and release plans.
- Added local check and release-gate scripts.
- Expanded the release plan into smaller milestone tags with explicit exit
  criteria and mandatory pentest-before-tag readiness checks.
- Added a spec-source policy requiring current official Ethereum sources,
  pinned revisions, and local fixture evidence before consensus-sensitive work.
- Addressed v0.1.0 pentest release-gate findings for CI pinning, advisory
  policy, release readiness, lints, and metadata validation.
- Added explicit secret-handling policy and hardened current placeholder
  primitives/protocol helpers flagged during pentest.
- Added advisory checks for pinned CI tools and GitHub Actions currency.
- Started `0.2.0` by moving support crates to the `eth-valkyoth-*` namespace
  and adding a crates.io release-order helper.
- Added release-readiness negative tests for missing or stale release evidence.
- Addressed v0.2.0 pentest findings for constant-time equality, decode-limit
  enforcement, fork activation semantics, typestate direction, advisory policy,
  deterministic release gates, and RPC trust-model defaults.
- Implemented `0.3.0` domain newtypes with explicit wei and transaction type
  primitives, conversion coverage, and the v0.3 release gate.
- Added optional `eth-valkyoth-sanitization` and `eth-valkyoth-derive` support
  crates outside the default `eth` feature set.
- Addressed v0.3.0 pentest findings for constant-time primitive equality,
  cumulative decode allocation accounting, enum sanitization acknowledgement,
  typed transaction disambiguation, and release/tooling gates.
