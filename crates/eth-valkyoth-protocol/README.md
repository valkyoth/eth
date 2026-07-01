# eth-valkyoth-protocol

Support crate for `eth`: fork-aware `no_std` Ethereum protocol validation
state and transaction envelope shell classification.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.13"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.13.0` release classifies EIP-2718 typed transaction envelopes and
decodes legacy plus EIP-2930 access-list transaction fields into explicitly
unvalidated models. It does not validate signatures, recover senders, enforce
chain binding, account for gas, apply duplicate access-list policy, or imply
fork validity.
