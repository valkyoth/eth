use super::{ADDITIONS, VALIDATIONS};
use crate::{
    EvmBls12G1Add, EvmBls12381Fp, EvmBls12381G1Affine, EvmCoreError, EvmFork, EvmGas, EvmGasMeter,
    EvmPrecompileDescriptor, EvmPrecompileGasPolicy, EvmPrecompileKind, EvmPrecompileRegistry,
    EvmPrecompileStatus,
};

fn descriptor() -> Result<EvmPrecompileDescriptor, EvmCoreError> {
    EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?.descriptor(EvmPrecompileKind::Bls12G1Add)
}

fn reset() {
    VALIDATIONS.with(|count| count.set(0));
    ADDITIONS.with(|count| count.set(0));
}

fn work() -> (usize, usize) {
    (
        VALIDATIONS.with(core::cell::Cell::get),
        ADDITIONS.with(core::cell::Cell::get),
    )
}

#[test]
fn admission_failures_do_not_validate_or_add() -> Result<(), EvmCoreError> {
    reset();
    let input = [0; 256];
    let mut output = [0xa5; 129];
    for len in [0, 1, 127, 128, 255, 257] {
        let bytes = [0; 257];
        assert_eq!(
            descriptor()?
                .quote::<EvmBls12G1Add>(
                    bytes
                        .get(..len)
                        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
                )
                .err(),
            Some(EvmCoreError::PrecompileInvalidInputLength)
        );
    }
    let identity =
        EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?.descriptor(EvmPrecompileKind::Identity)?;
    assert_eq!(
        identity.quote::<EvmBls12G1Add>(&input).err(),
        Some(EvmCoreError::PrecompileBackendUnavailable)
    );
    let mut forged = descriptor()?;
    forged.gas_policy = EvmPrecompileGasPolicy::Fixed(EvmGas::new(1));
    assert_eq!(
        forged.quote::<EvmBls12G1Add>(&input).err(),
        Some(EvmCoreError::PrecompileDescriptorMismatch)
    );
    for gas in [0, 374] {
        let mut meter = EvmGasMeter::try_new(EvmGas::new(gas.max(1)))?;
        if gas == 0 {
            meter.charge(EvmGas::new(1))?;
        }
        let before = meter.used();
        assert_eq!(
            descriptor()?
                .quote::<EvmBls12G1Add>(&input)?
                .authorize_and_execute_bls12_g1_add(&mut meter, &mut output),
            Err(EvmCoreError::OutOfGas)
        );
        assert_eq!(meter.used(), before);
    }
    let mut meter = EvmGasMeter::try_new(EvmGas::new(1000))?;
    assert_eq!(
        descriptor()?
            .quote::<EvmBls12G1Add>(&input)?
            .authorize_and_execute_bls12_g1_add(&mut meter, &mut output[..127]),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(meter.used(), EvmGas::new(0));
    assert_eq!(output, [0xa5; 129]);
    assert_eq!(work(), (0, 0));
    Ok(())
}

#[test]
fn malformed_points_fail_atomically_after_payment_without_addition() -> Result<(), EvmCoreError> {
    for (offset, value, error, validations) in [
        (0, 1, EvmCoreError::PrecompileFieldElementOutOfRange, 0),
        (128, 1, EvmCoreError::PrecompileFieldElementOutOfRange, 0),
        (127, 1, EvmCoreError::PrecompilePointNotOnCurve, 1),
        (255, 1, EvmCoreError::PrecompilePointNotOnCurve, 2),
    ] {
        reset();
        let mut input = [0; 256];
        *input
            .get_mut(offset)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)? = value;
        let mut output = [0xa5; 129];
        let quote = descriptor()?.quote::<EvmBls12G1Add>(&input)?;
        assert_eq!(work(), (0, 0));
        let mut meter = EvmGasMeter::try_new(EvmGas::new(1000))?;
        meter.charge(EvmGas::new(11))?;
        let result = quote.authorize_and_execute_bls12_g1_add(&mut meter, &mut output)?;
        assert_eq!(result.status(), EvmPrecompileStatus::CallFailure);
        assert_eq!(result.error(), Some(error));
        assert_eq!(result.gas_consumed(), EvmGas::new(989));
        assert_eq!(result.output_len(), 0);
        assert!(result.requires_rollback());
        assert_eq!(meter.used(), meter.limit());
        assert_eq!(output, [0xa5; 129]);
        assert_eq!(work(), (validations, 0));
    }
    Ok(())
}

#[test]
fn charged_non_subgroup_addition_preserves_suffix_and_unused_gas() -> Result<(), EvmCoreError> {
    let point = EvmBls12381G1Affine::try_from_coordinates(
        EvmBls12381Fp::from_u64(0),
        EvmBls12381Fp::from_u64(2),
    )?;
    let mut input = [0; 256];
    input[..128].copy_from_slice(&point.to_be_bytes());
    input[128..].copy_from_slice(&point.to_be_bytes());
    for gas in [375, 1000] {
        reset();
        let mut output = [0xa5; 129];
        let mut meter = EvmGasMeter::try_new(EvmGas::new(gas))?;
        let quote = descriptor()?.quote::<EvmBls12G1Add>(&input)?;
        assert_eq!(quote.gas_cost(), EvmGas::new(375));
        assert_eq!(quote.output_len(), 128);
        let result = quote.authorize_and_execute_bls12_g1_add(&mut meter, &mut output)?;
        assert_eq!(result.status(), EvmPrecompileStatus::Success);
        assert_eq!(result.gas_consumed(), EvmGas::new(375));
        assert_eq!(result.output_len(), 128);
        assert_eq!(result.error(), None);
        assert!(!result.requires_rollback());
        assert_eq!(&output[..128], &point.negate().to_be_bytes());
        assert_eq!(output[128], 0xa5);
        assert_eq!(meter.used(), EvmGas::new(375));
        assert_eq!(work(), (2, 1));
    }
    Ok(())
}

#[test]
fn fork_activation_and_other_bls_backends_remain_explicit() -> Result<(), EvmCoreError> {
    let mut address = [0; 20];
    address[19] = 0x0b;
    let address = crate::EvmAddress::from_bytes(address);
    assert_eq!(
        EvmPrecompileRegistry::try_new(EvmFork::CANCUN)?.lookup(address),
        Err(EvmCoreError::PrecompileNotAvailableInFork)
    );
    assert_eq!(
        EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?.lookup(address)?,
        Some(descriptor()?)
    );
    for kind in [
        EvmPrecompileKind::Bls12G1Msm,
        EvmPrecompileKind::Bls12G2Add,
        EvmPrecompileKind::Bls12PairingCheck,
        EvmPrecompileKind::Bls12MapFpToG1,
    ] {
        assert_eq!(
            EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?
                .descriptor(kind)?
                .implementation,
            crate::EvmPrecompileImplementation::RequiresCryptoBackend
        );
    }
    Ok(())
}
