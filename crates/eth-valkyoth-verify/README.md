# eth-valkyoth-verify

Support crate for `eth`: `no_std` Ethereum verification boundaries.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.25"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.14.2` support-crate release, shipped with `eth` `0.25.0`, aligns the
published codec, primitive, hash, and protocol dependency ranges for the public
RLP derive surface.

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
before sender recovery. They do not encode arbitrary typed data or prove that a
domain separator was computed correctly.
