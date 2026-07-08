use crate::{
    EVM_BN254_PAIRING_ITEM_BYTES, EVM_BN254_PAIRING_OUTPUT_BYTES, EvmCoreError, EvmFork, EvmGas,
    EvmGasMeter, EvmPrecompileImplementation, EvmPrecompileKind, EvmPrecompilePlan,
    EvmPrecompileRegistry,
    bn254_tower::{Fp12, exercise_tower_accumulation},
    execute_bn254_pairing, parse_bn254_pairing_input,
};

use crate::bn254_pairing::for_each_valid_pairing_tuple;

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn bn254_pairing_empty_input_returns_one() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &[])?;
    let mut output = [0u8; EVM_BN254_PAIRING_OUTPUT_BYTES];
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(100_000)));
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(100_000))?;
    assert_eq!(
        plan.execute_bn254_pairing(&mut gas_meter, &[], &mut output)?,
        32
    );
    assert_eq!(gas_meter.used(), EvmGas::new(100_000));
    assert_eq!(output, word_one());

    let istanbul = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(istanbul, &[])?.gas_cost(),
        Some(EvmGas::new(45_000))
    );
    Ok(())
}

#[test]
fn bn254_pairing_parses_official_generator_tuple_but_fails_closed() -> Result<(), EvmCoreError> {
    let input = generator_pairing_tuple();
    assert_eq!(parse_bn254_pairing_input(&input)?, 1);

    let descriptor = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        descriptor.implementation,
        EvmPrecompileImplementation::NativeBn254PairingFrame
    );
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(79_000)));

    let mut output = [7u8; EVM_BN254_PAIRING_OUTPUT_BYTES];
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(79_000))?;
    assert_eq!(
        plan.execute_bn254_pairing(&mut gas_meter, &input, &mut output),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    assert_eq!(gas_meter.used(), EvmGas::new(79_000));
    assert_eq!(output, [7u8; EVM_BN254_PAIRING_OUTPUT_BYTES]);
    Ok(())
}

#[test]
fn bn254_pairing_plan_charges_every_execution() -> Result<(), EvmCoreError> {
    let pairing = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    let plan = EvmPrecompilePlan::try_new(pairing, &[])?;
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(90_000))?;
    let mut output = [0u8; EVM_BN254_PAIRING_OUTPUT_BYTES];
    assert_eq!(
        plan.execute_bn254_pairing(&mut gas_meter, &[], &mut output)?,
        EVM_BN254_PAIRING_OUTPUT_BYTES
    );
    assert_eq!(gas_meter.used(), EvmGas::new(45_000));
    assert_eq!(
        plan.execute_bn254_pairing(&mut gas_meter, &[], &mut output)?,
        EVM_BN254_PAIRING_OUTPUT_BYTES
    );
    assert_eq!(gas_meter.used(), EvmGas::new(90_000));
    Ok(())
}

#[test]
fn bn254_pairing_streams_validated_tuples_once() -> Result<(), EvmCoreError> {
    let input = generator_pairing_tuple();
    let mut seen = 0usize;
    let pairs = for_each_valid_pairing_tuple(&input, |tuple| {
        seen = seen.saturating_add(1);
        assert!(!tuple.g1.infinity);
        assert!(!tuple.g2.infinity);
    })?;
    assert_eq!(pairs, 1);
    assert_eq!(seen, 1);
    Ok(())
}

#[test]
fn bn254_pairing_stream_stops_at_first_invalid_tuple() {
    let valid = generator_pairing_tuple();
    let mut input = [0u8; EVM_BN254_PAIRING_ITEM_BYTES * 2];
    if let Some(first) = input.get_mut(..EVM_BN254_PAIRING_ITEM_BYTES) {
        first.copy_from_slice(&valid);
    }
    if let Some(second) = input.get_mut(EVM_BN254_PAIRING_ITEM_BYTES..) {
        second.copy_from_slice(&valid);
    }
    write_word(
        &mut input,
        EVM_BN254_PAIRING_ITEM_BYTES.saturating_add(64),
        field_modulus(),
    );

    let mut seen = 0usize;
    assert_eq!(
        for_each_valid_pairing_tuple(&input, |_| {
            seen = seen.saturating_add(1);
        }),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
    assert_eq!(seen, 1);
}

#[test]
fn bn254_pairing_tower_accumulation_uses_validated_tuple_stream() -> Result<(), EvmCoreError> {
    let input = generator_pairing_tuple();
    let (pairs, acc) = exercise_tower_accumulation(&input)?;
    assert_eq!(pairs, 1);
    assert_ne!(acc, Fp12::ONE);
    Ok(())
}

#[test]
fn bn254_pairing_rejects_bad_lengths_and_short_output() -> Result<(), EvmCoreError> {
    assert_eq!(
        parse_bn254_pairing_input(&[0u8; EVM_BN254_PAIRING_ITEM_BYTES - 1]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    let mut short = [9u8; EVM_BN254_PAIRING_OUTPUT_BYTES - 1];
    assert_eq!(
        execute_bn254_pairing(&[], &mut short),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(short, [9u8; EVM_BN254_PAIRING_OUTPUT_BYTES - 1]);
    Ok(())
}

#[test]
fn bn254_pairing_rejects_invalid_g2_field_and_curve_points() {
    let mut invalid_field = generator_pairing_tuple();
    write_word(&mut invalid_field, 64, field_modulus());
    assert_eq!(
        parse_bn254_pairing_input(&invalid_field),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );

    let mut off_curve = [0u8; EVM_BN254_PAIRING_ITEM_BYTES];
    if let Some(last) = off_curve.last_mut() {
        *last = 1;
    }
    assert_eq!(
        parse_bn254_pairing_input(&off_curve),
        Err(EvmCoreError::PrecompilePointNotOnCurve)
    );
}

#[test]
fn bn254_pairing_rejects_valid_twist_point_outside_subgroup() {
    let mut input = generator_pairing_tuple();
    write_word(
        &mut input,
        64,
        hex32("1d6f09d463630f0967551d6f9d18ae3625a8d2e9cda114ec79ad899fc5f89222"),
    );
    write_word(
        &mut input,
        96,
        hex32("08b0e0b7458f5f522a3d0e9e2723eb95f807d3a90d5ba5ce58e1c906ef7ffaef"),
    );
    write_word(
        &mut input,
        128,
        hex32("274d387b58982993ed2c1853d39bf822e1d8c7cb029f6ce2fcd46b2d14cb1dac"),
    );
    write_word(
        &mut input,
        160,
        hex32("0aedf27dfce57ba51747d526c1e92028be56a45cedbf8f595b2269e5393474fa"),
    );
    assert_eq!(
        parse_bn254_pairing_input(&input),
        Err(EvmCoreError::PrecompilePointNotInSubgroup)
    );
    assert_eq!(
        EvmCoreError::PrecompilePointNotInSubgroup.code(),
        "precompile_point_not_in_subgroup"
    );
}

fn generator_pairing_tuple() -> [u8; EVM_BN254_PAIRING_ITEM_BYTES] {
    let mut output = [0u8; EVM_BN254_PAIRING_ITEM_BYTES];
    write_word(
        &mut output,
        0,
        hex32("0000000000000000000000000000000000000000000000000000000000000001"),
    );
    write_word(
        &mut output,
        32,
        hex32("0000000000000000000000000000000000000000000000000000000000000002"),
    );
    write_word(
        &mut output,
        64,
        hex32("198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2"),
    );
    write_word(
        &mut output,
        96,
        hex32("1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed"),
    );
    write_word(
        &mut output,
        128,
        hex32("090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b"),
    );
    write_word(
        &mut output,
        160,
        hex32("12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa"),
    );
    output
}

fn word_one() -> [u8; 32] {
    let mut output = [0u8; 32];
    if let Some(last) = output.last_mut() {
        *last = 1;
    }
    output
}

fn field_modulus() -> [u8; 32] {
    hex32("30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47")
}

fn write_word<const N: usize>(target: &mut [u8; N], offset: usize, word: [u8; 32]) {
    if let Some(output) = target.get_mut(offset..offset.saturating_add(32)) {
        output.copy_from_slice(&word);
    }
}

fn hex32(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);
    let mut output = [0u8; 32];
    for (target, pair) in output.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        let high = pair.first().copied().map(hex_nibble).unwrap_or(0);
        let low = pair.get(1).copied().map(hex_nibble).unwrap_or(0);
        *target = (high << 4) | low;
    }
    output
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'a' | b'A' => 10,
        b'b' | b'B' => 11,
        b'c' | b'C' => 12,
        b'd' | b'D' => 13,
        b'e' | b'E' => 14,
        b'f' | b'F' => 15,
        _ => 0,
    }
}
