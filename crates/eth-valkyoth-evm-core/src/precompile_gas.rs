use crate::{
    EvmCoreError, EvmFork, EvmGas, advanced_precompile, modexp,
    precompile::{EvmPrecompileDescriptor, EvmPrecompileGasPolicy, PAIRING_ITEM_BYTES, WORD_BYTES},
};

pub(crate) fn gas_cost(
    descriptor: EvmPrecompileDescriptor,
    input: &[u8],
) -> Result<Option<EvmGas>, EvmCoreError> {
    match descriptor.gas_policy {
        EvmPrecompileGasPolicy::Fixed(gas) => Ok(Some(gas)),
        EvmPrecompileGasPolicy::Words { base, per_word } => {
            Ok(Some(word_gas(base, per_word, input.len())?))
        }
        EvmPrecompileGasPolicy::Bn254Pairing => {
            Ok(Some(bn254_pairing_gas(descriptor.fork, input.len())?))
        }
        EvmPrecompileGasPolicy::Modexp => {
            Ok(Some(modexp::modexp_gas_cost(descriptor.fork, input)?))
        }
        EvmPrecompileGasPolicy::Blake2FRounds => Ok(Some(blake2f_round_gas(input)?)),
        EvmPrecompileGasPolicy::Bls12G1Msm
        | EvmPrecompileGasPolicy::Bls12G2Msm
        | EvmPrecompileGasPolicy::Bls12Pairing => Ok(Some(advanced_precompile::dynamic_gas(
            descriptor.gas_policy,
            input.len(),
        )?)),
        EvmPrecompileGasPolicy::DeferredDynamic => Ok(None),
    }
}

fn word_gas(base: EvmGas, per_word: EvmGas, len: usize) -> Result<EvmGas, EvmCoreError> {
    let words = words_for_len(len)?;
    let variable = words
        .checked_mul(per_word.get())
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    base.checked_add(EvmGas::new(variable))
        .map_err(|_| EvmCoreError::PrecompileGasOverflow)
}

fn bn254_pairing_gas(fork: EvmFork, len: usize) -> Result<EvmGas, EvmCoreError> {
    let pairs = len
        .checked_div(PAIRING_ITEM_BYTES)
        .and_then(|count| u64::try_from(count).ok())
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let (base, per_pair) = if fork.get() >= EvmFork::ISTANBUL.get() {
        (45_000u64, 34_000u64)
    } else {
        (100_000u64, 80_000u64)
    };
    let variable = pairs
        .checked_mul(per_pair)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let total = base
        .checked_add(variable)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    Ok(EvmGas::new(total))
}

fn blake2f_round_gas(input: &[u8]) -> Result<EvmGas, EvmCoreError> {
    let rounds = input
        .get(..4)
        .and_then(|prefix| <[u8; 4]>::try_from(prefix).ok())
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    Ok(EvmGas::new(u64::from(u32::from_be_bytes(rounds))))
}

fn words_for_len(len: usize) -> Result<u64, EvmCoreError> {
    if len == 0 {
        return Ok(0);
    }
    let rounded = len
        .checked_add(WORD_BYTES.saturating_sub(1))
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let words = rounded
        .checked_div(WORD_BYTES)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    u64::try_from(words).map_err(|_| EvmCoreError::PrecompileGasOverflow)
}
