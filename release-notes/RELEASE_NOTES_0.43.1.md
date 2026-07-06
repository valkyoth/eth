# eth 0.43.1 Release Notes

Status: release candidate; pentest clean; awaiting final GitHub checks.

`0.43.1` adds the native EVM historical fork matrix. The release makes older
Ethereum execution forks explicit in `eth-valkyoth-evm-core` before later
stateful execution work builds on fork selection.

This release does not implement pre-Berlin state gas schedules. State opcodes
under Frontier through Berlin still fail closed until `0.43.2` implements and
tests historical pricing.

## Added

- Historical `EvmFork` identifiers for Homestead, Tangerine Whistle, Spurious
  Dragon, Byzantium, Constantinople, Petersburg, Istanbul, Berlin, and
  Amsterdam.
- `EvmFork::is_known` for roadmap-known fork identifiers.
- `EvmFork::supports_warm_cold_state_access` for the currently claimed
  London-through-Prague state gas model.
- `EvmFork::opcode_introduced_in` and `EvmFork::opcode_is_introduced` for the
  modeled opcode subset.
- `docs/evm-fork-matrix.md` documenting native fork support, protocol
  alignment, opcode introduction boundaries, and deferred historical gas work.

## Changed

- `EvmFork` numeric identifiers now use a chronological historical fork
  ordering instead of the compressed bootstrap ordering from `0.43.0`.
  These identifiers are crate-local table keys and must not be persisted as
  consensus, network, or wire identifiers.
- `OpcodeTable::instruction` now checks opcode-introduction boundaries before
  returning metadata.
- `REVERT` is unavailable before Byzantium in the opcode table.
- `EXTCODEHASH` is recognized as a Constantinople opcode and `SELFBALANCE` as
  an Istanbul opcode, while executable state access remains claimed only for
  the London-and-later warm/cold model.
- `eth-valkyoth-evm-core` publishes as `0.5.0`.
- `eth` publishes as `0.43.1` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.5.0`.

## Security Notes

- Pre-Berlin state execution remains fail-closed. The fork matrix must not be
  read as a historical gas-schedule claim.
- `EvmFork::AMSTERDAM` is known to the roadmap but unsupported by
  `OpcodeTable::try_new` until a concrete fork scope is admitted.
- Unsupported opcodes and unsupported fork identifiers return named errors
  instead of silently falling through to a later fork model.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo check -p eth --features evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/release_crates.py --check`
- `scripts/release_0_43_1_gate.sh`

## Pentest

- Permanent report: `security/pentest/v0.43.1.md`.
- Initial pentest found an interpreter-vs-`OpcodeTable` fork-boundary
  enforcement gap and a duplicated warm/cold support ceiling. Both have been
  remediated before retest.
- Retest was clean. No blocking findings remain for the v0.43.1 release scope.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.5.0`.
- `eth` publishes as `0.43.1`.
- Other support crates are unchanged and are not republished.
