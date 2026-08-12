//! Release-blocking adversarial work-per-gas evidence for native precompiles.

#[path = "support/precompile_benchmark_inputs.rs"]
mod inputs;

use std::{hint::black_box, io, time::Instant, vec};

use eth_valkyoth_evm_core::{
    EvmBlake2F, EvmBn254Mul, EvmBn254Pairing, EvmFork, EvmGas, EvmGasMeter, EvmIdentity, EvmModexp,
    EvmPrecompileDescriptor, EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus,
    EvmRipemd160, EvmSha256,
};

const LINEAR_INPUT_BYTES: usize = 1_048_577;
const HASH_ROUNDS: u32 = 4;
const BN254_MUL_ROUNDS: u32 = 16;
const BN254_PAIRING_ROUNDS: u32 = 2;
const MODEXP_ROUNDS: u32 = 2;
const MODEXP_LEGACY_BYTES: usize = 64;
const MODEXP_WIDE_BYTES: usize = 256;
const BLAKE2F_ROUNDS: u32 = 100_000;
const BLAKE2F_SAMPLES: u32 = 4;

// Reviewed ceilings intentionally leave broad headroom for the slowest CI
// deployment class while still failing large algorithmic regressions.
const MAX_IDENTITY_PS_PER_GAS: u128 = 10_000;
const MAX_SHA256_PS_PER_GAS: u128 = 100_000;
const MAX_RIPEMD160_PS_PER_GAS: u128 = 50_000;
const MAX_BN254_MUL_PS_PER_GAS: u128 = 500_000;
const MAX_BN254_PAIRING_PS_PER_GAS: u128 = 1_000_000;
const MAX_MODEXP_PS_PER_GAS: u128 = 5_000_000;
const MAX_BLAKE2F_PS_PER_GAS: u128 = 100_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = EvmPrecompileRegistry::try_new(EvmFork::ISTANBUL)?;
    let linear = vec![0x5a_u8; LINEAR_INPUT_BYTES];
    let bn254_mul_input = inputs::dense_bn254_mul();
    let pairing_input = inputs::generator_bn254_pairing();
    let modexp_legacy_input = inputs::dense_modexp(MODEXP_LEGACY_BYTES);
    let modexp_wide_input = inputs::dense_modexp(MODEXP_WIDE_BYTES);
    let blake2f_input = inputs::high_round_blake2f(BLAKE2F_ROUNDS);

    let metrics = [
        (
            "identity",
            benchmark_identity(registry.descriptor(EvmPrecompileKind::Identity)?, &linear)?,
            MAX_IDENTITY_PS_PER_GAS,
        ),
        (
            "sha256",
            benchmark_sha256(registry.descriptor(EvmPrecompileKind::Sha256)?, &linear)?,
            MAX_SHA256_PS_PER_GAS,
        ),
        (
            "ripemd160",
            benchmark_ripemd160(registry.descriptor(EvmPrecompileKind::Ripemd160)?, &linear)?,
            MAX_RIPEMD160_PS_PER_GAS,
        ),
        (
            "bn254_mul",
            benchmark_bn254_mul(
                registry.descriptor(EvmPrecompileKind::Bn254Mul)?,
                &bn254_mul_input,
            )?,
            MAX_BN254_MUL_PS_PER_GAS,
        ),
        (
            "bn254_pairing",
            benchmark_bn254_pairing(
                registry.descriptor(EvmPrecompileKind::Bn254Pairing)?,
                &pairing_input,
            )?,
            MAX_BN254_PAIRING_PS_PER_GAS,
        ),
        (
            "modexp_legacy",
            benchmark_modexp(
                registry.descriptor(EvmPrecompileKind::Modexp)?,
                &modexp_legacy_input,
            )?,
            MAX_MODEXP_PS_PER_GAS,
        ),
        (
            "modexp_wide",
            benchmark_modexp(
                registry.descriptor(EvmPrecompileKind::Modexp)?,
                &modexp_wide_input,
            )?,
            MAX_MODEXP_PS_PER_GAS,
        ),
        (
            "blake2f",
            benchmark_blake2f(
                registry.descriptor(EvmPrecompileKind::Blake2F)?,
                &blake2f_input,
            )?,
            MAX_BLAKE2F_PS_PER_GAS,
        ),
    ];

    for (name, measured, ceiling) in metrics {
        println!("{name}_ps_per_gas={measured} ceiling={ceiling}");
        enforce_ceiling(name, measured, ceiling)?;
    }
    Ok(())
}

fn benchmark_identity(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<u128, Box<dyn std::error::Error>> {
    let gas = descriptor.quote::<EvmIdentity>(input)?.gas_cost();
    let mut output = vec![0_u8; input.len()];
    measure(gas, HASH_ROUNDS, || {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmIdentity>(input)?
            .authorize_and_execute_identity(&mut meter, &mut output)?;
        require_success(outcome.status())?;
        black_box(&output);
        Ok(())
    })
}

fn benchmark_sha256(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<u128, Box<dyn std::error::Error>> {
    let gas = descriptor.quote::<EvmSha256>(input)?.gas_cost();
    let mut output = [0_u8; 32];
    measure(gas, HASH_ROUNDS, || {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmSha256>(input)?
            .authorize_and_execute_sha256(&mut meter, &mut output)?;
        require_success(outcome.status())?;
        black_box(&output);
        Ok(())
    })
}

fn benchmark_ripemd160(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<u128, Box<dyn std::error::Error>> {
    let gas = descriptor.quote::<EvmRipemd160>(input)?.gas_cost();
    let mut output = [0_u8; 32];
    measure(gas, HASH_ROUNDS, || {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmRipemd160>(input)?
            .authorize_and_execute_ripemd160(&mut meter, &mut output)?;
        require_success(outcome.status())?;
        black_box(&output);
        Ok(())
    })
}

fn benchmark_bn254_mul(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8; 96],
) -> Result<u128, Box<dyn std::error::Error>> {
    let gas = descriptor.quote::<EvmBn254Mul>(input)?.gas_cost();
    let mut output = [0_u8; 64];
    measure(gas, BN254_MUL_ROUNDS, || {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmBn254Mul>(input)?
            .authorize_and_execute_bn254_mul(&mut meter, &mut output)?;
        require_success(outcome.status())?;
        black_box(&output);
        Ok(())
    })
}

fn benchmark_bn254_pairing(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8; 192],
) -> Result<u128, Box<dyn std::error::Error>> {
    let gas = descriptor.quote::<EvmBn254Pairing>(input)?.gas_cost();
    let mut output = [0_u8; 32];
    measure(gas, BN254_PAIRING_ROUNDS, || {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmBn254Pairing>(input)?
            .authorize_and_execute_bn254_pairing(&mut meter, &mut output)?;
        require_success(outcome.status())?;
        black_box(&output);
        Ok(())
    })
}

fn benchmark_modexp(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<u128, Box<dyn std::error::Error>> {
    let gas = descriptor.quote::<EvmModexp>(input)?.gas_cost();
    let workspace_limbs = eth_valkyoth_evm_core::modexp_workspace_limbs(input)?;
    let mut workspace_storage = vec![0_u32; workspace_limbs];
    let output_len = descriptor.quote::<EvmModexp>(input)?.output_len();
    let mut output = vec![0_u8; output_len];
    measure(gas, MODEXP_ROUNDS, || {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let mut workspace = eth_valkyoth_evm_core::EvmModExpWorkspace::new(&mut workspace_storage);
        let outcome = descriptor
            .quote::<EvmModexp>(input)?
            .authorize_and_execute_modexp(&mut meter, &mut output, &mut workspace)?;
        require_success(outcome.status())?;
        black_box(&output);
        Ok(())
    })
}

fn benchmark_blake2f(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8; 213],
) -> Result<u128, Box<dyn std::error::Error>> {
    let gas = descriptor.quote::<EvmBlake2F>(input)?.gas_cost();
    let mut output = [0_u8; 64];
    measure(gas, BLAKE2F_SAMPLES, || {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmBlake2F>(input)?
            .authorize_and_execute_blake2f(&mut meter, &mut output)?;
        require_success(outcome.status())?;
        black_box(&output);
        Ok(())
    })
}

fn measure<F>(gas: EvmGas, rounds: u32, mut execute: F) -> Result<u128, Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<(), Box<dyn std::error::Error>>,
{
    execute()?;
    let started = Instant::now();
    for _ in 0..rounds {
        execute()?;
    }
    picoseconds_per_gas(started.elapsed().as_nanos(), gas, rounds)
}

fn require_success(status: EvmPrecompileStatus) -> Result<(), Box<dyn std::error::Error>> {
    if status != EvmPrecompileStatus::Success {
        return Err(io::Error::other("paid precompile benchmark call failed").into());
    }
    Ok(())
}

fn enforce_ceiling(
    name: &str,
    measured: u128,
    ceiling: u128,
) -> Result<(), Box<dyn std::error::Error>> {
    if measured > ceiling {
        return Err(io::Error::other(format!(
            "{name} work-per-gas regression: {measured} > {ceiling} ps/gas"
        ))
        .into());
    }
    Ok(())
}

fn picoseconds_per_gas(
    elapsed_nanos: u128,
    gas: EvmGas,
    rounds: u32,
) -> Result<u128, Box<dyn std::error::Error>> {
    let total_gas = u128::from(gas.get())
        .checked_mul(u128::from(rounds))
        .ok_or_else(|| io::Error::other("benchmark gas overflow"))?;
    elapsed_nanos
        .checked_mul(1_000)
        .ok_or_else(|| io::Error::other("benchmark duration scale overflow"))?
        .checked_div(total_gas)
        .ok_or_else(|| io::Error::other("benchmark gas must be nonzero").into())
}
