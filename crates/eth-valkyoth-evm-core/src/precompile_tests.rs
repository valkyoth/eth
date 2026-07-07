use crate::{
    EVM_PRECOMPILE_INPUT_LIMIT, EvmAddress, EvmCoreError, EvmFork, EvmGas,
    EvmPrecompileImplementation, EvmPrecompileKind, EvmPrecompilePlan, EvmPrecompileRegistry,
    execute_identity,
};

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn precompile_addresses_are_low_canonical_accounts() {
    assert_eq!(
        EvmPrecompileKind::Identity.address().to_bytes(),
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]
    );
    assert_eq!(
        EvmPrecompileKind::Bls12MapFp2ToG2.address().to_bytes(),
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17]
    );
}

#[test]
fn registry_is_fork_aware() -> Result<(), EvmCoreError> {
    assert_eq!(
        registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Bn254Add),
        Err(EvmCoreError::PrecompileNotAvailableInFork)
    );
    assert_eq!(
        registry(EvmFork::BYZANTIUM)?
            .descriptor(EvmPrecompileKind::Bn254Add)?
            .implementation,
        EvmPrecompileImplementation::RequiresCryptoBackend
    );
    assert_eq!(
        registry(EvmFork::ISTANBUL)?
            .descriptor(EvmPrecompileKind::Blake2F)?
            .address,
        EvmPrecompileKind::Blake2F.address()
    );
    assert_eq!(
        registry(EvmFork::CANCUN)?
            .descriptor(EvmPrecompileKind::KzgPointEvaluation)?
            .address,
        EvmPrecompileKind::KzgPointEvaluation.address()
    );
    Ok(())
}

#[test]
fn lookup_rejects_unknown_and_future_precompile_addresses() -> Result<(), EvmCoreError> {
    let unknown =
        EvmAddress::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 99]);
    let non_reserved =
        EvmAddress::from_bytes([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]);
    assert_eq!(registry(EvmFork::FRONTIER)?.lookup(unknown)?, None);
    assert_eq!(registry(EvmFork::FRONTIER)?.lookup(non_reserved)?, None);
    assert_eq!(
        registry(EvmFork::BERLIN)?.lookup(EvmPrecompileKind::KzgPointEvaluation.address()),
        Err(EvmCoreError::PrecompileNotAvailableInFork)
    );
    Ok(())
}

#[test]
fn lookup_returns_descriptors_for_known_addresses() -> Result<(), EvmCoreError> {
    let identity = registry(EvmFork::FRONTIER)?
        .lookup(EvmPrecompileKind::Identity.address())?
        .ok_or(EvmCoreError::PrecompileNotAvailableInFork)?;
    assert_eq!(identity.kind, EvmPrecompileKind::Identity);
    assert_eq!(identity.fork, EvmFork::FRONTIER);

    let kzg = registry(EvmFork::CANCUN)?
        .lookup(EvmPrecompileKind::KzgPointEvaluation.address())?
        .ok_or(EvmCoreError::PrecompileNotAvailableInFork)?;
    assert_eq!(kzg.kind, EvmPrecompileKind::KzgPointEvaluation);
    assert_eq!(kzg.fork, EvmFork::CANCUN);
    Ok(())
}

#[test]
fn identity_plan_computes_word_gas_and_executes() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Identity)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &[1u8; 33])?;
    let mut output = [0u8; 33];
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(21)));
    assert_eq!(plan.execute_identity(&[1u8; 33], &mut output)?, 33);
    assert_eq!(output, [1u8; 33]);
    Ok(())
}

#[test]
fn identity_output_buffer_is_checked_before_copy() -> Result<(), EvmCoreError> {
    let mut output = [0u8; 2];
    assert_eq!(
        execute_identity(&[1, 2, 3], &mut output),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(output, [0, 0]);
    Ok(())
}

#[test]
fn input_policy_rejects_unbounded_or_bad_lengths() -> Result<(), EvmCoreError> {
    let blake = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(blake, &[0u8; 212]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    let pairing = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(pairing, &[0u8; 191]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    static OVERSIZED: [u8; EVM_PRECOMPILE_INPUT_LIMIT + 1] = [0u8; EVM_PRECOMPILE_INPUT_LIMIT + 1];
    let identity = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Identity)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(identity, &OVERSIZED),
        Err(EvmCoreError::PrecompileInputTooLarge)
    );
    Ok(())
}

#[test]
fn crypto_precompile_plans_do_not_execute_without_backend() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Sha256)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &[0u8; 1])?;
    let mut output = [0u8; 32];
    assert_eq!(
        plan.execute_identity(&[0u8; 1], &mut output),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[test]
fn known_precompile_gas_policies_are_bounded() -> Result<(), EvmCoreError> {
    let sha = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Sha256)?;
    assert_eq!(sha.output_len, Some(32));
    assert_eq!(
        EvmPrecompilePlan::try_new(sha, &[0u8; 33])?.gas_cost(),
        Some(EvmGas::new(84))
    );

    let byzantium_pairing =
        registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(byzantium_pairing, &[0u8; 192])?.gas_cost(),
        Some(EvmGas::new(180_000))
    );

    let bn_add = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Add)?;
    assert_eq!(bn_add.output_len, Some(64));

    let pairing = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(pairing, &[0u8; 384])?.gas_cost(),
        Some(EvmGas::new(113_000))
    );

    let mut blake_input = [0u8; 213];
    blake_input[3] = 12;
    let blake = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    assert_eq!(blake.output_len, Some(64));
    assert_eq!(
        EvmPrecompilePlan::try_new(blake, &blake_input)?.gas_cost(),
        Some(EvmGas::new(12))
    );
    Ok(())
}

#[test]
fn deferred_dynamic_precompile_gas_is_not_zero_cost() -> Result<(), EvmCoreError> {
    let modexp = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Modexp)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(modexp, &[0u8; 96])?.gas_cost(),
        None
    );

    let bls = registry(EvmFork::PRAGUE)?.descriptor(EvmPrecompileKind::Bls12PairingCheck)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(bls, &[0u8; 384])?.gas_cost(),
        None
    );
    Ok(())
}
