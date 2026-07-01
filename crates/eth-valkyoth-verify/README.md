# eth-valkyoth-verify

Support crate for `eth`: `no_std` Ethereum verification boundaries.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.21"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.10.0` support-crate release, shipped with `eth` `0.21.0`, adds
EIP-712 domain-safety checks on top of digest-level sender recovery. Use raw
digest recovery only after constructing the correct Ethereum signing digest and
checking the transaction or structured-data domain.

Sender recovery is not full transaction validation. It does not build signing
hashes from decoded transactions, prove fork validity, enforce fee rules,
validate account state, or validate blob/KZG commitments.

EIP-712 helpers require the caller to provide both `chainId` and
`verifyingContract`, then check them against the expected execution context
before sender recovery. They do not encode arbitrary typed data or prove that a
domain separator was computed correctly.
