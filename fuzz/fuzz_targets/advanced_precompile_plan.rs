#![no_main]

use eth_valkyoth_evm_core::{
    EvmFork, EvmPrecompileInputPolicy, EvmPrecompileKind, EvmPrecompilePlan,
    EvmPrecompileRegistry,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, input)) = data.split_first() else {
        return;
    };
    let kind = match selector & 7 {
        0 => EvmPrecompileKind::KzgPointEvaluation,
        1 => EvmPrecompileKind::Bls12G1Add,
        2 => EvmPrecompileKind::Bls12G1Msm,
        3 => EvmPrecompileKind::Bls12G2Add,
        4 => EvmPrecompileKind::Bls12G2Msm,
        5 => EvmPrecompileKind::Bls12PairingCheck,
        6 => EvmPrecompileKind::Bls12MapFpToG1,
        _ => EvmPrecompileKind::Bls12MapFp2ToG2,
    };
    let descriptor = EvmPrecompileRegistry::try_new(kind.introduced_in())
        .and_then(|registry| registry.descriptor(kind))
        .expect("advanced precompile descriptor exists in its introduction fork");
    let expected_valid = match descriptor.input_policy {
        EvmPrecompileInputPolicy::Exact(expected) => input.len() == expected,
        EvmPrecompileInputPolicy::NonEmptyMultipleOf(item) => {
            !input.is_empty() && item != 0 && input.len().is_multiple_of(item)
        }
        EvmPrecompileInputPolicy::BoundedAny | EvmPrecompileInputPolicy::MultipleOf(_) => false,
    };
    let plan = EvmPrecompilePlan::try_new(descriptor, input);
    assert_eq!(plan.is_ok(), expected_valid);
    if let Ok(plan) = plan {
        assert!(plan.gas_cost().is_some());
        assert_eq!(plan.input_len(), input.len());
        assert!(plan.descriptor().output_len.is_some());
        assert!(matches!(
            plan.descriptor().fork,
            EvmFork::CANCUN | EvmFork::PRAGUE
        ));
    }
});
