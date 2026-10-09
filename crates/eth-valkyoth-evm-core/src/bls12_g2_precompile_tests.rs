use super::{ADDITIONS, VALIDATIONS};
use crate::{
    EvmBls12G2Add, EvmBls12381Fp2 as Fp2, EvmBls12381G2Affine as G2, EvmCoreError, EvmFork, EvmGas,
    EvmGasMeter, EvmPrecompileDescriptor, EvmPrecompileGasPolicy, EvmPrecompileKind,
    EvmPrecompileRegistry, EvmPrecompileStatus,
    bls12_fp2::{INVERSIONS, ROOTS},
};

fn descriptor() -> Result<EvmPrecompileDescriptor, EvmCoreError> {
    EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?.descriptor(EvmPrecompileKind::Bls12G2Add)
}

fn reset() {
    for counter in [&VALIDATIONS, &ADDITIONS, &INVERSIONS, &ROOTS] {
        counter.with(|n| n.set(0));
    }
}

fn work() -> [usize; 4] {
    [&VALIDATIONS, &ADDITIONS, &INVERSIONS, &ROOTS].map(|n| n.with(core::cell::Cell::get))
}

#[test]
fn failed_admission_never_validates_adds_inverts_or_takes_roots() -> Result<(), EvmCoreError> {
    reset();
    let bytes = [0; 513];
    for len in [0, 1, 255, 256, 511, 513] {
        assert_eq!(
            descriptor()?
                .quote::<EvmBls12G2Add>(
                    bytes
                        .get(..len)
                        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
                )
                .err(),
            Some(EvmCoreError::PrecompileInvalidInputLength)
        );
    }
    let input = [0; 512];
    let mut output = [0xa5; 257];
    let identity =
        EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?.descriptor(EvmPrecompileKind::Identity)?;
    assert_eq!(
        identity.quote::<EvmBls12G2Add>(&input).err(),
        Some(EvmCoreError::PrecompileBackendUnavailable)
    );
    let mut forged = descriptor()?;
    forged.gas_policy = EvmPrecompileGasPolicy::Fixed(EvmGas::new(1));
    assert_eq!(
        forged.quote::<EvmBls12G2Add>(&input).err(),
        Some(EvmCoreError::PrecompileDescriptorMismatch)
    );
    for gas in [0_u64, 599] {
        let mut meter = EvmGasMeter::try_new(EvmGas::new(gas.max(1)))?;
        if gas == 0 {
            meter.charge(EvmGas::new(1))?;
        }
        let before = meter.used();
        assert_eq!(
            descriptor()?
                .quote::<EvmBls12G2Add>(&input)?
                .authorize_and_execute_bls12_g2_add(&mut meter, &mut output),
            Err(EvmCoreError::OutOfGas)
        );
        assert_eq!(meter.used(), before);
    }
    let mut meter = EvmGasMeter::try_new(EvmGas::new(1000))?;
    assert_eq!(
        descriptor()?
            .quote::<EvmBls12G2Add>(&input)?
            .authorize_and_execute_bls12_g2_add(&mut meter, &mut output[..255]),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(meter.used(), EvmGas::new(0));
    assert_eq!(output, [0xa5; 257]);
    assert_eq!(work(), [0; 4]);
    Ok(())
}

#[test]
fn paid_malformed_points_burn_remaining_gas_without_output_or_expensive_work()
-> Result<(), EvmCoreError> {
    for (offset, error, validations) in [
        (0, EvmCoreError::PrecompileFieldElementOutOfRange, 0),
        (256, EvmCoreError::PrecompileFieldElementOutOfRange, 0),
        (255, EvmCoreError::PrecompilePointNotOnCurve, 1),
        (511, EvmCoreError::PrecompilePointNotOnCurve, 2),
    ] {
        reset();
        let mut input = [0; 512];
        *input
            .get_mut(offset)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)? = 1;
        let mut output = [0xa5; 257];
        let quote = descriptor()?.quote::<EvmBls12G2Add>(&input)?;
        assert_eq!(work(), [0; 4]);
        let mut meter = EvmGasMeter::try_new(EvmGas::new(1000))?;
        meter.charge(EvmGas::new(11))?;
        let result = quote.authorize_and_execute_bls12_g2_add(&mut meter, &mut output)?;
        assert_eq!(result.status(), EvmPrecompileStatus::CallFailure);
        assert_eq!(result.error(), Some(error));
        assert_eq!(result.gas_consumed(), EvmGas::new(989));
        assert_eq!(result.output_len(), 0);
        assert!(result.requires_rollback());
        assert_eq!(meter.used(), meter.limit());
        assert_eq!(output, [0xa5; 257]);
        assert_eq!(work(), [validations, 0, 0, 0]);
    }
    Ok(())
}

#[test]
fn successful_paid_addition_preserves_suffix_and_charges_exact_gas() -> Result<(), EvmCoreError> {
    let x = Fp2::from_u64(2);
    let four = crate::EvmBls12381Fp::from_u64(4);
    let y = x
        .square()
        .mul_mod(x)
        .add_mod(Fp2::from_coefficients(four, four))
        .sqrt()
        .ok_or(EvmCoreError::PrecompilePointNotOnCurve)?;
    let point = G2::try_from_coordinates(x, y)?;
    let expected = point.double().to_be_bytes();
    let mut input = [0; 512];
    input[..256].copy_from_slice(&point.to_be_bytes());
    input[256..].copy_from_slice(&point.to_be_bytes());
    for gas in [600, 1000] {
        reset();
        let mut output = [0xa5; 257];
        let mut meter = EvmGasMeter::try_new(EvmGas::new(gas))?;
        let quote = descriptor()?.quote::<EvmBls12G2Add>(&input)?;
        assert_eq!(quote.gas_cost(), EvmGas::new(600));
        assert_eq!(quote.output_len(), 256);
        assert_eq!(work(), [0; 4]);
        let result = quote.authorize_and_execute_bls12_g2_add(&mut meter, &mut output)?;
        assert_eq!(result.status(), EvmPrecompileStatus::Success);
        assert_eq!(result.gas_consumed(), EvmGas::new(600));
        assert_eq!(result.output_len(), 256);
        assert_eq!(result.error(), None);
        assert!(!result.requires_rollback());
        assert_eq!(&output[..256], &expected);
        assert_eq!(output[256], 0xa5);
        assert_eq!(meter.used(), EvmGas::new(600));
        assert_eq!(work(), [2, 1, 1, 0]);
    }
    Ok(())
}

#[test]
fn g2_add_is_prague_only_and_other_backends_remain_closed() -> Result<(), EvmCoreError> {
    let address = EvmPrecompileKind::Bls12G2Add.address();
    assert_eq!(
        EvmPrecompileRegistry::try_new(EvmFork::CANCUN)?.lookup(address),
        Err(EvmCoreError::PrecompileNotAvailableInFork)
    );
    assert_eq!(
        EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?.lookup(address)?,
        Some(descriptor()?)
    );
    for kind in [
        EvmPrecompileKind::Bls12G2Msm,
        EvmPrecompileKind::Bls12PairingCheck,
        EvmPrecompileKind::Bls12MapFp2ToG2,
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
