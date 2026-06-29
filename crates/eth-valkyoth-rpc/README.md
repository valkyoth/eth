# eth-valkyoth-rpc

Support crate for `eth`: future RPC trust-policy boundary.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = "0.8"
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The trusted RPC model requires an explicit acknowledgment string at
construction time so single-provider trust decisions are visible in review.
