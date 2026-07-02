# eth-valkyoth-verify

Support crate for `eth`: `no_std` Ethereum verification boundaries.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.29"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.17.2` support-crate release, shipped with `eth` `0.29.0`, updates the
published dependency range for `eth-valkyoth-protocol 0.24.0`. No verification
API changes are introduced by this patch release.

The previous `0.17.1` support-crate release, shipped with `eth` `0.28.0`, updates the
published dependency range for `eth-valkyoth-protocol 0.23.0`. No verification
API changes are introduced by this patch release.

The previous `0.17.0` support-crate release, shipped with `eth` `0.27.0`, adds EIP-712
JSON parser fuzz coverage and a raw JSON structural-depth regression test. The
`json` feature continues to rely on `serde_json`'s default recursion guard and
must not be built with `serde_json/unbounded_depth`.

The previous `0.16.0` support-crate release, shipped with `eth` `0.26.1`, adds an
optional `json` feature for bounded EIP-712 JSON-RPC typed-data parsing. The
feature depends on current `serde`/`serde_json`, requires `std`, rejects
duplicate JSON object keys, enforces explicit parser limits, and remains
disabled by default.

The previous `0.15.0` support-crate release, shipped with `eth` `0.26.0`, adds a
no-allocation EIP-712 typed-data encoder over caller-provided borrowed
descriptors. It supports canonical `encodeType`, bounded `encodeData`,
`hashStruct`, domain separator construction, and final `0x1901` signing digest
construction without adding a concrete Keccak backend or JSON parser.

The previous `0.14.2` support-crate release aligned the published codec,
primitive, hash, and protocol dependency ranges for the public RLP derive
surface.

The previous `0.14.1` support-crate release aligned the protocol dependency
with the EIP-7702 set-code transaction validity gate.

The previous `0.14.0` release added EIP-7702 set-code transaction signing
hashes, decoded set-code transaction signature validation, and authorization
tuple signing-hash plus signer recovery helpers. The transaction signature
domain and authorization signature domain use distinct APIs and hash newtypes.

The crate also provides decoded transaction signature validation helpers for
legacy EIP-155, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 transaction
domains. Use raw digest recovery only after constructing the correct Ethereum
signing digest and checking the transaction, authorization, or structured-data
domain.

Decoded transaction signature validation is still not full execution
validation. It does not itself prove fork validity, enforce fee rules, validate
account state, enforce EIP-7702 authorization chain/nonce/account-state policy,
or validate blob/KZG commitments. Use the protocol validity gate for the
non-cryptographic set-code transaction checks.

EIP-712 helpers require the caller to provide both `chainId` and
`verifyingContract`, then check them against the expected execution context
before sender recovery. The typed-data encoder now computes domain separators
and message hashes from borrowed descriptors. JSON-RPC typed-data parsing is
available only through the opt-in `json` feature and does not affect default
`no_std` builds.
