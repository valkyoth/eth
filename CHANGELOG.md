# Changelog

All notable changes to `eth` are documented here.

## Unreleased

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
