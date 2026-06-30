# Keccak-256 Boundary

Status: v0.9.3 boundary decision

Ethereum execution-layer hashing uses Keccak-256, not FIPS SHA3-256. The hash
boundary must be explicit before transaction hashes, recovered sender addresses,
header hashes, receipt roots, or proof verification are implemented.

## Decision

`eth` uses a trait boundary in `eth-valkyoth-hash`:

- `Keccak256` for incremental hashing;
- `Keccak256Digest` as the `B256` digest domain;
- `hash_one` and `hash_chunks` helpers for caller-provided hashers.

No concrete Keccak implementation crate is admitted in `v0.9.3`.

## Rationale

The default graph stays `no_std`, allocation-free, and implementation-neutral.
Callers can use hardware hashing, platform APIs, WASM host functions, embedded
providers, or a reviewed software crate without `eth` choosing a backend for
every deployment.

This is important for:

- transaction hashes;
- sender recovery;
- execution header hashes;
- receipt and trie roots;
- proof verification;
- deployments that need platform-specific or audited hashing providers.

## Evaluated Options

| Option | Decision | Reason |
| --- | --- | --- |
| Trait boundary only | selected for `v0.9.3` | Keeps default graph dependency-free and makes the hashing contract explicit before transaction work. |
| Built-in `tiny-keccak` backend | deferred | Current crates.io version checked on 2026-06-30: `2.0.2`, license `CC0-1.0`, default features empty, explicit `keccak` feature available. It is not admitted until a future release performs feature, audit, maintenance, and MSRV evidence review. |
| Both trait and optional backend | deferred | Likely future shape if a first-party optional backend is useful, but not needed before transaction types exist. |

## Security Rules

- Implementations must compute Ethereum Keccak-256, not SHA3-256.
- Hashing remains caller-provided unless a future release explicitly admits an
  implementation crate.
- Any admitted implementation must be feature-gated and documented in
  `release-crates.toml`, `deny.toml`, release notes, and this document.
- Transaction, header, sender-recovery, and proof milestones must depend on this
  boundary instead of importing hash crates directly.
- Test doubles are acceptable only for boundary tests; they must not be exposed
  as cryptographic implementations.

## Future Admission Checklist

Before admitting a concrete software backend:

- check current crates.io version;
- verify license compatibility;
- review default and optional features;
- verify `no_std` and allocation behavior;
- run `cargo deny check`;
- run `cargo audit`;
- add conformance vectors for Ethereum Keccak-256;
- keep the backend out of default features unless the release plan explicitly
  changes that policy.
