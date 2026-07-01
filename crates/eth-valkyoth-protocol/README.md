# eth-valkyoth-protocol

Support crate for `eth`: fork-aware `no_std` Ethereum protocol validation
state and transaction envelope shell classification.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.17"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.17.0` release adds caller-reviewed `ChainSpec`, `ForkSpec`, `Hardfork`,
and `ValidationContext` types for explicit fork activation context. It retains
the earlier EIP-2718 typed envelope classification and unvalidated transaction
models for legacy, EIP-2930 access-list, EIP-1559 dynamic-fee, and EIP-4844 blob
transactions. It does not validate signatures, recover senders, enforce
transaction chain binding, account for gas or blob gas, verify KZG
commitments/proofs, apply fee-order or duplicate access-list policy, or imply
fork validity.
