//! Independent BigUint differential coverage for the first-party ModExp engine.

use eth_valkyoth_evm_core::{
    EvmCoreError, EvmFork, EvmGasMeter, EvmModExpWorkspace, EvmModexp, EvmPrecompileKind,
    EvmPrecompileRegistry, EvmPrecompileStatus,
};
use num_bigint::BigUint;

const HEADER_BYTES: usize = 96;

#[test]
fn modexp_matches_independent_biguint_above_and_below_legacy_limit() -> Result<(), EvmCoreError> {
    for length in [1_usize, 32, 33, 64, 65, 80, 127, 256] {
        let base = patterned_bytes(length, 37, 11);
        let mut modulus = patterned_bytes(length, 19, 7);
        if let Some(first) = modulus.first_mut() {
            *first |= 0x80;
        }
        if let Some(last) = modulus.last_mut() {
            *last |= 1;
        }
        let exponent = [1_u8, 0, 1];
        let expected = BigUint::from_bytes_be(&base)
            .modpow(
                &BigUint::from_bytes_be(&exponent),
                &BigUint::from_bytes_be(&modulus),
            )
            .to_bytes_be();
        let mut expected_padded = vec![0_u8; length];
        let start = length
            .checked_sub(expected.len())
            .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
        expected_padded
            .get_mut(start..)
            .ok_or(EvmCoreError::PrecompileOutputTooSmall)?
            .copy_from_slice(&expected);

        let mut input = modexp_input(length, exponent.len(), length)?;
        input.extend_from_slice(&base);
        input.extend_from_slice(&exponent);
        input.extend_from_slice(&modulus);
        let registry = EvmPrecompileRegistry::try_new(EvmFork::BERLIN)?;
        let descriptor = registry.descriptor(EvmPrecompileKind::Modexp)?;
        let quote = descriptor.quote::<EvmModexp>(&input)?;
        let mut gas = EvmGasMeter::try_new(quote.gas_cost())?;
        let mut output = vec![0_u8; quote.output_len()];
        let mut storage = vec![0_u32; quote.modexp_workspace_limbs()?];
        let mut workspace = EvmModExpWorkspace::new(&mut storage);

        let outcome = quote.authorize_and_execute_modexp(&mut gas, &mut output, &mut workspace)?;
        assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
        assert_eq!(output, expected_padded, "operand length {length}");
    }
    Ok(())
}

fn patterned_bytes(length: usize, multiplier: usize, addend: usize) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let value = index.wrapping_mul(multiplier).wrapping_add(addend) & 0xff;
            u8::try_from(value).unwrap_or(0)
        })
        .collect()
}

fn modexp_input(
    base_len: usize,
    exponent_len: usize,
    modulus_len: usize,
) -> Result<Vec<u8>, EvmCoreError> {
    let mut input = vec![0_u8; HEADER_BYTES];
    write_len(&mut input, 0, base_len)?;
    write_len(&mut input, 32, exponent_len)?;
    write_len(&mut input, 64, modulus_len)?;
    Ok(input)
}

fn write_len(input: &mut [u8], offset: usize, value: usize) -> Result<(), EvmCoreError> {
    let bytes = value.to_be_bytes();
    let target = offset
        .checked_add(32)
        .and_then(|end| end.checked_sub(bytes.len()))
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let end = target
        .checked_add(bytes.len())
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    input
        .get_mut(target..end)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?
        .copy_from_slice(&bytes);
    Ok(())
}
