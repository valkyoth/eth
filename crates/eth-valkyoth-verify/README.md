# eth-valkyoth-verify

Support crate for `eth`: `no_std` Ethereum verification boundaries.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.20"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.9.0` support-crate release, shipped with `eth` `0.20.0`, adds
digest-level sender recovery through `k256`. Use it only after constructing the
correct Ethereum signing digest and checking the transaction replay domain.

Sender recovery is not full transaction validation. It does not build signing
hashes from decoded transactions, prove fork validity, enforce fee rules,
validate account state, or validate blob/KZG commitments.
