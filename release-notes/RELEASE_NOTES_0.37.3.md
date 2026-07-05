# eth 0.37.3 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.37.3` moves Ethereum sender recovery behind an explicit secp256k1 backend
boundary. The default `eth` graph no longer selects `k256`; callers can opt into
the reviewed compatibility adapter with `secp256k1-k256`.

## Added

- `RecoverableSecp256k1` defines the no-alloc backend contract for recovering
  uncompressed public-key payloads from Ethereum signing digests.
- `K256Secp256k1Backend` is available behind the explicit
  `secp256k1-k256` feature.
- Added backend-aware sender recovery, EIP-712 recovery, transaction signature
  validation, and EIP-7702 authorization validation APIs.
- Added `docs/signature-backend-boundary.md` for HSM, platform, WASM, embedded,
  and audited software backend integration rules.
- Added `scripts/release_0_37_3_gate.sh`, including a default dependency graph
  check that rejects accidental `k256` or `sha3` inclusion.

## Changed

- `eth-valkyoth-verify` publishes as `0.21.0`.
- `eth` publishes as `0.37.3` and exposes the `secp256k1-k256` facade feature.
- Verify tests use `eth-valkyoth-hash` with the optional `tiny-keccak` backend
  instead of a direct `sha3` dev-dependency.

## Migration

- Default builds should use `recover_sender_from_digest_with_backend`.
- Higher-level callers should use the matching `_with_backend` validators.
- The older convenience functions remain available only with
  `features = ["secp256k1-k256"]`.

## Security Notes

- Backend implementations must reject malformed scalars, enforce low-s,
  accept only y-parity values `0` and `1`, and return only a 64-byte
  uncompressed public-key payload.
- Address derivation remains first-party and uses the caller-provided
  Keccak-256 boundary.
- Concrete hashers and secp256k1 backends that hold mutable cryptographic state
  should document state-clearing behavior for sender-recovery paths.

## Verification

- `cargo test -p eth-valkyoth-verify`
- `cargo test -p eth-valkyoth-verify --all-features`
- `cargo tree -p eth --no-default-features`
- `scripts/release_0_37_3_gate.sh`

## Pentest

- External pentest is required before tagging.
- Permanent report path after clean retest:
  `security/pentest/v0.37.3.md`.

## Versioning

- `eth-valkyoth-verify` publishes as `0.21.0`.
- `eth` publishes as `0.37.3`.
- Other support crates are unchanged and are not republished.
