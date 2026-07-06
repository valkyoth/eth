# eth 0.42.0 Release Notes

Status: implementation ready; initial pentest found a release-gate issue under
remediation.

`0.42.0` adds fork-scoped gas accounting to the native
`eth-valkyoth-evm-core` interpreter. The release keeps the execution scope
narrow: it meters only the opcode subset admitted in `0.41.0` and adds
memory-expansion gas for `RETURN` and `REVERT` shells. It still does not claim
state access, calls, creates, logs, precompiles, refunds, storage warm/cold
tracking, or production-valid execution.

## Added

- `EvmGas`, `EvmGasSchedule`, and `EvmGasMeter`.
- `EVM_DEFAULT_GAS_LIMIT` and `EVM_MAX_GAS_LIMIT` release ceilings.
- Fork-scoped gas schedule construction through `EvmGasSchedule::for_fork`.
- Fixed base gas costs for the currently executable opcode subset.
- EVM memory-expansion gas formula for checked memory ranges.
- `ExecutionLimits` now carries an explicit gas limit and fork.
- `ExecutionReport` now records `gas_used` and `gas_remaining`.
- Fail-closed gas errors for zero gas limits, oversized gas limits, out of gas,
  and gas/memory-expansion arithmetic overflow.
- Regression coverage proving gas is charged before stack side effects.
- Release metadata validation now derives the current release version and
  requires the matching pentest report to exist with `Status: PASS`.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.3.0`.
- `eth` publishes as `0.42.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.3.0`.
- `ExecutionLimits::try_new` now requires `max_steps`, `gas_limit`, and
  `EvmFork`.

## Security Notes

- Every executed opcode in the admitted subset charges gas before applying stack
  or control-flow side effects.
- `RETURN` and `REVERT` validate memory ranges and charge memory expansion
  before popping their stack operands.
- Unsupported opcodes still fail closed and are not assigned executable gas
  semantics in this release.
- Gas accounting is limited to the current stack/control-flow subset. State,
  call, create, log, precompile, refund, and storage access-list accounting
  remain future releases.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo check -p eth --features evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/release_0_42_gate.sh`

## Pentest

- Pending. The release must not be tagged until the local `PENTEST.md` is
  converted into `security/pentest/v0.42.0.md` and the retest is clean.
- Initial pentest found that `scripts/validate-release-metadata.sh` did not
  enforce the v0.42.0 pentest report; this remediation makes the check
  version-derived instead of hand-appended.
- Retest noted that the first remediation extracted versions with line-based
  `sed`; the gate now parses the named TOML tables with `tomllib`.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.3.0`.
- `eth` publishes as `0.42.0`.
- Other support crates are unchanged and are not republished.
