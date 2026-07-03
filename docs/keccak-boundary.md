# Keccak-256 Boundary

Status: v0.31.0 boundary consumed by transaction signing hashes, decoded
transaction signature validation, sender recovery, EIP-712 digest framing, and
block header hashing; optional `tiny-keccak` backend admitted behind a
non-default feature. Receipt decoding added in v0.29.0 does not add a new hash
call site. Withdrawal-list decoding added in v0.30.0 also does not add a new
hash call site. MPT node decoding added in v0.31.0 does not add a new hash call
site; root verification remains a later proof milestone.

Ethereum execution-layer hashing uses Keccak-256, not FIPS SHA3-256. The hash
boundary must be explicit before transaction hashes, recovered sender addresses,
receipt roots, or proof verification are implemented.

## Decision

`eth` uses a trait boundary in `eth-valkyoth-hash`:

- `Keccak256` for incremental hashing;
- `Keccak256Digest` as the `B256` digest domain;
- `hash_one` and `hash_chunks` helpers for caller-provided hashers;
- `KECCAK256_EMPTY`, `verify_empty_digest`, and
  `verify_empty_digest_with` for backend conformance tests.

No concrete Keccak implementation crate was admitted in `v0.10.0`.
`v0.20.0` sender recovery consumes this boundary for public-key-to-address
hashing. `v0.21.0` EIP-712 helpers consume it for
`keccak256("\x19\x01" || domainSeparator || hashStruct(message))`. `v0.22.0`
transaction signing-hash helpers consume it for canonical decoded transaction
signing preimages. `v0.23.0` decoded transaction signature validation composes
the signing-hash and public-key-to-address hashing paths through caller-provided
Keccak implementations. `v0.27.0` admits an optional `tiny-keccak` software
backend behind the `tiny-keccak` support-crate feature and the `keccak-tiny`
facade feature. The default crate graph still does not include a concrete
Keccak backend.

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
| Trait boundary only | selected for `v0.10.0` | Keeps default graph dependency-free and makes the hashing contract explicit before transaction work. |
| Built-in `tiny-keccak` backend | admitted for `v0.27.0` | Current crates.io version checked on 2026-07-02: `2.0.2`, license `CC0-1.0`, default features empty, explicit `keccak` feature available. The backend is non-default and covered by empty-input, `abc`, and chunking KATs. |
| Both trait and optional backend | selected | The trait boundary remains the default; applications can opt into the first-party software backend outside the default graph. |

## Security Rules

- Implementations must compute Ethereum Keccak-256, not SHA3-256.
- Hashing remains caller-provided by default.
- The admitted `tiny-keccak` implementation is feature-gated and documented in
  `release-crates.toml`, `deny.toml`, release notes, and this document.
- Transaction, header, sender-recovery, and proof milestones must depend on this
  boundary instead of importing hash crates directly.
- Protocol-facing APIs should introduce domain newtypes such as `TxHash`,
  `BlockHash`, or `ReceiptRoot` instead of exposing raw `B256` or
  `Keccak256Digest` values directly.
- Sender-recovery paths hash public-key material. `TinyKeccak256` does not
  expose or claim a documented sponge-state zeroization contract. Deployments
  that require hasher state clearing must provide a custom backend with an
  explicit sanitization contract at the call site. When the optional
  sanitization bridge is used, prefer hashers that implement `SecureSanitize`
  and clear sponge state on drop.
- Test doubles are acceptable only for boundary tests; they must not be exposed
  as cryptographic implementations.

## Conformance Vector

Every admitted backend includes a known-answer test that distinguishes
Ethereum Keccak-256 from FIPS SHA3-256. The boundary crate exposes this value
as `eth_valkyoth_hash::KECCAK256_EMPTY`:

```rust
/// keccak256(b"")
pub const KECCAK256_EMPTY: [u8; 32] = [
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c,
    0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b,
    0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
];
```

SHA3-256 of the empty string produces a different digest, so this vector catches
the most common backend confusion. Backend tests can call
`verify_empty_digest::<Hasher>()` when the backend implements `Default`.
Configured, hardware-backed, or platform-backed hashers that cannot implement
`Default` can call `verify_empty_digest_with(hasher)` instead. Both helpers
compare the finalized output to the crate-level `KECCAK256_EMPTY` value.
`v0.27.0` also exposes `KECCAK256_ABC` and tests `TinyKeccak256` against both
the empty-input KAT and `keccak256(b"abc")`. The backend test suite also proves
that chunked and one-shot inputs produce identical digests.

## Future Admission Checklist

Before admitting a concrete software backend:

- check current crates.io version;
- verify license compatibility;
- review default and optional features;
- verify `no_std` and allocation behavior;
- run `cargo deny check`;
- run `cargo audit`;
- add Ethereum Keccak-256 conformance tests using `KECCAK256_EMPTY`;
- document state-clearing behavior for sender-recovery hashers;
- keep the backend out of default features unless the release plan explicitly
  changes that policy.
