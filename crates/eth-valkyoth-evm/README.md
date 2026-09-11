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

Non-forgeable execution admission, explicit host capabilities and bounded
gas-estimation contracts for [`eth`](https://crates.io/crates/eth).
This `no_std` boundary does not bundle a complete execution engine.

```sh
cargo add eth --features evm
```

## Example

Build a bounded gas-estimation policy without starting any backend:

```rust
use eth::evm::{GasEstimationPolicy, GasEstimationTermination};
use eth::primitives::Gas;

let policy = GasEstimationPolicy::try_new(
    8,
    Gas::new(50_000),
    GasEstimationTermination::BackendStepLimit { max_backend_steps: 1_000 },
)?;
assert_eq!(policy.gas_cap(), Gas::new(50_000));
# Ok::<(), eth::evm::GasEstimationError>(())
```

A policy is not an estimate. A request additionally needs an execution-ready
transaction, an immutable state snapshot and an admitted backend.

## Current Scope

- direct host adapters for the allocation-free embedded and optional
  pre-reserved fixed-width-radix node access trackers;
- validated transaction ceilings for warm access, journals, checkpoints,
  frames, memory, reusable arenas, caches, and abstract work;
- destructive governor reset, monotonic fail-closed charging, and non-copyable
  hierarchical work tokens that cannot create child authority;
- type-separated cumulative/high-water resource APIs, bounded abstract work,
  and nested EIP-2929 access rollback aligned with journal checkpoints;
- deterministic capacity, cancellation, delegation, and reset tests.

Execution admission and host-capability contracts include:

- `ClassifiedEnvelope -> CanonicallyDecodedTransaction ->
  ForkValidatedTransaction -> ExecutionReadyTransaction` promotion;
- fail-closed empty and unsupported typed-envelope admission;
- type-specific canonical decoding under one conserved `DecodeSession`;
- active-fork and signed-chain checks before execution-ready promotion;
- `ExecutionRequest` construction only from the non-forgeable final token;
- a request-bound `ExecutionHost` with private `StateJournal`,
  `AccessTracker`, `CryptoProvider`, and `TransactionArena` capabilities;
- journal-authoritative current storage associated with the request's exact
  immutable `StateView`;
- closure-scoped LIFO child execution, RAII host poisoning across every
  journal/arena unwind or partial finalization, and post-transition inspector
  events;
- fail-closed poisoning across destructive transaction resets and every
  direct root-level journal, access, crypto, or arena mutation;
- `BeginChildError` preserving frame rejection and journal-cleanup failures;
- allocation-free resettable borrowed memory and iterative frame storage;
- bounded gas-estimation policy and deterministic termination ceilings.

The fork-validation token does not claim complete transaction validity. Sender
recovery, intrinsic gas, nonce/account state, balance, fee, blob/KZG,
authorization, and other state-dependent consensus rules remain mandatory
later gates.

The compatibility `StateSnapshot` trait remains available and implements the
immutable `StateView` automatically. Current storage must be supplied by its
associated journal. No complete execution backend is admitted yet.

REVM remains unadmitted. The public dependency-review record is a dated
assessment, not permission to add an unchecked backend.

See [execution admission](https://github.com/valkyoth/eth/blob/main/docs/execution-admission-host.md)
and the [resource governor](https://github.com/valkyoth/eth/blob/main/docs/execution-resource-governor.md).

## License

MIT OR Apache-2.0, at your option.
