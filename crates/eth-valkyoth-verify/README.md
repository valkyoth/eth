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

The `0.13.0` support-crate release, shipped with `eth` `0.24.0`, adds
set-code replay-domain support and fail-closed decoded signature validation
handling for EIP-7702 set-code transactions. The unified validation helper now
returns an explicit unsupported-transaction-type error for set-code
transactions until the EIP-7702 signing-hash and authorization-signature paths
are admitted.

The crate also provides decoded transaction signature validation helpers for
legacy EIP-155, EIP-2930, EIP-1559, and EIP-4844 transaction domains. Use raw
digest recovery only after constructing the correct Ethereum signing digest and
checking the transaction or structured-data domain.

Decoded transaction signature validation is still not full execution
validation. It does not prove fork validity, enforce fee rules, validate account
state, validate set-code authorizations, or validate blob/KZG commitments.

EIP-712 helpers require the caller to provide both `chainId` and
`verifyingContract`, then check them against the expected execution context
before sender recovery. They do not encode arbitrary typed data or prove that a
domain separator was computed correctly.
