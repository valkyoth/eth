# Signature Backend Boundary

Status: `v0.37.3` implementation ready for pentest.

`eth-valkyoth-verify` no longer requires a concrete secp256k1 implementation in
the default runtime graph. Sender recovery is split into:

- a first-party representation and policy layer (`EthereumSignature`,
  replay-domain checks, signing-hash construction, and sender/address
  comparison);
- the `RecoverableSecp256k1` backend trait for public-key recovery;
- an explicit `secp256k1-k256` feature that admits the reviewed `k256`
  compatibility adapter.

## Backend Contract

Implementations of `RecoverableSecp256k1` must:

- recover the 64-byte uncompressed public-key payload `x || y` for the exact
  signing digest and signature;
- reject malformed or zero `r`/`s` scalars;
- enforce the EIP-2 low-s rule;
- accept only Ethereum y-parity recovery IDs `0` and `1`;
- return `VerifyError::InvalidSignature` for malformed signatures or failed
  recovery;
- document any state-clearing guarantees if the backend holds mutable
  cryptographic state.

The trait does not require `alloc`, `std`, or a software curve crate. HSM,
platform, WASM, embedded, or audited software backends can all implement the
same boundary.

## Address Hashing

The backend returns only the recovered public-key payload. Address derivation
remains first-party:

```text
address = low20(keccak256(uncompressed_public_key_x_y))
```

Callers still provide the Keccak implementation through
`eth-valkyoth-hash::Keccak256`. Software hashers used on sender-recovery paths
should have an explicit state-clearing policy when deployment rules require it.

## Optional k256 Adapter

The `K256Secp256k1Backend` adapter is available only with:

```toml
[dependencies]
eth = { version = "0.37.3", features = ["secp256k1-k256"] }
```

The default `eth` dependency graph must not include `k256` or `sha3`.
`scripts/release_0_37_3_gate.sh` checks that graph before release.

## Migration Notes

Default builds should call:

```rust
use eth::verify::recover_sender_from_digest_with_backend;
```

and pass a deployment-selected backend. The older convenience function
`recover_sender_from_digest` remains available only when `secp256k1-k256` is
enabled.

The same rule applies to higher-level helpers:

- `recover_eip712_sender_with_backend`;
- `validate_transaction_signature_with_backend`;
- per-transaction-family `*_with_backend` validators;
- `validate_set_code_authorization_signature_with_backend`.

The non-`_with_backend` helpers are compatibility conveniences for the explicit
`k256` feature, not default APIs.
