# eth 0.40.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.40.0` starts the native first-party EVM engine path with
`eth-valkyoth-evm-core`. This release adds dependency-free `no_std` EVM core
domains only; it does not execute bytecode and does not admit REVM or another
execution backend.

## Added

- New `eth-valkyoth-evm-core` crate.
- `EvmWord` for canonical 256-bit stack words.
- `EvmStack` with fixed compile-time capacity, no allocator dependency, and
  explicit overflow, underflow, zero-capacity, and too-large-capacity errors.
- `EvmMemory` borrowed memory view with a hard release cap and checked byte
  reads/writes.
- `ProgramCounter` with checked advancement.
- `EvmOpcode`, `OpcodeClass`, `OpcodeInfo`, `EvmFork`, and `OpcodeTable` as a
  fork-aware opcode table skeleton.
- `EvmCoreError` with stable error codes and optional `std::error::Error`
  support.
- Optional facade feature `evm-core`, re-exported as `eth::evm_core`.
- Facade error re-export for `EvmCoreError` when `evm-core` is enabled.
- `scripts/release_0_40_gate.sh` captures default, `evm`, `evm-core`, and
  all-feature dependency tree evidence.

## Changed

- `eth` publishes as `0.40.0`.
- `eth-valkyoth-evm-core` publishes as `0.1.0`.
- Existing support crates keep their previous published versions.

## Security Notes

- The new crate has no runtime dependencies.
- Unsafe code is forbidden.
- Stack and memory limits are explicit constants.
- Stack operations clear vacated slots when popping.
- Memory access uses checked slice access and returns named bounds errors.
- Unsupported opcodes and unsupported forks return explicit errors.
- Opcode metadata is a skeleton for future execution passes, not an execution
  claim.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo check -p eth --features evm-core`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo deny check`
- `cargo audit`
- `scripts/release_0_40_gate.sh`

## Pentest

- Pending. The release must not be tagged until the local `PENTEST.md` is
  converted into `security/pentest/v0.40.0.md` and the retest is clean.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.1.0`.
- `eth` publishes as `0.40.0`.
- Other support crates are unchanged and are not republished.
