# eth 0.43.0 Release Notes

Status: release candidate; pentest clean; awaiting final GitHub checks.

`0.43.0` adds explicit bounded state access to the dependency-free native EVM
core. The release keeps state access caller-owned: bytecode can read account
metadata, balance, code, and storage only through a host-state trait supplied to
`run_with_state`, and warm/cold access accounting is tracked in fixed-capacity
caller-provided sets.

This is still not a production-complete execution engine. Calls, creates, logs,
precompiles, refunds, committed storage writes, access-list transaction seeding,
and full official state-test execution remain future releases.

## Added

- `EvmAddress`, `EvmAccount`, `EvmStateContext`, and the `EvmState` host-state
  trait.
- `EvmAccessSet` and `EvmAccessStatus` for fixed-capacity warm/cold account and
  storage-slot tracking.
- `EvmExecution::run_with_state` for bytecode that explicitly receives a host
  state snapshot and access set.
- State opcode admission for `BALANCE`, `EXTCODESIZE`, `EXTCODECOPY`,
  `EXTCODEHASH`, `SELFBALANCE`, and `SLOAD`.
- A fail-closed `SSTORE` shell that charges the storage access path and returns
  `StateWriteUnsupported` without mutating host state or popping stack values.
- Warm/cold gas helpers for account access, storage access, `SELFBALANCE`, and
  code-copy word costs.
- State error categories for missing host state, state read failures, oversized
  code, exhausted access-list capacity, and unsupported writes.

## Changed

- Plain `EvmExecution::run` now fails closed with `StateAccessUnavailable` when
  bytecode reaches a state opcode without a host state snapshot.
- `OpcodeTable` now classifies the admitted state opcode domain as
  `OpcodeClass::State`; the interpreter remains the authoritative executable
  subset.
- The jumpdest bitmap moved into its own module so the interpreter file stays
  below the project line-count cap as execution coverage grows.
- `eth-valkyoth-evm-core` publishes as `0.4.0`.
- `eth` publishes as `0.43.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.4.0`.

## Security Notes

- State access is unavailable unless the caller chooses `run_with_state` and
  supplies an explicit host-state trait implementation.
- Warm/cold access tracking is deterministic, fixed-capacity, and allocation
  free. Capacity exhaustion fails closed with `StateAccessListFull`.
- Warm/cold state-access gas is admitted only for `EvmFork::LONDON` and later
  in this release. Pre-Berlin historical state gas schedules are not guessed;
  state opcodes fail closed under `EvmFork::FRONTIER`.
- `EvmAccessSet` uses linear scans over caller-selected fixed capacities.
  Integrators should keep capacities bounded relative to the execution gas
  limit and deployment policy.
- `EXTCODECOPY` validates memory bounds and charges account access, copy gas,
  and memory expansion before writing into memory.
- `SSTORE` intentionally does not commit state in this release. Journaled
  writes belong to the call/create release.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo check -p eth --features evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/release_crates.py --check`
- `scripts/release_0_43_gate.sh`

## Pentest

- Permanent report: `security/pentest/v0.43.0.md`.
- Initial pentest found fork-gating, access-set atomicity, array-slice lint
  coverage, and documentation/test-coverage issues; these have been remediated
  before retest.
- Retest was clean. No blocking findings remain for the v0.43.0 release scope.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.4.0`.
- `eth` publishes as `0.43.0`.
- Other support crates are unchanged and are not republished.
