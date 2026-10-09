# Charged BLS12-381 G2 Addition

Source milestone: v0.61.0, internal candidate. Pentest is pending.
No crates.io publication before v0.65.0.

## Contract

Prague address `0x0d` admits exactly 512 bytes, two canonical 256-byte G2
points. `EvmBls12G2Add` uses the sealed exact-input quote and private paid
capability. Output capacity and the fixed 600 gas are checked before curve
validation, point addition or normalization. Success writes exactly 256 bytes,
preserves any suffix and consumes 600 gas. No subgroup check is performed:
valid on-curve points outside the prime-order subgroup must be accepted.

Invalid frame length fails quotation. Short output or insufficient gas fails
admission without changing gas/output or performing curve work. Once paid,
invalid coordinates/points produce CALL failure, zero output length, unchanged
caller output and consumption of all remaining supplied child gas. CALL
integration remains responsible for handling admission errors and rollback.
The quote immutably borrows its input; no caller can substitute a different
frame or construct/forget a paid capability. Descriptor changes fail closed.

Inputs are public. Arithmetic is fixed-width, allocation-free and variable-time,
not suitable for secret scalars. Point validation and normalization do not
confer signature validity or subgroup membership. G2ADD uses no square roots;
finite normalization uses one Fp2 inverse. Validation and output encoding are
canonical `x.c0 || x.c1 || y.c0 || y.c1`, with infinity encoded as zero.

```rust
use eth::evm_core::{EvmBls12G2Add, EvmFork, EvmGas, EvmGasMeter,
    EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus};
let registry = EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?;
let descriptor = registry.descriptor(EvmPrecompileKind::Bls12G2Add)?;
let input = [0_u8; 512]; // public infinity + infinity
let mut output = [0_u8; 256];
let mut gas = EvmGasMeter::try_new(EvmGas::new(600))?;
let result = descriptor.quote::<EvmBls12G2Add>(&input)?
    .authorize_and_execute_bls12_g2_add(&mut gas, &mut output)?;
assert_eq!(result.status(), EvmPrecompileStatus::Success);
# Ok::<(), eth::error::EvmCoreError>(())
```

## Tower Conventions

The existing first-party representation is unchanged: Fp2 is
`Fp[v]/(v^2 + 1)`, wire order is `c0 || c1`, and the G2 twist is
`y^2 = x^3 + 4*(1+v)`. Later Fp6/Fp12 code must use `1+v` for the cubic
non-residue and preserve the EIP-2537 M-twist conventions. No Fp6/Fp12,
pairing, maps or subgroup validator is admitted by this release. Subgroup
validation is v0.62.0; the remaining MSM/map/pairing passes run through v0.70.0.

## Verification

- Thread-local test instrumentation counts validations, additions, Fp2
  inversions and roots. Wrong descriptor/length, short output and insufficient
  gas perform none. Paid malformed points preserve output and burn child gas.
- All nine official positive and seven rejection fixtures exercise paid
  execution. The non-subgroup fixture proves addition does not over-reject.
- `bls12381_g2_add` fuzzing compares paid outcomes to an independent BigUint
  polynomial/affine oracle, including malformed input and gas/output limits.
- The charged G2 benchmark checks 1,000 complete finite-point calls and a
  one-second host ceiling; this is not portable gas or side-channel proof.
- `scripts/release_0_61_0_gate.sh --implementation` runs portable checks;
  the rootless client runner is a separate required integration check for this
  milestone. Neither command substitutes for the external pentest.

## Cross-Client Evidence (2026-10-09)

`python3 scripts/run_bls_g2_client_differential.py` matched all 16 official
outcomes against each of the following runtime identities:

| Client | Version | Cases |
| --- | --- | --- |
| Geth | `v1.17.8-stable-a5790770`, Go 1.27.1 | 16 |
| Besu | `v26.9.0`, OpenJDK 25 | 16 |
| Nethermind | `v2.1.0+b3e7e84c-f`, .NET 10.0.12 | 16 |

Immutable image digests live in `scripts/modexp_client_config.py`. The runner
checks official latest release metadata and runtime identity. It requires
rootless Podman CPU/memory/PID controllers, loopback-only RPC, an internal
network, no capabilities, a read-only root, bounded tmpfs/logs and explicit
ownership-checked cleanup. It never disables those constraints to get a pass.

Besu 26.9.0 removed the old dev network. The first attempt failed at startup;
the runner now uses its built-in Ephemery genesis with FULL sync and no P2P.
Geth uses dev mode and Nethermind uses spaceneth. These are isolated RPC
fixture checks, not public-network sync, consensus or full-client conformance.
Nonzero positive outputs distinguish active G2ADD from an inactive empty account.
Negative cases require execution errors, not transport/protocol failures.
Cross-client gas tracing is not claimed; exact gas is verified locally.

## Sources

- [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537), checked 2026-10-09.
- Fixture hashes and source revision remain those recorded in the
  [G2 arithmetic document](bls12-g2-arithmetic.md); no upstream fixture was
  silently replaced during the maintenance refresh.
