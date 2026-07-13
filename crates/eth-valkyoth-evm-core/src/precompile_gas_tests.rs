use crate::{
    EvmCoreError, EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind, EvmPrecompilePlan,
    EvmPrecompileRegistry,
};

#[test]
fn identity_plan_charges_each_call_before_output_mutation() -> Result<(), EvmCoreError> {
    let registry = EvmPrecompileRegistry::try_new(EvmFork::FRONTIER)?;
    let descriptor = registry.descriptor(EvmPrecompileKind::Identity)?;
    let input = [1u8; 33];
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    let mut output = [7u8; 33];
    let mut insufficient = EvmGasMeter::try_new(EvmGas::new(20))?;

    assert_eq!(
        plan.execute_identity(&mut insufficient, &input, &mut output),
        Err(EvmCoreError::OutOfGas)
    );
    assert_eq!(insufficient.used(), EvmGas::new(0));
    assert_eq!(output, [7u8; 33]);

    let mut exact = EvmGasMeter::try_new(EvmGas::new(42))?;
    assert_eq!(plan.execute_identity(&mut exact, &input, &mut output)?, 33);
    assert_eq!(exact.used(), EvmGas::new(21));
    output.fill(7);
    assert_eq!(plan.execute_identity(&mut exact, &input, &mut output)?, 33);
    assert_eq!(exact.used(), EvmGas::new(42));
    Ok(())
}

#[test]
fn hash_plans_charge_before_output_mutation() -> Result<(), EvmCoreError> {
    let registry = EvmPrecompileRegistry::try_new(EvmFork::FRONTIER)?;
    let input = b"abc";
    let mut output = [7u8; 32];

    let sha = registry.descriptor(EvmPrecompileKind::Sha256)?;
    let sha_plan = EvmPrecompilePlan::try_new(sha, input)?;
    let mut sha_gas = EvmGasMeter::try_new(EvmGas::new(71))?;
    assert_eq!(
        sha_plan.execute_sha256(&mut sha_gas, input, &mut output),
        Err(EvmCoreError::OutOfGas)
    );
    assert_eq!(sha_gas.used(), EvmGas::new(0));
    assert_eq!(output, [7u8; 32]);

    let ripemd = registry.descriptor(EvmPrecompileKind::Ripemd160)?;
    let ripemd_plan = EvmPrecompilePlan::try_new(ripemd, input)?;
    let mut ripemd_gas = EvmGasMeter::try_new(EvmGas::new(719))?;
    assert_eq!(
        ripemd_plan.execute_ripemd160(&mut ripemd_gas, input, &mut output),
        Err(EvmCoreError::OutOfGas)
    );
    assert_eq!(ripemd_gas.used(), EvmGas::new(0));
    assert_eq!(output, [7u8; 32]);
    Ok(())
}
