<p align="center">
  <b>default dependency-free no_std EVM core domains for eth.</b><br>
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

First-party `no_std` EVM domains and bounded execution components for
[`eth`](https://crates.io/crates/eth). The default profile has no runtime
dependencies or allocator requirement. Optional node tracking uses the
sanitization bridge to erase retained access metadata.

```sh
cargo add eth --features evm-core
```

## Example

Execute an exact-input, gas-authorized identity precompile:

```rust
use eth::evm_core::{
    EvmFork, EvmGas, EvmGasMeter, EvmIdentity, EvmPrecompileKind,
    EvmPrecompileRegistry, EvmPrecompileStatus,
};

let descriptor = EvmPrecompileRegistry::try_new(EvmFork::CANCUN)?
    .descriptor(EvmPrecompileKind::Identity)?;
let quote = descriptor.quote::<EvmIdentity>(b"eth")?;
let mut output = [0_u8; 3];
let mut gas = EvmGasMeter::try_new(EvmGas::new(18))?;
let outcome = quote.authorize_and_execute_identity(&mut gas, &mut output)?;
assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
assert_eq!(&output, b"eth");
# Ok::<(), eth::error::EvmCoreError>(())
```

## BLS12-381 Arithmetic

Fp/Fp2 field arithmetic and validated G1/G2 affine/projective point operations
are available. Wire-only `G1Point`/`G2Point` types enforce canonical encoding;
the separate `G1Affine`/`G2Affine` types also enforce curve membership.
None establishes prime-subgroup membership.

```rust
use eth::evm_core::{EvmBls12381Fp as Fp, EvmBls12381Fp2 as Fp2, EvmBls12381G2Affine as G2};

let x = Fp2::from_u64(2);
let y = x.square().mul_mod(x)
    .add_mod(Fp2::from_coefficients(Fp::from_u64(4), Fp::from_u64(4)))
    .sqrt().ok_or(eth::error::EvmCoreError::PrecompilePointNotOnCurve)?;
let point = G2::try_from_coordinates(x, y)?;
assert_eq!(point.add_point(point), point.double());
assert!(point.add_point(point.negate()).is_infinity());
# Ok::<(), eth::error::EvmCoreError>(())
```

These fixed-size APIs are variable-time and **public-input only**. They must
not process signing secrets. Standalone arithmetic is not paid execution
authority. Charged G1ADD is available at Prague address `0x0b` with exact
256-byte input, 375 gas and atomic 128-byte output. G2ADD admission is assigned
to v0.61.0; subgroup, MSM, mapping and pairing follow in v0.62.0 through v0.70.0.
KZG execution remains unavailable.

See [G2 arithmetic](https://github.com/valkyoth/eth/blob/main/docs/bls12-g2-arithmetic.md),
[charged G1](https://github.com/valkyoth/eth/blob/main/docs/bls12-g1-addition.md)
and the [fork/opcode matrix](https://github.com/valkyoth/eth/blob/main/docs/evm-fork-matrix.md).
The interpreter is still a bounded subset, not a complete state-transition
engine: call/create planning and recognized unsupported operations fail closed.

## Security posture

- `no_std` by default.
- No allocator requirement for the fixed stack and borrowed memory domains.
- Unsafe code is forbidden.
- Stack, memory, bytecode, execution-step, and gas limits are explicit
  constants.
- Caller-provided EVM memory is zero-initialized on construction. Execution
  contexts are one-shot until an explicit destructive `reset`, preventing
  stack, memory, or program-counter reuse across requests.
- State access is available only through explicit host-state traits and an
  injected access tracker.
- Failed and reverted stateful runs restore the caller's warm/cold access
  checkpoint so retries and reverted scopes cannot inherit discounted access
  costs.
- `EvmEmbeddedAccessTracker` is an explicit fixed-capacity, linear-scan,
  allocation-free profile. The optional `alloc` feature adds
  `EvmNodeAccessTracker`, which pre-reserves bounded compressed-radix indexes
  and undo storage at construction. Lookup and insertion are bounded by fixed
  Ethereum key width without later allocation. Both implement nested LIFO
  EIP-2929 scope rollback; node rollback touches only post-checkpoint
  insertions and erases removed keys on rollback, reset, and drop.
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
  gas-bounded ModExp, BN254 add/mul, BN254 pairing frames, BLAKE2F, and
  ECRECOVER can execute without default crypto dependencies; ECRECOVER requires
  caller-provided secp256k1 and Keccak backend traits. BN254 pairing validates
  bounded frames, G2 subgroup membership, tuple streaming,
  line-function arithmetic, Miller-loop accumulation with sparse line-factor
  multiplication, optimized bounded final exponentiation, Frobenius Q1/-Q2
  point mapping, and the projective post-loop line carrier, then writes
  canonical EIP-197 zero/one output words. BLAKE2F executes EIP-152 exact-length frames with
  final-flag validation and round-count gas.
  Charged EIP-2537 G1 addition is admitted. Other KZG/BLS precompiles
  remain bounded plans and fail closed. EIP-2537 fixed frames,
  non-empty MSM/pairing lists, output lengths, discount gas, and pairing gas
  are enforced at planning time.
- Every executable precompile requires an immutable exact-input quote and
  atomic typed execution. Canonical registry validation, output admission, and
  gas charging happen before expensive work. Safe Rust cannot mutate or
  substitute the quoted input while the quote is live.
- ModExp keeps declared lengths as 256-bit values through gas admission and
  executes every payable Prague-era EIP-198/EIP-2565 frame with caller-owned
  workspace and no private operand ceiling. See the
  [ModExp contract](https://github.com/valkyoth/eth/blob/main/docs/modexp-precompile.md).
- Armed paid authority is crate-private and cannot be named, dropped, or
  forgotten by external safe Rust. Its internal drop guard consumes the
  complete dedicated child meter on unwind; public outcomes are must-use.
- `EvmPrecompileOutcome` reports one precise success or call failure, gas
  consumed, output length, and bounded error. Execution failures consume all
  gas supplied to the dedicated precompile meter and request CALL rollback.
- A false `JUMPI` does not convert or validate the unused destination word;
  true branches retain canonical `JUMPDEST` validation.
- `EXTCODECOPY` ignores both offsets for zero-length copies and zero-fills any
  code offset outside the release code-size domain without invoking the host.
- Unsupported opcodes and unsupported forks are rejected with named errors.
- No nested call/create execution, log, remaining cryptographic precompile,
  refund, or committed storage-write semantics are claimed yet.

## License

MIT OR Apache-2.0, at your option.
