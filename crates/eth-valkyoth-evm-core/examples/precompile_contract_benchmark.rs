//! Public-input work-per-gas evidence for the paid precompile boundary.

use std::{hint::black_box, time::Instant, vec};

use eth_valkyoth_evm_core::{
    EvmBn254Mul, EvmFork, EvmGas, EvmGasMeter, EvmIdentity, EvmPrecompileDescriptor,
    EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus, EvmSha256,
};

const LINEAR_INPUT_BYTES: usize = 1_048_577;
const LINEAR_ROUNDS: u32 = 8;
const CURVE_ROUNDS: u32 = 64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = EvmPrecompileRegistry::try_new(EvmFork::ISTANBUL)?;
    let input = vec![0x5a_u8; LINEAR_INPUT_BYTES];

    let identity = benchmark_identity(registry.descriptor(EvmPrecompileKind::Identity)?, &input)?;
    let sha256 = benchmark_sha256(registry.descriptor(EvmPrecompileKind::Sha256)?, &input)?;
    let bn254_mul = benchmark_bn254_mul(
        registry.descriptor(EvmPrecompileKind::Bn254Mul)?,
        &[0_u8; 96],
    )?;

    println!(
        "input_bytes={LINEAR_INPUT_BYTES} identity_ps_per_gas={identity} \
         sha256_ps_per_gas={sha256} bn254_mul_ps_per_gas={bn254_mul}"
    );
    Ok(())
}

fn benchmark_identity(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<u128, Box<dyn std::error::Error>> {
    let quote = descriptor.quote::<EvmIdentity>(input)?;
    let gas = quote.gas_cost();
    let mut output = vec![0_u8; input.len()];
    let started = Instant::now();
    for _ in 0..LINEAR_ROUNDS {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmIdentity>(input)?
            .authorize(&mut meter, &mut output)?
            .execute_identity();
        require_success(outcome.status())?;
        black_box(&output);
    }
    nanos_per_gas(started.elapsed().as_nanos(), gas, LINEAR_ROUNDS)
}

fn benchmark_sha256(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<u128, Box<dyn std::error::Error>> {
    let quote = descriptor.quote::<EvmSha256>(input)?;
    let gas = quote.gas_cost();
    let mut output = [0_u8; 32];
    let started = Instant::now();
    for _ in 0..LINEAR_ROUNDS {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmSha256>(input)?
            .authorize(&mut meter, &mut output)?
            .execute_sha256();
        require_success(outcome.status())?;
        black_box(output);
    }
    nanos_per_gas(started.elapsed().as_nanos(), gas, LINEAR_ROUNDS)
}

fn benchmark_bn254_mul(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<u128, Box<dyn std::error::Error>> {
    let quote = descriptor.quote::<EvmBn254Mul>(input)?;
    let gas = quote.gas_cost();
    let mut output = [0_u8; 64];
    let started = Instant::now();
    for _ in 0..CURVE_ROUNDS {
        let mut meter = EvmGasMeter::try_new(gas)?;
        let outcome = descriptor
            .quote::<EvmBn254Mul>(input)?
            .authorize(&mut meter, &mut output)?
            .execute_bn254_mul();
        require_success(outcome.status())?;
        black_box(output);
    }
    nanos_per_gas(started.elapsed().as_nanos(), gas, CURVE_ROUNDS)
}

fn require_success(status: EvmPrecompileStatus) -> Result<(), Box<dyn std::error::Error>> {
    if status != EvmPrecompileStatus::Success {
        return Err("paid precompile benchmark call failed".into());
    }
    Ok(())
}

fn nanos_per_gas(
    elapsed_nanos: u128,
    gas: EvmGas,
    rounds: u32,
) -> Result<u128, Box<dyn std::error::Error>> {
    let total_gas = u128::from(gas.get())
        .checked_mul(u128::from(rounds))
        .ok_or("benchmark gas overflow")?;
    elapsed_nanos
        .checked_mul(1_000)
        .ok_or("benchmark duration scale overflow")?
        .checked_div(total_gas)
        .ok_or_else(|| "benchmark gas must be nonzero".into())
}
