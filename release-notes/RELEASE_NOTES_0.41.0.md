# eth 0.41.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.41.0` adds the first deterministic native EVM opcode execution pass to
`eth-valkyoth-evm-core`. The interpreter is intentionally narrow: it executes
basic stack arithmetic, bitwise/comparison, stack manipulation, program-counter,
dynamic jump, and return/revert shell opcodes without gas accounting, state
access, calls, creates, logs, precompiles, or host behavior.

## Added

- `EvmExecution` no-alloc interpreter context over a fixed-capacity
  `EvmStack` and caller-provided `EvmMemory`.
- `ExecutionLimits` with a non-zero maximum instruction count and a release
  hard ceiling.
- `ExecutionStatus` and `ExecutionReport` for stopped, returned, and reverted
  execution outcomes.
- Wrapping 256-bit `EvmWord` arithmetic for `ADD`, `MUL`, and `SUB`.
- `EvmWord` bitwise, comparison, zero, boolean, `usize`, and bounded
  big-endian slice helpers.
- Opcode execution for `STOP`, `ADD`, `MUL`, `SUB`, `LT`, `GT`, `EQ`,
  `ISZERO`, `AND`, `OR`, `XOR`, `NOT`, `POP`, `PC`, `PUSH1..=PUSH32`,
  `DUP1..=DUP16`, `SWAP1..=SWAP16`, `JUMP`, `JUMPI`, `JUMPDEST`, `RETURN`,
  and `REVERT`.
- Dynamic jump validation that accepts only real `JUMPDEST` bytes outside PUSH
  immediate data.
- Fail-closed errors for zero or oversized step limits, step-limit exhaustion,
  truncated PUSH immediates, invalid jump destinations, oversized word inputs,
  and out-of-range return/revert memory spans.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.2.0`.
- `eth` publishes as `0.41.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.2.0`.

## Security Notes

- This release still does not admit REVM or another external execution engine.
- Gas accounting is not implemented yet; no production execution validity claim
  is made until the fork-aware gas pass lands.
- State access, calls, creates, logs, precompiles, and memory load/store
  execution remain unsupported and fail closed.
- `RETURN` and `REVERT` only report checked memory ranges; they do not copy
  output bytes or commit state.
- Step limits are mandatory and capped so malformed code cannot run
  indefinitely or request impractically large local runs.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo check -p eth --features evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/release_0_41_gate.sh`

## Pentest

- Pending. The release must not be tagged until the local `PENTEST.md` is
  converted into `security/pentest/v0.41.0.md` and the retest is clean.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.2.0`.
- `eth` publishes as `0.41.0`.
- Other support crates are unchanged and are not republished.
