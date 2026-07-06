# eth 0.43.2 Release Notes

Status: implementation ready; awaiting pentest.

`0.43.2` adds historical state-read gas schedules to the dependency-free
native EVM core. The release replaces the temporary pre-Berlin state-access
fail-closed behavior with explicit fork-specific pricing for the currently
executable state-read subset.

This is still a bootstrap EVM engine. It does not claim calls, creates, logs,
precompiles, refunds, committed storage writes, or full official state-test
execution.

## Added

- Historical flat state-read pricing for Frontier and Homestead.
- EIP-150 IO repricing for Tangerine Whistle through Byzantium.
- EIP-1052 `EXTCODEHASH` pricing for Constantinople and Petersburg.
- EIP-1884 `BALANCE`, `EXTCODEHASH`, `SLOAD`, and `SELFBALANCE` pricing for
  Istanbul.
- EIP-2929 warm/cold state access accounting beginning at Berlin.
- Dedicated historical gas tests for each claimed pricing boundary.

## Changed

- `EvmFork::supports_warm_cold_state_access` now begins at Berlin instead of
  London.
- `OpcodeTable::instruction` now admits state opcode metadata for historical
  forks once the opcode exists at that fork.
- Pre-Berlin state reads no longer mutate the warm/cold access tracker.
- Account-read gas is selected by opcode and fork instead of by one shared
  latest-like account-access cost.
- `SSTORE` remains a write shell, but its storage access precharge now follows
  the selected fork before returning `StateWriteUnsupported`.
- `docs/evm-fork-matrix.md` documents the exact claimed state-read gas table.

## Security Notes

- Historical pricing is claimed only for the currently executable state-read
  subset: `BALANCE`, `EXTCODESIZE`, `EXTCODECOPY`, `EXTCODEHASH`,
  `SELFBALANCE`, `SLOAD`, and the fail-closed `SSTORE` shell.
- The native engine still rejects unimplemented behavior with named errors
  instead of silently falling through to later fork rules.
- Pre-Berlin access does not consume warm/cold access-set capacity, preventing
  Berlin-only accounting structures from becoming a historical fork dependency.
- Gas constants were checked against Ethereum EIPs 150, 1052, 1884, 2200, and
  2929 before implementation.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo check -p eth --features evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/release_crates.py --check`
- `scripts/release_0_43_2_gate.sh`

## Pentest

- Pending. The release must not be tagged until the local `PENTEST.md` is
  converted into `security/pentest/v0.43.2.md` and the retest is clean.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.6.0`.
- `eth` publishes as `0.43.2` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.6.0`.
- Other support crates are unchanged and are not republished.
