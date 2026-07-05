# eth 0.38.0 Release Notes

Status: implementation ready; awaiting pentest before tagging.

`0.38.0` starts the explicit EVM execution environment boundary. It does not
admit REVM or any other concrete execution backend.

## Added

- `eth-valkyoth-evm` publishes `BlockExecutionContext`,
  `ExecutionEnvironment`, `ExecutionTransaction`, `ExecutionRequest`,
  `ExecutionReport`, `ExecutionResult`, `ExecutionStatus`, `StateSnapshot`,
  `SnapshotAccount`, and stable execution/snapshot error types.
- `ExecutionEnvironment::try_new` rejects inactive forks and fork/block chain,
  number, or timestamp mismatches.
- `ExecutionTransaction::decode` binds raw transaction bytes to a bounded
  protocol `TransactionEnvelope` shell through `decode_transaction_envelope`.
- `ExecutionRequest::report` records the exact fork/block environment,
  transaction type domain, caller-computed transaction hash, and
  caller-provided snapshot ID.
- `docs/evm-execution-environment.md` documents the boundary.
- `scripts/release_0_38_gate.sh` captures default, `evm`, and all-feature
  dependency tree evidence.

## Changed

- `eth-valkyoth-evm` publishes as `0.9.0`.
- `eth` publishes as `0.38.0` and points its optional `evm` feature at
  `eth-valkyoth-evm 0.9.0`.
- The facade error module re-exports EVM boundary errors when the `evm` feature
  is enabled.

## Security Notes

- No third-party EVM engine is admitted by this release.
- REVM remains rejected by the existing dependency review and dependency gates.
- Execution reports require a caller-computed transaction hash so reports bind
  the exact transaction identity without pulling a concrete Keccak backend into
  this crate.
- Future execution and gas-estimation code must consume the explicit
  environment, transaction, and snapshot boundary.

## Verification

- `cargo test -p eth-valkyoth-evm`
- `cargo check -p eth --features evm`
- `cargo clippy -p eth-valkyoth-evm -p eth --all-targets --all-features -- -D warnings`
- `cargo tree -p eth --no-default-features --features evm -e normal`
- `scripts/release_0_38_gate.sh`

## Pentest

- Run pentest on the implementation commit before tagging.
- Permanent report path after clean retest:
  `security/pentest/v0.38.0.md`.

## Versioning

- `eth-valkyoth-evm` publishes as `0.9.0`.
- `eth` publishes as `0.38.0`.
- Other support crates are unchanged and are not republished.
