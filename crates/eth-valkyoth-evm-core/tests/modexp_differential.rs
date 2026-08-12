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
        let modulus = patterned_bytes(length, 19, 7);
        let exponent = [1_u8, 0, 1];
        assert_complete_case(&base, &exponent, &modulus)?;
    }
    Ok(())
}

#[test]
fn modexp_matches_biguint_for_adversarial_operand_shapes() -> Result<(), EvmCoreError> {
    for leading_zero_bytes in [4_usize, 8, 12] {
        let mut modulus = vec![0_u8; leading_zero_bytes];
        modulus.push(5);
        assert_complete_case(&[2], &[3], &modulus)?;
    }

    let base = patterned_bytes(65, 37, 11);
    let mut even_modulus = patterned_bytes(80, 19, 7);
    if let Some(last) = even_modulus.last_mut() {
        *last &= 0xfe;
    }
    assert_complete_case(&base, &[0, 0, 0, 1], &even_modulus)?;
    let short_base = base
        .get(..33)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    assert_complete_case(short_base, &[0_u8; 32], &[0_u8; 65])?;
    let mut sparse_exponent = vec![0_u8; 32];
    if let Some(last) = sparse_exponent.last_mut() {
        *last = 1;
    }
    assert_complete_case(short_base, &sparse_exponent, &even_modulus)?;

    let mut truncated = modexp_input(2, 2, 4)?;
    truncated.extend_from_slice(&[1, 2, 0, 3, 5]);
    assert_frame(&truncated, 2, 2, 4)?;
    Ok(())
}

fn assert_complete_case(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Result<(), EvmCoreError> {
    let mut input = modexp_input(base.len(), exponent.len(), modulus.len())?;
    input.extend_from_slice(base);
    input.extend_from_slice(exponent);
    input.extend_from_slice(modulus);
    assert_frame(&input, base.len(), exponent.len(), modulus.len())
}

fn assert_frame(
    input: &[u8],
    base_len: usize,
    exponent_len: usize,
    modulus_len: usize,
) -> Result<(), EvmCoreError> {
    let base = padded_segment(input, HEADER_BYTES, base_len);
    let exponent_offset = HEADER_BYTES.saturating_add(base_len);
    let exponent = padded_segment(input, exponent_offset, exponent_len);
    let modulus_offset = exponent_offset.saturating_add(exponent_len);
    let modulus = padded_segment(input, modulus_offset, modulus_len);
    let modulus_value = BigUint::from_bytes_be(&modulus);
    let expected = if modulus_value == BigUint::from(0_u8) {
        Vec::new()
    } else {
        BigUint::from_bytes_be(&base)
            .modpow(&BigUint::from_bytes_be(&exponent), &modulus_value)
            .to_bytes_be()
    };
    let mut expected_padded = vec![0_u8; modulus_len];
    let start = modulus_len
        .checked_sub(expected.len())
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    expected_padded
        .get_mut(start..)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?
        .copy_from_slice(&expected);

    let registry = EvmPrecompileRegistry::try_new(EvmFork::BERLIN)?;
    let descriptor = registry.descriptor(EvmPrecompileKind::Modexp)?;
    let quote = descriptor.quote::<EvmModexp>(input)?;
    let mut gas = EvmGasMeter::try_new(quote.gas_cost())?;
    let mut output = vec![0_u8; quote.output_len()];
    let mut storage = vec![0_u32; quote.modexp_workspace_limbs()?];
    let mut workspace = EvmModExpWorkspace::new(&mut storage);
    let outcome = quote.authorize_and_execute_modexp(&mut gas, &mut output, &mut workspace)?;
    assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
    assert_eq!(output, expected_padded);
    Ok(())
}

fn padded_segment(input: &[u8], offset: usize, len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| {
            offset
                .checked_add(index)
                .and_then(|position| input.get(position))
                .copied()
                .unwrap_or(0)
        })
        .collect()
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
