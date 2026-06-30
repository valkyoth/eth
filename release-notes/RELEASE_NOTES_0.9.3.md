# eth 0.9.3 Release Notes

Status: implementation in progress; pentest required before tag

## Summary

`0.9.3` establishes the Keccak-256 hashing boundary before transaction hashes,
sender recovery, header hashing, receipt roots, or proof verification are
implemented.

The release adds `eth-valkyoth-hash`, a small `no_std` support crate that
defines a caller-provided Keccak-256 trait. It does not admit a concrete hashing
implementation into the default dependency graph.

## Included So Far

- Added `eth-valkyoth-hash` with the `Keccak256` trait,
  `Keccak256Digest = B256`, `hash_one`, and `hash_chunks`.
- Added test doubles for the trait boundary without exposing a fake
  cryptographic implementation.
- Re-exported the hash crate from the `eth` facade as `eth::hash`.
- Documented the boundary decision in `docs/keccak-boundary.md`.
- Evaluated implementation options: trait-only, optional `tiny-keccak`, and a
  combined trait plus optional backend model.
- Checked current `tiny-keccak` crate metadata on 2026-06-30: latest crates.io
  version `2.0.2`, license `CC0-1.0`, empty default features, explicit
  `keccak` feature available.
- Deferred any concrete Keccak implementation dependency until a future release
  adds feature, audit, maintenance, MSRV, and conformance-vector evidence.
- Updated README examples, implementation/spec planning docs, release metadata,
  and publish ordering for the new support crate.

## Still Required Before Tag

- Maintainer pentest must be run for the exact implementation commit.
- Any pentest findings must be fixed and retested.
- A permanent report must be written at `security/pentest/v0.9.3.md`.
- GitHub checks must pass on the final release report commit.

## Verification

```bash
cargo test -p eth-valkyoth-hash -p eth
scripts/checks.sh
scripts/release_0_9_gate.sh
scripts/check_latest_tools.sh
cargo deny check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
