use super::{G1_DISCOUNTS, G1_ITEM_BYTES, G1_MUL_GAS, G2_DISCOUNTS, G2_ITEM_BYTES, G2_MUL_GAS};
use crate::{EvmCoreError, EvmGas};

// Independently copied from the official EIP-2537 discount tables instead of
// deriving the oracle from the production constants:
// https://eips.ethereum.org/EIPS/eip-2537
const G1_ORACLE: &str = include_str!("testdata/eip2537_g1_discounts.txt");
const G2_ORACLE: &str = include_str!("testdata/eip2537_g2_discounts.txt");

type GasFunction = fn(usize) -> Result<EvmGas, EvmCoreError>;

#[test]
fn every_eip2537_msm_discount_and_capped_gas_value_matches_oracle() -> Result<(), EvmCoreError> {
    verify_schedule(
        G1_ORACLE,
        &G1_DISCOUNTS,
        G1_ITEM_BYTES,
        G1_MUL_GAS,
        super::g1_msm,
    )?;
    verify_schedule(
        G2_ORACLE,
        &G2_DISCOUNTS,
        G2_ITEM_BYTES,
        G2_MUL_GAS,
        super::g2_msm,
    )
}

fn verify_schedule(
    oracle: &str,
    discounts: &[u16; 128],
    item_bytes: usize,
    multiplication_gas: u64,
    gas_function: GasFunction,
) -> Result<(), EvmCoreError> {
    let mut entries = 0_usize;
    for (token, &actual_discount) in oracle.split_ascii_whitespace().zip(discounts) {
        let expected_discount = token
            .parse::<u16>()
            .map_err(|_| EvmCoreError::PrecompileGasOverflow)?;
        assert_eq!(actual_discount, expected_discount);

        entries = entries
            .checked_add(1)
            .ok_or(EvmCoreError::PrecompileGasOverflow)?;
        let input_len = entries
            .checked_mul(item_bytes)
            .ok_or(EvmCoreError::PrecompileGasOverflow)?;
        let expected_gas = u64::try_from(entries)
            .map_err(|_| EvmCoreError::PrecompileGasOverflow)?
            .checked_mul(multiplication_gas)
            .and_then(|gas| gas.checked_mul(u64::from(expected_discount)))
            .and_then(|gas| gas.checked_div(1_000))
            .ok_or(EvmCoreError::PrecompileGasOverflow)?;
        assert_eq!(gas_function(input_len)?, EvmGas::new(expected_gas));
    }

    assert_eq!(entries, discounts.len());
    assert_eq!(oracle.split_ascii_whitespace().count(), discounts.len());

    let capped_items = entries
        .checked_add(1)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let capped_input_len = capped_items
        .checked_mul(item_bytes)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let capped_discount = u64::from(
        discounts
            .last()
            .copied()
            .ok_or(EvmCoreError::PrecompileGasOverflow)?,
    );
    let capped_gas = u64::try_from(capped_items)
        .map_err(|_| EvmCoreError::PrecompileGasOverflow)?
        .checked_mul(multiplication_gas)
        .and_then(|gas| gas.checked_mul(capped_discount))
        .and_then(|gas| gas.checked_div(1_000))
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    assert_eq!(gas_function(capped_input_len)?, EvmGas::new(capped_gas));
    Ok(())
}
