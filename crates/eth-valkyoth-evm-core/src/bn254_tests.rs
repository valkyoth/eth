use crate::{
    EVM_BN254_POINT_BYTES, EvmCoreError, EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind,
    EvmPrecompilePlan, EvmPrecompileRegistry,
    bn254::{execute_bn254_add, execute_bn254_mul},
};

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn bn254_add_matches_generator_doubling_vector() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Add)?;
    let input = two_generator_points();
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    let mut output = [0u8; EVM_BN254_POINT_BYTES];
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(500))?;

    assert_eq!(plan.gas_cost(), Some(EvmGas::new(500)));
    assert_eq!(
        plan.execute_bn254_add(&mut gas_meter, &input, &mut output)?,
        64
    );
    assert_eq!(gas_meter.used(), EvmGas::new(500));
    assert_eq!(
        output,
        point(
            "030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3",
            "15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4",
        )
    );
    Ok(())
}

#[test]
fn bn254_mul_matches_generator_scalar_vector() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Mul)?;
    let input = generator_mul_input(3);
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    let mut output = [0u8; EVM_BN254_POINT_BYTES];
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(6_000))?;

    assert_eq!(plan.gas_cost(), Some(EvmGas::new(6_000)));
    assert_eq!(
        plan.execute_bn254_mul(&mut gas_meter, &input, &mut output)?,
        64
    );
    assert_eq!(gas_meter.used(), EvmGas::new(6_000));
    assert_eq!(
        output,
        point(
            "0769bf9ac56bea3ff40232bcb1b6bd159315d84715b8e679f2d355961915abf0",
            "2ab799bee0489429554fdb7c8d086475319e63b40b9c5b57cdf1ff3dd9fe2261",
        )
    );
    Ok(())
}

#[test]
fn bn254_empty_inputs_succeed_as_infinity() -> Result<(), EvmCoreError> {
    let mut output = [9u8; EVM_BN254_POINT_BYTES];
    assert_eq!(execute_bn254_add(&[], &mut output)?, 64);
    assert_eq!(output, [0u8; EVM_BN254_POINT_BYTES]);

    output.fill(9);
    assert_eq!(execute_bn254_mul(&[], &mut output)?, 64);
    assert_eq!(output, [0u8; EVM_BN254_POINT_BYTES]);
    Ok(())
}

#[test]
fn bn254_uses_eip196_padding_and_ignores_surplus_bytes() -> Result<(), EvmCoreError> {
    let generator = point(
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
    );
    let mut padded_add = [0u8; 64];
    padded_add.copy_from_slice(&generator);
    let mut output = [0u8; EVM_BN254_POINT_BYTES];
    assert_eq!(execute_bn254_add(&padded_add, &mut output)?, 64);
    assert_eq!(output, generator);

    let mut long_add = [0xffu8; 140];
    long_add[..128].copy_from_slice(&two_generator_points());
    assert_eq!(execute_bn254_add(&long_add, &mut output)?, 64);
    assert_eq!(
        output,
        point(
            "030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3",
            "15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4",
        )
    );

    assert_eq!(execute_bn254_mul(&generator, &mut output)?, 64);
    assert_eq!(output, [0u8; EVM_BN254_POINT_BYTES]);
    Ok(())
}

#[test]
fn bn254_rejects_invalid_field_and_point_inputs() {
    let mut invalid_field = [0u8; 128];
    invalid_field[..32].copy_from_slice(&field_modulus());
    let mut output = [7u8; EVM_BN254_POINT_BYTES];
    assert_eq!(
        execute_bn254_add(&invalid_field, &mut output),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
    assert_eq!(output, [7u8; EVM_BN254_POINT_BYTES]);

    let mut not_on_curve = [0u8; 128];
    not_on_curve[31] = 1;
    not_on_curve[63] = 1;
    assert_eq!(
        execute_bn254_add(&not_on_curve, &mut output),
        Err(EvmCoreError::PrecompilePointNotOnCurve)
    );
    assert_eq!(output, [7u8; EVM_BN254_POINT_BYTES]);
}

#[test]
fn bn254_mul_accepts_full_width_scalars() -> Result<(), EvmCoreError> {
    let mut field_scalar = [0u8; 96];
    let generator = point(
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
    );
    field_scalar[..64].copy_from_slice(&generator);
    field_scalar[64..].copy_from_slice(&field_modulus());

    let mut output = [0u8; EVM_BN254_POINT_BYTES];
    assert_eq!(execute_bn254_mul(&field_scalar, &mut output)?, 64);

    let mut max_scalar = field_scalar;
    if let Some(scalar) = max_scalar.get_mut(64..) {
        scalar.fill(0xff);
    }
    assert_eq!(execute_bn254_mul(&max_scalar, &mut output)?, 64);
    Ok(())
}

#[test]
fn bn254_add_and_mul_agree_for_generator_double() -> Result<(), EvmCoreError> {
    let mut add_output = [0u8; EVM_BN254_POINT_BYTES];
    let mut mul_output = [0u8; EVM_BN254_POINT_BYTES];
    assert_eq!(
        execute_bn254_add(&two_generator_points(), &mut add_output)?,
        64
    );
    assert_eq!(
        execute_bn254_mul(&generator_mul_input(2), &mut mul_output)?,
        64
    );
    assert_eq!(add_output, mul_output);

    let generator = point(
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
    );
    assert_eq!(
        execute_bn254_mul(&generator_mul_input(1), &mut mul_output)?,
        64
    );
    assert_eq!(mul_output, generator);
    Ok(())
}

#[test]
fn bn254_plan_checks_kind_and_input_length() -> Result<(), EvmCoreError> {
    let add = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Add)?;
    let plan = EvmPrecompilePlan::try_new(add, &two_generator_points())?;
    let mut output = [0u8; EVM_BN254_POINT_BYTES];
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(40_000))?;
    assert_eq!(
        plan.execute_bn254_add(&mut gas_meter, &[], &mut output),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );

    let mul = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Mul)?;
    let wrong_plan = EvmPrecompilePlan::try_new(mul, &generator_mul_input(2))?;
    assert_eq!(
        wrong_plan.execute_bn254_add(&mut gas_meter, &generator_mul_input(2), &mut output),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[test]
fn bn254_add_and_mul_plans_charge_every_execution() -> Result<(), EvmCoreError> {
    let input = two_generator_points();
    let add = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Add)?;
    let add_plan = EvmPrecompilePlan::try_new(add, &input)?;
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(300))?;
    let mut output = [0u8; EVM_BN254_POINT_BYTES];
    assert_eq!(
        add_plan.execute_bn254_add(&mut gas_meter, &input, &mut output)?,
        EVM_BN254_POINT_BYTES
    );
    assert_eq!(gas_meter.used(), EvmGas::new(150));
    assert_eq!(
        add_plan.execute_bn254_add(&mut gas_meter, &input, &mut output)?,
        EVM_BN254_POINT_BYTES
    );
    assert_eq!(gas_meter.used(), EvmGas::new(300));

    let input = generator_mul_input(2);
    let mul = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Mul)?;
    let mul_plan = EvmPrecompilePlan::try_new(mul, &input)?;
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(12_000))?;
    assert_eq!(
        mul_plan.execute_bn254_mul(&mut gas_meter, &input, &mut output)?,
        EVM_BN254_POINT_BYTES
    );
    assert_eq!(gas_meter.used(), EvmGas::new(6_000));
    assert_eq!(
        mul_plan.execute_bn254_mul(&mut gas_meter, &input, &mut output)?,
        EVM_BN254_POINT_BYTES
    );
    assert_eq!(gas_meter.used(), EvmGas::new(12_000));
    Ok(())
}

#[test]
fn bn254_gas_tracks_byzantium_and_istanbul_pricing() -> Result<(), EvmCoreError> {
    let add_byzantium = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Add)?;
    let add_istanbul = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Add)?;
    let mul_byzantium = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Mul)?;
    let mul_istanbul = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Mul)?;

    assert_eq!(
        EvmPrecompilePlan::try_new(add_byzantium, &[])?.gas_cost(),
        Some(EvmGas::new(500))
    );
    assert_eq!(
        EvmPrecompilePlan::try_new(add_istanbul, &[])?.gas_cost(),
        Some(EvmGas::new(150))
    );
    assert_eq!(
        EvmPrecompilePlan::try_new(mul_byzantium, &[])?.gas_cost(),
        Some(EvmGas::new(40_000))
    );
    assert_eq!(
        EvmPrecompilePlan::try_new(mul_istanbul, &[])?.gas_cost(),
        Some(EvmGas::new(6_000))
    );
    Ok(())
}

fn two_generator_points() -> [u8; 128] {
    let mut input = [0u8; 128];
    let generator = point(
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
    );
    input[..64].copy_from_slice(&generator);
    input[64..].copy_from_slice(&generator);
    input
}

fn generator_mul_input(scalar: u8) -> [u8; 96] {
    let mut input = [0u8; 96];
    let generator = point(
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
    );
    input[..64].copy_from_slice(&generator);
    input[95] = scalar;
    input
}

fn point(x: &str, y: &str) -> [u8; 64] {
    let mut output = [0u8; 64];
    output[..32].copy_from_slice(&hex32(x));
    output[32..].copy_from_slice(&hex32(y));
    output
}

fn field_modulus() -> [u8; 32] {
    hex32("30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47")
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
