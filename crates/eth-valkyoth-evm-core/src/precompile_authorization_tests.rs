use crate::{
    EvmBlake2F, EvmBn254Add, EvmCoreError, EvmEcRecover, EvmEcRecoverBackend,
    EvmEcRecoverSignature, EvmFork, EvmGas, EvmGasMeter, EvmIdentity, EvmPrecompileGasPolicy,
    EvmPrecompileKeccak256, EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus,
    EvmSha256,
};
use std::format;

fn descriptor(
    fork: EvmFork,
    kind: EvmPrecompileKind,
) -> Result<crate::EvmPrecompileDescriptor, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)?.descriptor(kind)
}

#[test]
fn exact_input_quote_authorizes_one_success_outcome() -> Result<(), EvmCoreError> {
    let input = b"eth";
    let quote =
        descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Identity)?.quote::<EvmIdentity>(input)?;
    assert_eq!(quote.gas_cost(), EvmGas::new(18));
    assert_eq!(quote.output_len(), input.len());

    let mut meter = EvmGasMeter::try_new(EvmGas::new(18))?;
    let mut output = [0_u8; 3];
    let outcome = quote.authorize_and_execute_identity(&mut meter, &mut output)?;

    assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
    assert_eq!(outcome.gas_consumed(), EvmGas::new(18));
    assert_eq!(outcome.output_len(), output.len());
    assert_eq!(outcome.error(), None);
    assert!(!outcome.requires_rollback());
    assert_eq!(&output, input);
    assert_eq!(meter.used(), EvmGas::new(18));
    Ok(())
}

#[test]
fn output_and_gas_admission_fail_before_mutation() -> Result<(), EvmCoreError> {
    let input = [5_u8; 33];
    let descriptor = descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Identity)?;
    let mut output = [9_u8; 33];
    let mut short_output = [9_u8; 32];

    let mut exact_meter = EvmGasMeter::try_new(EvmGas::new(21))?;
    assert_eq!(
        descriptor
            .quote::<EvmIdentity>(&input)?
            .authorize_internal(&mut exact_meter, &mut short_output)
            .err(),
        Some(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(exact_meter.used(), EvmGas::new(0));
    assert_eq!(output, [9_u8; 33]);

    let mut short_meter = EvmGasMeter::try_new(EvmGas::new(20))?;
    assert_eq!(
        descriptor
            .quote::<EvmIdentity>(&input)?
            .authorize_internal(&mut short_meter, &mut output)
            .err(),
        Some(EvmCoreError::OutOfGas)
    );
    assert_eq!(short_meter.used(), EvmGas::new(0));
    assert_eq!(output, [9_u8; 33]);
    Ok(())
}

#[test]
fn execution_failure_consumes_supplied_gas_and_requests_rollback() -> Result<(), EvmCoreError> {
    let mut invalid = [0_u8; 213];
    invalid[..4].copy_from_slice(&1_u32.to_be_bytes());
    invalid[212] = 2;
    let quote =
        descriptor(EvmFork::ISTANBUL, EvmPrecompileKind::Blake2F)?.quote::<EvmBlake2F>(&invalid)?;
    let mut meter = EvmGasMeter::try_new(EvmGas::new(77))?;
    let mut output = [11_u8; 64];

    let outcome = quote
        .authorize_internal(&mut meter, &mut output)?
        .execute_blake2f();

    assert_eq!(outcome.status(), EvmPrecompileStatus::CallFailure);
    assert_eq!(outcome.gas_consumed(), EvmGas::new(77));
    assert_eq!(outcome.output_len(), 0);
    assert_eq!(
        outcome.error(),
        Some(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert!(outcome.requires_rollback());
    assert_eq!(meter.used(), EvmGas::new(77));
    assert_eq!(output, [11_u8; 64]);
    Ok(())
}

#[test]
fn dropping_paid_authority_consumes_the_complete_child_meter() -> Result<(), EvmCoreError> {
    let input = b"eth";
    let quote =
        descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Identity)?.quote::<EvmIdentity>(input)?;
    let mut meter = EvmGasMeter::try_new(EvmGas::new(100))?;
    meter.charge(EvmGas::new(7))?;
    let mut output = [0_u8; 3];

    let paid = quote.authorize_internal(&mut meter, &mut output)?;
    drop(paid);

    assert_eq!(meter.used(), meter.limit());
    assert_eq!(output, [0_u8; 3]);
    Ok(())
}

#[test]
fn successful_paid_execution_leaves_unused_child_gas() -> Result<(), EvmCoreError> {
    let input = b"eth";
    let quote =
        descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Identity)?.quote::<EvmIdentity>(input)?;
    let mut meter = EvmGasMeter::try_new(EvmGas::new(100))?;
    let mut output = [0_u8; 3];

    let outcome = quote.authorize_and_execute_identity(&mut meter, &mut output)?;

    assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
    assert_eq!(meter.used(), EvmGas::new(18));
    assert_eq!(&output, input);
    Ok(())
}

#[test]
fn panicking_ecrecover_backend_consumes_the_complete_child_meter() -> Result<(), EvmCoreError> {
    let seed = runtime_test_byte();
    let mut input = [seed ^ seed; 128];
    input[63] = 27;
    input[95] = seed;
    input[127] = seed;
    let quote = descriptor(EvmFork::FRONTIER, EvmPrecompileKind::EcRecover)?
        .quote::<EvmEcRecover>(&input)?;
    let mut meter = EvmGasMeter::try_new(EvmGas::new(4_000))?;
    let mut output = [seed; 32];

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let result = quote.authorize_and_execute_ecrecover(
            &mut meter,
            &mut output,
            PanickingBackend,
            UnreachableHasher,
        );
        assert!(result.is_ok());
    }));

    assert!(unwind.is_err());
    assert_eq!(meter.used(), meter.limit());
    assert!(output.iter().all(|byte| *byte == seed));
    Ok(())
}

struct PanickingBackend;

impl EvmEcRecoverBackend for PanickingBackend {
    fn recover_uncompressed_public_key(
        &mut self,
        _digest: [u8; 32],
        _signature: EvmEcRecoverSignature,
    ) -> Option<[u8; 64]> {
        assert!(std::thread::panicking(), "intentional test backend panic");
        None
    }
}

struct UnreachableHasher;

impl EvmPrecompileKeccak256 for UnreachableHasher {
    fn keccak256(&mut self, _input: &[u8]) -> [u8; 32] {
        assert!(std::thread::panicking(), "hasher must not be reached");
        [0_u8; 32]
    }
}

#[test]
fn canonical_registry_rejects_forged_gas_metadata() -> Result<(), EvmCoreError> {
    let mut forged = descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Sha256)?;
    forged.gas_policy = EvmPrecompileGasPolicy::Fixed(EvmGas::new(1));
    assert_eq!(
        forged.quote::<EvmSha256>(b"expensive input").err(),
        Some(EvmCoreError::PrecompileDescriptorMismatch)
    );
    Ok(())
}

#[test]
fn marker_kind_mismatch_cannot_authorize_work() -> Result<(), EvmCoreError> {
    let identity = descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Identity)?;
    assert_eq!(
        identity.quote::<EvmSha256>(b"eth").err(),
        Some(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[test]
fn gas_replaces_the_old_global_input_ceiling() -> Result<(), EvmCoreError> {
    static INPUT: [u8; 1_048_577] = [3_u8; 1_048_577];
    let quote =
        descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Identity)?.quote::<EvmIdentity>(&INPUT)?;
    assert_eq!(quote.output_len(), INPUT.len());
    assert!(quote.gas_cost().get() > 98_000);

    let mut meter = EvmGasMeter::try_new(EvmGas::new(98_000))?;
    let mut output = std::vec![0_u8; INPUT.len()];
    assert_eq!(
        quote.authorize_internal(&mut meter, &mut output).err(),
        Some(EvmCoreError::OutOfGas)
    );
    assert_eq!(meter.used(), EvmGas::new(0));
    Ok(())
}

#[test]
fn expensive_curve_validation_is_reached_only_after_payment() -> Result<(), EvmCoreError> {
    let seed = runtime_test_byte();
    let zero = seed ^ seed;
    let mut invalid_point = [zero; 128];
    let y_low = invalid_point
        .get_mut(63)
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    *y_low = seed;
    let quote = descriptor(EvmFork::ISTANBUL, EvmPrecompileKind::Bn254Add)?
        .quote::<EvmBn254Add>(&invalid_point)?;
    let mut meter = EvmGasMeter::try_new(EvmGas::new(500))?;
    let mut output = [seed; 64];

    let outcome = quote
        .authorize_internal(&mut meter, &mut output)?
        .execute_bn254_add();
    assert_eq!(outcome.status(), EvmPrecompileStatus::CallFailure);
    assert_eq!(meter.used(), meter.limit());
    assert!(output.iter().all(|byte| *byte == seed));
    Ok(())
}

fn runtime_test_byte() -> u8 {
    std::process::id()
        .to_ne_bytes()
        .into_iter()
        .fold(u8::MIN, |combined, byte| combined | byte)
}

#[test]
fn quote_debug_redacts_exact_input() -> Result<(), EvmCoreError> {
    let quote = descriptor(EvmFork::FRONTIER, EvmPrecompileKind::Identity)?
        .quote::<EvmIdentity>(b"sensitive transaction bytes")?;
    let rendered = format!("{quote:?}");
    assert!(rendered.contains("input_len"));
    assert!(!rendered.contains("sensitive"));
    Ok(())
}
