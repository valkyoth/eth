# k256 Dependency Admission

Status: admitted as an explicit optional backend for `v0.37.3`

`eth-valkyoth-verify` admits `k256` for secp256k1 ECDSA public-key recovery
only behind the explicit `secp256k1-k256` feature. The default protocol-core
graph exposes the first-party `RecoverableSecp256k1` boundary and does not
select a concrete curve implementation.

## Version Decision

| Crate | Version | Decision |
| --- | --- | --- |
| `k256` | `0.13.4` | Selected stable release. |
| `k256` | `0.14.0-rc.15` | Not selected because it is a release candidate. |
| `secp256k1` | `0.32.0-beta.2` | Not selected because it is beta and uses `CC0-1.0`; only reviewed dependencies with scoped cargo-deny exceptions may use that license. |

The selected version was checked with `cargo info k256@0.13.4` on 2026-07-01.
It declares `rust-version = 1.65`, below this workspace's Rust `1.90.0` MSRV.
Recovery tests now use the project `eth-valkyoth-hash` Keccak boundary with the
optional `tiny-keccak` test backend instead of a direct `sha3` dev-dependency.

## Feature Decision

`eth-valkyoth-verify` uses:

```toml
k256 = { version = "0.13.4", default-features = false, features = ["ecdsa"], optional = true }
```

Default features are disabled so `std`, PKCS#8, Schnorr, and precomputed-table
features are not admitted by accident. The `ecdsa` feature is required for
`Signature`, `RecoveryId`, `SigningKey` test fixtures, and
`VerifyingKey::recover_from_prehash`.

## Security Rules

- `eth-valkyoth-verify` accepts only Ethereum y-parity recovery IDs `0` and
  `1`; recovery IDs `2` and `3` are rejected at the API boundary.
- `s` values must be low-s. High-s signatures fail with
  `VerifyError::InvalidSignature`.
- `r` and `s` must be valid non-zero secp256k1 scalars.
- The caller must pass the already constructed Ethereum signing digest. This
  release does not build transaction signing preimages.
- Public-key-to-address hashing must use the `eth-valkyoth-hash::Keccak256`
  trait boundary. No concrete Keccak backend is admitted in this release.
- Concrete sender-recovery hashers should have an explicit state-clearing
  policy. Prefer the optional sanitization bridge when a stateful software
  hasher buffers sensitive deployment context.
- Sender recovery must keep at least one independent Keccak-backed Ethereum
  vector test. Self-referential signing/recovery round trips are not enough for
  consensus-sensitive address attribution.

## Verification

Expected checks before tagging `v0.37.3`:

```bash
cargo test -p eth-valkyoth-verify --all-features
cargo clippy -p eth-valkyoth-verify --all-targets --all-features -- -D warnings
cargo tree -p eth --no-default-features
cargo deny check
cargo audit
scripts/release_0_37_3_gate.sh
```
