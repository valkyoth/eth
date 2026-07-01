# eth-valkyoth-verify

Support crate for `eth`: `no_std` Ethereum verification boundaries.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.19"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.8.0` support-crate release, shipped with `eth` `0.19.0`, adds
replay-domain validation helpers for decoded transaction field models. Use
these helpers to reject pre-EIP-155 legacy transactions and wrong-chain legacy,
EIP-2930, EIP-1559, or EIP-4844 transactions before trusting future
sender-recovery results.

Replay-domain validation is not signature validation. It does not recover
senders, check low-s policy, prove fork validity, enforce fee rules, or validate
blob/KZG commitments.
