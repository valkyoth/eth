extern crate std;

use std::vec::Vec;

use crate::{
    EVM_MODEXP_MAX_OPERAND_BYTES, EvmCoreError, EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind,
    EvmPrecompilePlan, EvmPrecompileRegistry, modexp::execute_modexp, parse_modexp_input,
};

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn modexp_parses_right_padded_header_and_payload() -> Result<(), EvmCoreError> {
    let parsed = parse_modexp_input(&[])?;
    assert_eq!(parsed.base_len(), 0);
    assert_eq!(parsed.exponent_len(), 0);
    assert_eq!(parsed.modulus_len(), 0);

    let mut input = modexp_input(1, 1, 2);
    input.push(5);
    input.push(2);
    input.push(7);

    let mut output = [0u8; 2];
    assert_eq!(execute_modexp(&input, &mut output)?, 2);
    assert_eq!(output, [0, 25]);
    Ok(())
}

#[test]
fn modexp_executes_eip198_fermat_vector() -> Result<(), EvmCoreError> {
    let mut input = modexp_input(1, 32, 32);
    input.push(3);
    input.extend_from_slice(&hex32(
        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
    ));
    input.extend_from_slice(&hex32(
        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
    ));

    let descriptor = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Modexp)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    let mut output = [0u8; 32];
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(13_056))?;

    assert_eq!(plan.gas_cost(), Some(EvmGas::new(13_056)));
    assert_eq!(
        plan.execute_modexp(&mut gas_meter, &input, &mut output)?,
        32
    );
    assert_eq!(gas_meter.used(), EvmGas::new(13_056));
    assert_eq!(output[31], 1);
    assert!(output[..31].iter().all(|byte| *byte == 0));
    Ok(())
}

#[test]
fn modexp_uses_berlin_eip2565_gas_floor_and_formula() -> Result<(), EvmCoreError> {
    let empty = registry(EvmFork::BERLIN)?.descriptor(EvmPrecompileKind::Modexp)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(empty, &[])?.gas_cost(),
        Some(EvmGas::new(200))
    );

    let mut input = modexp_input(1, 32, 32);
    input.push(3);
    input.extend_from_slice(&[0xff; 32]);
    input.extend_from_slice(&[0xff; 32]);
    let berlin = registry(EvmFork::BERLIN)?.descriptor(EvmPrecompileKind::Modexp)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(berlin, &input)?.gas_cost(),
        Some(EvmGas::new(1_360))
    );
    Ok(())
}

#[test]
fn modexp_gas_uses_declared_short_exponent_width() -> Result<(), EvmCoreError> {
    let mut exponent_three = modexp_input(1, 1, 32);
    exponent_three.push(3);
    exponent_three.push(3);
    exponent_three.extend_from_slice(&[0xff; 32]);

    let byzantium = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Modexp)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(byzantium, &exponent_three)?.gas_cost(),
        Some(EvmGas::new(51))
    );

    let berlin = registry(EvmFork::BERLIN)?.descriptor(EvmPrecompileKind::Modexp)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(berlin, &exponent_three)?.gas_cost(),
        Some(EvmGas::new(200))
    );

    let mut exponent_zero = modexp_input(1, 1, 32);
    exponent_zero.push(3);
    exponent_zero.push(0);
    exponent_zero.extend_from_slice(&[0xff; 32]);
    assert_eq!(
        EvmPrecompilePlan::try_new(byzantium, &exponent_zero)?.gas_cost(),
        Some(EvmGas::new(51))
    );
    Ok(())
}

#[test]
fn modexp_zero_modulus_and_zero_modulus_len_are_bounded() -> Result<(), EvmCoreError> {
    let mut empty_modulus = modexp_input(1, 1, 0);
    empty_modulus.extend_from_slice(&[9, 3]);
    let mut empty_output = [9u8; 1];
    assert_eq!(execute_modexp(&empty_modulus, &mut empty_output)?, 0);
    assert_eq!(empty_output, [9]);

    let mut zero_modulus = modexp_input(1, 1, 2);
    zero_modulus.extend_from_slice(&[9, 3, 0, 0]);
    let mut output = [9u8; 2];
    assert_eq!(execute_modexp(&zero_modulus, &mut output)?, 2);
    assert_eq!(output, [0, 0]);
    Ok(())
}

#[test]
fn modexp_rejects_oversized_operands_and_short_output() -> Result<(), EvmCoreError> {
    let input = modexp_input(EVM_MODEXP_MAX_OPERAND_BYTES + 1, 1, 1);
    assert_eq!(
        parse_modexp_input(&input),
        Err(EvmCoreError::PrecompileInputTooLarge)
    );

    let mut small_output = [0u8; 1];
    let mut needs_two = modexp_input(1, 1, 2);
    needs_two.extend_from_slice(&[2, 3, 5, 0]);
    assert_eq!(
        execute_modexp(&needs_two, &mut small_output),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(small_output, [0]);
    Ok(())
}

#[test]
fn modexp_plan_rejects_wrong_input_len_or_kind() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Modexp)?;
    let input = modexp_input(0, 0, 0);
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    let mut output = [0u8; 1];
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(1_000))?;
    assert_eq!(
        plan.execute_modexp(&mut gas_meter, &[], &mut output),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );

    let identity = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Identity)?;
    let wrong_plan = EvmPrecompilePlan::try_new(identity, &input)?;
    assert_eq!(
        wrong_plan.execute_modexp(&mut gas_meter, &input, &mut output),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[test]
fn modexp_plan_charges_every_execution() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::BERLIN)?.descriptor(EvmPrecompileKind::Modexp)?;
    let input = modexp_input(0, 0, 0);
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(400))?;
    let mut output = [0u8; 0];

    assert_eq!(plan.gas_cost(), Some(EvmGas::new(200)));
    assert_eq!(plan.execute_modexp(&mut gas_meter, &input, &mut output)?, 0);
    assert_eq!(gas_meter.used(), EvmGas::new(200));
    assert_eq!(plan.execute_modexp(&mut gas_meter, &input, &mut output)?, 0);
    assert_eq!(gas_meter.used(), EvmGas::new(400));
    Ok(())
}

#[test]
fn modexp_plan_rejects_same_length_input_with_changed_operand_cost() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Modexp)?;
    let mut planned_input = modexp_input(1, 32, 32);
    planned_input.extend_from_slice(&[0_u8; 65]);
    let plan = EvmPrecompilePlan::try_new(descriptor, &planned_input)?;
    let mut execution_input = planned_input.clone();
    if let Some(exponent) = execution_input.get_mut(97) {
        *exponent = u8::MAX;
    }
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(100_000))?;
    let mut output = [0xa5_u8; 32];

    assert_eq!(
        plan.execute_modexp(&mut gas_meter, &execution_input, &mut output),
        Err(EvmCoreError::PrecompilePlanInputMismatch)
    );
    assert_eq!(gas_meter.used(), EvmGas::new(0));
    assert!(output.iter().all(|byte| *byte == 0xa5));
    Ok(())
}

fn modexp_input(base_len: usize, exponent_len: usize, modulus_len: usize) -> Vec<u8> {
    let mut input = Vec::from([0u8; 96]);
    write_len(&mut input, 0, base_len);
    write_len(&mut input, 32, exponent_len);
    write_len(&mut input, 64, modulus_len);
    input
}

fn write_len(input: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    let Some(end) = offset.checked_add(32) else {
        return;
    };
    let Some(target) = end.checked_sub(bytes.len()) else {
        return;
    };
    let Some(range_end) = target.checked_add(bytes.len()) else {
        return;
    };
    if let Some(target) = input.get_mut(target..range_end) {
        target.copy_from_slice(&bytes);
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
