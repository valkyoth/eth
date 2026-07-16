<p align="center">
  <b>dependency-free no_std EVM core domains for eth.</b><br>
  Explicit domains, bounded decode policy, first-party EVM work, and security-gated release evidence.
</p>

<div align="center">
  <a href="https://crates.io/crates/eth">eth crate</a>
  |
  <a href="https://docs.rs/eth-valkyoth-evm-core">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/eth/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/eth">
    <img src="https://raw.githubusercontent.com/valkyoth/eth/main/.github/images/eth.webp" alt="eth Rust crate overview">
  </a>
</p>

# eth-valkyoth-evm-core

`eth-valkyoth-evm-core` is an internal support crate for
[`eth`](https://crates.io/crates/eth). It provides the dependency-free,
`no_std` EVM core domains used while the first-party audited EVM engine is
built in small release passes.

Most users should depend on `eth` and enable the optional `evm-core` feature:

```toml
[dependencies]
eth = { version = "0.52.1", features = ["evm-core"] }
```

This crate executes only the audited bootstrap opcode subset. It exposes
bounded types for EVM words, stacks, memory, program counters, opcode
classification, fork identifiers, deterministic core errors, explicit
host-state traits, fixed-capacity warm/cold access tracking, historical fork
identifiers, opcode-introduction metadata, and a no-alloc interpreter for stack
arithmetic, bitwise/comparison, stack manipulation, dynamic jumps,
return/revert shells, state reads, `EXTCODECOPY`, and a fail-closed `SSTORE`
shell. It also exposes call/create planning domains for frame depth, static
write protection, return-data ranges, journal checkpoint policy, and a
fork-aware precompile registry with bounded input/gas planning. Bytecode input
is capped at the EIP-170 code-size ceiling, precompile input planning is capped
at a release hard limit, and valid jump destinations are precomputed once per
run with a fixed-size no-alloc bitset.
The Frontier identity, SHA-256, RIPEMD-160, bounded Byzantium ModExp,
BN254 add/mul, BN254 pairing frames, and Istanbul BLAKE2F execute through
first-party dependency-free implementations. ECRECOVER executes through
explicit caller-provided secp256k1 and Keccak backend traits. KZG and BLS
cryptographic execution remains fail closed, while their descriptors now carry
exact EIP-4844/EIP-2537 input, output, and gas plans. Canonical EIP-2537 Fp,
Fr, Fp2, unrestricted MSM scalar, G1/G2 coordinate, infinity, and complete
precompile-frame parsing is available without allocation. Parsed point values
are wire-valid only; curve and subgroup validation remains fail closed until
the assigned arithmetic releases.

```rust
use eth::evm_core::{EVM_BLS12381_G1_POINT_BYTES, EvmBls12381G1Point};

let encoded = [0_u8; EVM_BLS12381_G1_POINT_BYTES];
let point = EvmBls12381G1Point::try_from_be_bytes(&encoded)?;
assert!(point.is_infinity());
# Ok::<(), eth::error::EvmCoreError>(())
```

The complete wire contract is documented in
[`docs/bls12-381-wire-encodings.md`](https://github.com/valkyoth/eth/blob/main/docs/bls12-381-wire-encodings.md).

## Security posture

- `no_std` by default.
- No allocator requirement for the fixed stack and borrowed memory domains.
- Unsafe code is forbidden.
- Stack, memory, bytecode, execution-step, and gas limits are explicit
  constants.
- Caller-provided EVM memory is zero-initialized on construction. Execution
  contexts are one-shot until an explicit destructive `reset`, preventing
  stack, memory, or program-counter reuse across requests.
- State access is available only through explicit host-state traits and
  caller-provided fixed-capacity warm/cold access sets.
- Failed stateful runs restore the caller's warm/cold access tracker so an
  out-of-gas retry cannot inherit discounted access costs.
- Warm/cold access sets are linear-scan, allocation-free structures; choose
  capacities that are bounded relative to the gas limit and deployment policy.
- Frontier through Istanbul use explicit flat historical state-read pricing for
  the currently executable subset; Berlin and later use warm/cold state-access
  gas.
- Historical fork identifiers are explicit through Prague. Amsterdam is known
  to the roadmap but is rejected by the executable table until a concrete scope
  is admitted.
- Call/create opcodes are recognized, stack/memory/policy validated, and then
  rejected with `CallCreateExecutionUnsupported`; no hidden host calls or
  state commits occur. Zero-length ranges canonicalize their irrelevant offset
  to zero without host-width conversion or memory expansion.
- Precompile descriptors are fork-aware. Identity, SHA-256, RIPEMD-160,
  bounded ModExp, BN254 add/mul, BN254 pairing frames, BLAKE2F, and
  ECRECOVER can execute without default crypto dependencies; ECRECOVER requires
  caller-provided secp256k1 and Keccak backend traits. BN254 pairing validates
  bounded frames, G2 subgroup membership, tuple streaming,
  line-function arithmetic, Miller-loop accumulation with sparse line-factor
  multiplication, optimized bounded final exponentiation, Frobenius Q1/-Q2
  point mapping, and the projective post-loop line carrier, then writes
  canonical EIP-197 zero/one output words. BLAKE2F executes EIP-152 exact-length frames with
  final-flag validation and round-count gas.
  KZG and BLS cryptographic precompiles are bounded plans only and fail closed
  until their first-party implementations are admitted. EIP-2537 fixed frames,
  non-empty MSM/pairing lists, output lengths, discount gas, and pairing gas
  are enforced at planning time.
- Every executable precompile recomputes gas from the actual input immediately
  before charging. A same-length input with a different content-dependent
  BLAKE2F or ModExp cost is rejected before arithmetic or output mutation.
- A false `JUMPI` does not convert or validate the unused destination word;
  true branches retain canonical `JUMPDEST` validation.
- `EXTCODECOPY` ignores both offsets for zero-length copies and zero-fills any
  code offset outside the release code-size domain without invoking the host.
- Unsupported opcodes and unsupported forks are rejected with named errors.
- No nested call/create execution, log, remaining cryptographic precompile,
  refund, or committed storage-write semantics are claimed yet.
