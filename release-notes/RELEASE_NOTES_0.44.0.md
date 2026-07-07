# eth 0.44.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.44.0` adds the native EVM call/create safety boundary. The release admits
CALL, CALLCODE, DELEGATECALL, STATICCALL, CREATE, and CREATE2 as explicit
fork-aware domains, validates their stack/memory/policy shape, and then fails
closed before host calls or state commits.

This is not full nested EVM execution. It deliberately avoids hidden host
behavior until gas forwarding, returndata copying, value transfer, account
creation, and journaled state writes are implemented and pentested.

## Added

- `EvmCallKind` and `EvmCreateKind` domains.
- `EvmCallFramePolicy` with the 1024-frame depth limit, static-frame handling,
  CALL value restriction, and create rejection in static frames.
- `EvmCallPlan` and `EvmCreatePlan` validation domains.
- `EvmReturnDataRange` for bounded return-data copy validation.
- `EvmJournal` and `EvmJournalCheckpoint` as a fixed-capacity LIFO checkpoint
  policy for future commit/revert work.
- Opcode constants and metadata for CALL, CALLCODE, DELEGATECALL, STATICCALL,
  CREATE, and CREATE2.
- Interpreter planning paths that validate call/create operands and memory
  ranges, then return `CallCreateExecutionUnsupported` without popping stack
  operands.
- Tests for opcode introduction boundaries, depth/static policy, return-data
  range checks, journal LIFO behavior, and fail-closed interpreter handling.

## Changed

- `OpcodeTable::instruction` now classifies admitted call/create opcodes as
  `OpcodeClass::CallCreate`.
- `EvmFork::opcode_introduced_in` now records:
  - CREATE, CALL, and CALLCODE at Frontier;
  - DELEGATECALL at Homestead;
  - STATICCALL at Byzantium;
  - CREATE2 at Constantinople.
- `docs/evm-fork-matrix.md` documents the call/create safety boundary.
- `eth-valkyoth-evm-core` publishes as `0.7.0`.
- `eth` publishes as `0.44.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.7.0`.

## Security Notes

- Call/create opcodes do not perform host calls, value transfers, account
  creation, returndata copying, or state commits in this release.
- CALL with non-zero value is rejected in a static frame.
- CREATE and CREATE2 are rejected in a static frame.
- Journal commit/revert is explicit and LIFO; mismatched checkpoints fail with
  named errors.
- All call/create interpreter paths preserve stack operands before returning an
  error.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo check -p eth --features evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/release_crates.py --check`
- `scripts/release_0_44_gate.sh`

## Pentest

- Pending. The release must not be tagged until the local `PENTEST.md` is
  converted into `security/pentest/v0.44.0.md` and the retest is clean.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.7.0`.
- `eth` publishes as `0.44.0`.
- Other support crates are unchanged and are not republished.
