extern crate std;

use std::vec::Vec;

use crate::{
    EVM_MAX_GAS_LIMIT, EvmCoreError, EvmFork, EvmGas, EvmGasMeter, EvmModExpWorkspace, EvmModexp,
    EvmPrecompileKind, EvmPrecompilePlan, EvmPrecompileRegistry, modexp::execute_modexp,
    modexp_workspace_limbs, parse_modexp_input,
};

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn modexp_parses_right_padded_header_and_payload() -> Result<(), EvmCoreError> {
    let parsed = parse_modexp_input(&[])?;
    assert!(parsed.base_len().is_zero());
    assert!(parsed.exponent_len().is_zero());
    assert!(parsed.modulus_len().is_zero());

    let mut input = modexp_input(1, 1, 2);
    input.push(5);
    input.push(2);
    input.push(7);

    let mut output = [0u8; 2];
    assert_eq!(execute_direct(&input, &mut output)?, 2);
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
fn modexp_zero_modulus_and_zero_modulus_len_follow_eip198() -> Result<(), EvmCoreError> {
    let mut empty_modulus = modexp_input(1, 1, 0);
    empty_modulus.extend_from_slice(&[9, 3]);
    let mut empty_output = [9u8; 1];
    assert_eq!(execute_direct(&empty_modulus, &mut empty_output)?, 0);
    assert_eq!(empty_output, [9]);

    let mut zero_modulus = modexp_input(1, 1, 2);
    zero_modulus.extend_from_slice(&[9, 3, 0, 0]);
    let mut output = [9u8; 2];
    assert_eq!(execute_direct(&zero_modulus, &mut output)?, 2);
    assert_eq!(output, [0, 0]);
    Ok(())
}

#[test]
fn modexp_rejects_short_output_without_modifying_it() -> Result<(), EvmCoreError> {
    let mut small_output = [0u8; 1];
    let mut needs_two = modexp_input(1, 1, 2);
    needs_two.extend_from_slice(&[2, 3, 5, 0]);
    let required = modexp_workspace_limbs(&needs_two)?;
    let mut storage = std::vec![0_u32; required];
    let mut workspace = EvmModExpWorkspace::new(&mut storage);
    assert_eq!(
        execute_modexp(&needs_two, &mut small_output, &mut workspace),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(small_output, [0]);
    Ok(())
}

#[test]
fn modexp_executes_an_eighty_byte_independent_oracle_vector() -> Result<(), EvmCoreError> {
    let base = hex_bytes(
        "0b30557a9fc4e90e33587da2c7ec11365b80a5caef14395e83a8cdf2173c6186\
         abd0f51a3f6489aed3f81d42678cb1d6fb20456a8fb4d9fe23486d92b7dc0126\
         4b7095badf04294e7398bde2072c5176",
    );
    let modulus = hex_bytes(
        "871a2d405366798c9fb2c5d8ebfe1124374a5d708396a9bccfe2f5081b2e4154\
         677a8da0b3c6d9ecff1225384b5e718497aabdd0e3f6091c2f4255687b8ea1b4\
         c7daed001326394c5f728598abbed1e4",
    );
    let expected = hex_bytes(
        "4c1fe90213de06c1b685ce2133b948ffa00645e212648f578c946e63bf5d8514\
         b9ceb453d91a2dc63f637352ea2d9c9cfe953d78cd02f0c87280cfd00836ce0f\
         474451904dc4cedeeb9bc66b082fbdc0",
    );
    let mut input = modexp_input(base.len(), 3, modulus.len());
    input.extend_from_slice(&base);
    input.extend_from_slice(&[1, 0, 1]);
    input.extend_from_slice(&modulus);
    let mut output = [0_u8; 80];

    assert_eq!(execute_direct(&input, &mut output)?, 80);
    assert_eq!(output.as_slice(), expected.as_slice());
    Ok(())
}

#[test]
fn modexp_matches_u128_oracle_across_limb_boundaries() -> Result<(), EvmCoreError> {
    let mut state = 0x9e37_79b9_7f4a_7c15_d1b5_4a32_d192_ed03_u128;
    for round in 0..64_u32 {
        state = state
            .wrapping_mul(0xda94_2042_e4dd_58b5_9e37_79b9_7f4a_7c15_u128)
            .wrapping_add(u128::from(round).wrapping_add(1));
        let modulus = state | (1_u128 << 127) | 1;
        let base = state.rotate_left(round % u128::BITS);
        let exponent = u128::from((round % 31).saturating_add(1));
        let expected = pow_mod_u128(base, exponent, modulus);
        let mut input = modexp_input(16, 16, 16);
        input.extend_from_slice(&base.to_be_bytes());
        input.extend_from_slice(&exponent.to_be_bytes());
        input.extend_from_slice(&modulus.to_be_bytes());
        let mut output = [0_u8; 16];

        assert_eq!(execute_direct(&input, &mut output)?, 16);
        assert_eq!(u128::from_be_bytes(output), expected);
    }
    Ok(())
}

#[test]
fn modexp_workspace_failure_is_pre_authorization_and_output_atomic() -> Result<(), EvmCoreError> {
    let mut input = modexp_input(9, 1, 9);
    input.extend_from_slice(&[0xa5_u8; 9]);
    input.push(3);
    input.extend_from_slice(&[0xff_u8; 9]);
    let descriptor = registry(EvmFork::BERLIN)?.descriptor(EvmPrecompileKind::Modexp)?;
    let quote = descriptor.quote::<EvmModexp>(&input)?;
    let required = quote.modexp_workspace_limbs()?;
    let mut storage = std::vec![0_u32; required.saturating_sub(1)];
    let mut workspace = EvmModExpWorkspace::new(&mut storage);
    let mut output = [0xa5_u8; 9];
    let mut gas = EvmGasMeter::try_new(EvmGas::new(1_000))?;

    assert_eq!(
        quote.authorize_and_execute_modexp(&mut gas, &mut output, &mut workspace),
        Err(EvmCoreError::PrecompileWorkspaceTooSmall)
    );
    assert_eq!(gas.used(), EvmGas::new(0));
    assert_eq!(output, [0xa5_u8; 9]);
    Ok(())
}

#[test]
fn modexp_preserves_wide_lengths_until_gas_rejects_the_call() -> Result<(), EvmCoreError> {
    let mut input = [0_u8; 96];
    input[0] = 1;
    let parsed = parse_modexp_input(&input)?;
    assert_eq!(parsed.base_len().to_be_bytes()[0], 1);
    assert_eq!(
        parsed.base_len().try_to_usize(),
        Err(EvmCoreError::PrecompileInputTooLarge)
    );

    let descriptor = registry(EvmFork::BERLIN)?.descriptor(EvmPrecompileKind::Modexp)?;
    let quote = descriptor.quote::<EvmModexp>(&input)?;
    assert_eq!(quote.gas_cost(), EvmGas::new(EVM_MAX_GAS_LIMIT + 1));
    let mut gas = EvmGasMeter::try_new(EvmGas::new(EVM_MAX_GAS_LIMIT))?;
    let mut output = [];
    let mut storage = [];
    let mut workspace = EvmModExpWorkspace::new(&mut storage);
    assert_eq!(
        quote.authorize_and_execute_modexp(&mut gas, &mut output, &mut workspace),
        Err(EvmCoreError::OutOfGas)
    );
    Ok(())
}

#[test]
fn modexp_zero_output_accepts_unrepresentable_exponent_length() -> Result<(), EvmCoreError> {
    let mut input = [0_u8; 96];
    input[32] = 1;
    let descriptor = registry(EvmFork::BERLIN)?.descriptor(EvmPrecompileKind::Modexp)?;
    let quote = descriptor.quote::<EvmModexp>(&input)?;
    assert_eq!(quote.gas_cost(), EvmGas::new(200));
    let mut gas = EvmGasMeter::try_new(EvmGas::new(200))?;
    let mut output = [];
    let mut storage = [];
    let mut workspace = EvmModExpWorkspace::new(&mut storage);
    let outcome = quote.authorize_and_execute_modexp(&mut gas, &mut output, &mut workspace)?;
    assert_eq!(outcome.output_len(), 0);
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

fn execute_direct(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    let required = modexp_workspace_limbs(input)?;
    let mut storage = std::vec![0_u32; required];
    let mut workspace = EvmModExpWorkspace::new(&mut storage);
    execute_modexp(input, output, &mut workspace)
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

fn hex_bytes(hex: &str) -> Vec<u8> {
    let compact: Vec<u8> = hex
        .as_bytes()
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    assert_eq!(compact.len() % 2, 0);
    compact
        .chunks_exact(2)
        .map(|pair| {
            let high = pair.first().copied().map(hex_nibble).unwrap_or(0);
            let low = pair.get(1).copied().map(hex_nibble).unwrap_or(0);
            (high << 4) | low
        })
        .collect()
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

fn pow_mod_u128(mut base: u128, mut exponent: u128, modulus: u128) -> u128 {
    base = base.checked_rem(modulus).unwrap_or(0);
    let mut result = 1_u128.checked_rem(modulus).unwrap_or(0);
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = mul_mod_u128(result, base, modulus);
        }
        base = mul_mod_u128(base, base, modulus);
        exponent >>= 1;
    }
    result
}

fn mul_mod_u128(mut left: u128, mut right: u128, modulus: u128) -> u128 {
    let mut result = 0_u128;
    while right != 0 {
        if right & 1 != 0 {
            result = add_mod_u128(result, left, modulus);
        }
        left = add_mod_u128(left, left, modulus);
        right >>= 1;
    }
    result
}

fn add_mod_u128(left: u128, right: u128, modulus: u128) -> u128 {
    if left >= modulus.saturating_sub(right) {
        left.saturating_sub(modulus.saturating_sub(right))
    } else {
        left.saturating_add(right)
    }
}
