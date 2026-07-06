# eth-valkyoth-evm-core

`eth-valkyoth-evm-core` is an internal support crate for
[`eth`](https://crates.io/crates/eth). It provides the dependency-free,
`no_std` EVM core domains used while the first-party audited EVM engine is
built in small release passes.

Most users should depend on `eth` and enable the optional `evm-core` feature:

```toml
[dependencies]
eth = { version = "0.43.1", features = ["evm-core"] }
```

This crate executes only the audited bootstrap opcode subset. It exposes
bounded types for EVM words, stacks, memory, program counters, opcode
classification, fork identifiers, deterministic core errors, explicit
host-state traits, fixed-capacity warm/cold access tracking, historical fork
identifiers, opcode-introduction metadata, and a no-alloc interpreter for stack
arithmetic, bitwise/comparison, stack manipulation, dynamic jumps,
return/revert shells, state reads, `EXTCODECOPY`, and a fail-closed `SSTORE`
shell. Bytecode input is capped at the EIP-170 code-size
ceiling, and valid jump destinations are precomputed once per run with a
fixed-size no-alloc bitset.

## Security posture

- `no_std` by default.
- No allocator requirement for the fixed stack and borrowed memory domains.
- Unsafe code is forbidden.
- Stack, memory, bytecode, execution-step, and gas limits are explicit
  constants.
- State access is available only through explicit host-state traits and
  caller-provided fixed-capacity warm/cold access sets.
- Warm/cold access sets are linear-scan, allocation-free structures; choose
  capacities that are bounded relative to the gas limit and deployment policy.
- Warm/cold state-access gas is admitted only for `EvmFork::LONDON` and later;
  pre-Berlin historical state gas schedules are rejected rather than guessed.
- Historical fork identifiers are explicit through Prague. Amsterdam is known
  to the roadmap but is rejected by the executable table until a concrete scope
  is admitted.
- Unsupported opcodes and unsupported forks are rejected with named errors.
- No call/create, log, precompile, refund, or committed storage-write semantics
  are claimed yet.
