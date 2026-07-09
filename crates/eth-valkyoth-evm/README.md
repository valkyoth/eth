<p align="center">
  <b>explicit no_std EVM execution boundary for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-evm">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/eth">
    <img src="https://raw.githubusercontent.com/valkyoth/eth/main/.github/images/eth.webp" alt="eth Rust crate overview">
  </a>
</p>

# eth-valkyoth-evm

Support crate for `eth`: explicit no_std EVM execution boundary and REVM
dependency admission review.

Most users should depend on the facade crate instead:

```toml
[dependencies]
eth = { version = "0.40.0", features = ["evm"] }
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.10.0` support-crate release, shipped with `eth` `0.39.0`, adds the
bounded gas-estimation boundary on top of the first execution boundary types:

- `ExecutionEnvironment` binds an active fork validation context to a matching
  block context;
- `ExecutionTransaction` binds raw transaction bytes to a bounded decoded
  transaction envelope shell;
- `StateSnapshot` supplies account/storage state behind a caller-provided
  snapshot ID;
- `ExecutionReport` records the exact environment, transaction type,
  caller-computed transaction hash, and state snapshot selected for an
  execution attempt.
- `GasEstimationPolicy`, `GasEstimationRequest`, and `GasEstimationReport`
  require maximum attempts, a gas cap, and a deterministic termination guard
  under hard release ceilings before future estimators can run.

No execution backend is admitted yet. The previous REVM dependency review
remains in force: current REVM candidates are rejected by this repository's
MSRV and cargo-deny policy.
