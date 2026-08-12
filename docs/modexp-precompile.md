# ModExp Precompile

Status: `v0.55.0` implements the dependency-free Prague-era EIP-198 and
EIP-2565 contract without the former 64-byte private operand ceiling.

## Consensus Scope

For supported forks from Byzantium through Prague, address `0x05`:

- reads three 32-byte big-endian lengths without narrowing them;
- treats missing calldata as infinite right-zero-padding and ignores surplus;
- charges EIP-198 gas before Berlin and EIP-2565 gas from Berlin;
- returns `(base ** exponent) % modulus` at exactly the declared modulus width;
- returns no bytes for a zero modulus length and all-zero bytes for a zero
  modulus value.

The parser stores each declared length in `EvmModExpLength`. A payable quote is
the authority for converting those values to host `usize`. If the canonical
gas exceeds this crate's reviewed `EVM_MAX_GAS_LIMIT`, the quote records
`EVM_MAX_GAS_LIMIT + 1`; every constructible meter then rejects it as
`OutOfGas` without output allocation or arithmetic.

Osaka-era EIP-7823 input limits and EIP-7883 gas changes are not claimed by the
currently supported Prague fork table. They are explicit work under
`v0.115.0` fork-manifest admission and `v0.116.0` current-fork execution, not
an implicit extension of the Prague rules implemented here.

## Workspace Contract

The engine remains dependency-free and `no_std`. It does not allocate from
attacker-declared lengths. Integrators ask the exact quote for
`modexp_workspace_limbs()`, provide that many `u32` limbs through
`EvmModExpWorkspace`, and call `authorize_and_execute_modexp`.

```rust
use eth::evm_core::{
    EvmFork, EvmGasMeter, EvmModExpWorkspace, EvmModexp, EvmPrecompileKind,
    EvmPrecompileRegistry, EvmPrecompileStatus,
};

let mut input = [0_u8; 102];
input[31] = 1; // base length
input[63] = 1; // exponent length
input[95] = 3; // modulus length
input[96..].copy_from_slice(&[5, 3, 0, 0, 7, 0]);

let descriptor = EvmPrecompileRegistry::try_new(EvmFork::BERLIN)?
    .descriptor(EvmPrecompileKind::Modexp)?;
let quote = descriptor.quote::<EvmModexp>(&input)?;
let mut output = [0_u8; 3];
let mut storage = [0_u32; 7];
let mut workspace = EvmModExpWorkspace::new(&mut storage);
let mut gas = EvmGasMeter::try_new(quote.gas_cost())?;
let outcome = quote.authorize_and_execute_modexp(&mut gas, &mut output, &mut workspace)?;

assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
assert_eq!(output, [0, 0, 6]);
# Ok::<(), eth::error::EvmCoreError>(())
```

Workspace admission and output capacity are checked before charging. Expensive
arithmetic starts only after the exact immutable input quote is charged. The
engine computes in caller storage and copies the final result to output only
after successful completion, so pre-authorization failures leave output and
the gas meter unchanged.

The current implementation uses little-endian `u32` limbs, schoolbook
multiplication, normalized long division, binary exponentiation, and streamed
base reduction. Workspace is six limbs per declared modulus limb plus one
guard limb. Virtual segment reads provide zero padding without materializing
absent base, exponent, or modulus bytes.

## Security Boundary

ModExp inputs are public EVM data. The implementation is intentionally not
constant-time and must not be reused for RSA private exponents, private keys,
or other secret-dependent arithmetic.

Release evidence includes:

- the official EIP-198 Fermat vector;
- zero, truncation, padding, output, wide-length, and insufficient-workspace
  tests;
- deterministic `u128` oracle comparisons across limb boundaries;
- dev-only `num-bigint 0.5.1` differential cases from 1 through 256 bytes plus
  leading-zero, even, zero, unequal-width, sparse, and truncated operands;
- the bounded `modexp_frame` fuzz target, which requires every fully admitted
  payable frame to succeed;
- Berlin-priced dense 64/256-byte and sparse/zero 1,024-byte work-per-gas
  benchmarks.
