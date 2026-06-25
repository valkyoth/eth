# Changelog

All notable changes to `eth` are documented here.

## Unreleased

- Started `0.6.0` with a dependency and tooling refresh before RLP scalar
  decoder work: updated `quote` to `1.0.46`, updated optional `sanitization`
  support to `1.2.2`, confirmed GitHub tooling is current, and added the
  v0.6 release gate.
- Added canonical RLP scalar byte-string decoding with exact-consumption,
  malformed length, list-prefix rejection, and budget enforcement tests.
- Added official scalar RLP example fixtures and long-length overflow coverage.
- Added fuzz coverage for exact and prefix RLP scalar decoding paths.
- Refreshed pinned official Ethereum source revisions for v0.6.0 parser work.
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
