# eth-valkyoth-verify

Support crate for `eth`: `no_std` Ethereum verification boundaries.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.24"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.14.0` support-crate release, shipped with `eth` `0.24.1`, adds
EIP-7702 set-code transaction signing hashes, decoded set-code transaction
signature validation, and authorization tuple signing-hash plus signer recovery
helpers. The transaction signature domain and authorization signature domain
use distinct APIs and hash newtypes.

The crate also provides decoded transaction signature validation helpers for
legacy EIP-155, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 transaction
domains. Use raw digest recovery only after constructing the correct Ethereum
signing digest and checking the transaction, authorization, or structured-data
domain.

Decoded transaction signature validation is still not full execution
validation. It does not prove fork validity, enforce fee rules, validate account
state, enforce EIP-7702 authorization chain/nonce/account-state policy, or
validate blob/KZG commitments.

EIP-712 helpers require the caller to provide both `chainId` and
`verifyingContract`, then check them against the expected execution context
before sender recovery. They do not encode arbitrary typed data or prove that a
domain separator was computed correctly.
