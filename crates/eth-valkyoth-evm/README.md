# eth-valkyoth-evm

Support crate for `eth`: explicit no_std EVM execution boundary and REVM
dependency admission review.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = { version = "0.38.0", features = ["evm"] }
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.9.0` support-crate release, shipped with `eth` `0.38.0`, adds the
first execution boundary types:

- `ExecutionEnvironment` binds an active fork validation context to a matching
  block context;
- `ExecutionTransaction` binds raw transaction bytes to a bounded decoded
  transaction envelope shell;
- `StateSnapshot` supplies account/storage state behind a caller-provided
  snapshot ID;
- `ExecutionReport` records the exact environment, transaction type, and state
  snapshot selected for an execution attempt.

No execution backend is admitted yet. The previous REVM dependency review
remains in force: current REVM candidates are rejected by this repository's
MSRV and cargo-deny policy.
