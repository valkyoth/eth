# eth-valkyoth-evm-core

`eth-valkyoth-evm-core` is an internal support crate for
[`eth`](https://crates.io/crates/eth). It provides the dependency-free,
`no_std` EVM core domains used while the first-party audited EVM engine is
built in small release passes.

Most users should depend on `eth` and enable the optional `evm-core` feature:

```toml
[dependencies]
eth = { version = "0.41.0", features = ["evm-core"] }
```

This crate executes only the first audited basic opcode subset. It exposes
bounded types for EVM words, stacks, memory, program counters, opcode
classification, fork identifiers, deterministic core errors, and a no-alloc
interpreter for stack arithmetic, bitwise/comparison, stack manipulation,
dynamic jumps, and return/revert shells.

## Security posture

- `no_std` by default.
- No allocator requirement for the fixed stack and borrowed memory domains.
- Unsafe code is forbidden.
- Stack, memory, and execution-step limits are explicit constants.
- Unsupported opcodes and unsupported forks are rejected with named errors.
- No gas, state, call/create, log, or precompile execution is claimed yet.
