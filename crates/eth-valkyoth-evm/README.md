<p align="center">
  <b>non-forgeable execution admission and explicit no_std EVM host powers.</b><br>
  Canonical transaction typestates, bounded host capabilities, and security-gated release evidence.
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

Support crate for `eth`: non-forgeable execution admission, explicit EVM host
capabilities, and bounded gas-estimation contracts.

Most users should depend on the facade crate:

```toml
[dependencies]
eth = { version = "0.52.7", features = ["evm"] }
```

Crates.io: <https://crates.io/crates/eth>

This package is published separately so the `eth` workspace can keep small,
auditable crate boundaries. Treat it as a lower-level building block unless the
`eth` documentation explicitly says otherwise.

The `0.11.0` support-crate release, shipped with `eth` `0.52.7`, provides:

- `ClassifiedEnvelope -> CanonicallyDecodedTransaction ->
  ForkValidatedTransaction -> ExecutionReadyTransaction` promotion;
- fail-closed empty and unsupported typed-envelope admission;
- type-specific canonical decoding under one conserved `DecodeSession`;
- active-fork and signed-chain checks before execution-ready promotion;
- `ExecutionRequest` construction only from the non-forgeable final token;
- separate `StateView`, `StateJournal`, `BlockEnvironment`, `AccessTracker`,
  `CryptoProvider`, optional `Inspector`, and `TransactionArena` contracts;
- allocation-free resettable borrowed memory and iterative frame storage;
- bounded gas-estimation policy and deterministic termination ceilings.

The fork-validation token does not claim complete transaction validity. Sender
recovery, intrinsic gas, nonce/account state, balance, fee, blob/KZG,
authorization, and other state-dependent consensus rules remain mandatory
later gates.

The compatibility `StateSnapshot` trait remains available and implements
`StateView` automatically. No complete execution backend is admitted yet.
