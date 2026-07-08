use crate::{EvmCoreError, EvmFork, EvmGas, modexp};

/// Computes ModExp gas for fuzz and differential test harnesses.
pub fn testing_modexp_gas_cost(fork: EvmFork, input: &[u8]) -> Result<EvmGas, EvmCoreError> {
    modexp::modexp_gas_cost(fork, input)
}
