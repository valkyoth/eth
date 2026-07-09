#![no_main]

use eth_valkyoth_evm_core::{
    EVM_BLAKE2F_INPUT_BYTES, EVM_BLAKE2F_OUTPUT_BYTES, EvmFork, EvmGas, EvmGasMeter,
    EvmPrecompileKind, EvmPrecompilePlan, EvmPrecompileRegistry, execute_blake2f,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::ISTANBUL)
        .and_then(|registry| registry.descriptor(EvmPrecompileKind::Blake2F))
        .expect("Istanbul BLAKE2F descriptor exists");
    let plan = EvmPrecompilePlan::try_new(descriptor, data);
    if data.len() != EVM_BLAKE2F_INPUT_BYTES {
        assert!(plan.is_err());
        let mut output = [0u8; EVM_BLAKE2F_OUTPUT_BYTES];
        assert!(execute_blake2f(data, &mut output).is_err());
        return;
    }

    let plan = plan.expect("exact BLAKE2F frame plans gas");
    let rounds = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(u64::from(rounds))));
    if rounds > 16 {
        return;
    }

    let mut output = [0u8; EVM_BLAKE2F_OUTPUT_BYTES];
    let mut gas = EvmGasMeter::try_new(EvmGas::new(u64::from(rounds).saturating_add(1)))
        .expect("positive fuzz gas limit");
    let result = plan.execute_blake2f(data, &mut output, &mut gas);
    if matches!(data[EVM_BLAKE2F_INPUT_BYTES - 1], 0 | 1) {
        assert_eq!(result, Ok(EVM_BLAKE2F_OUTPUT_BYTES));
    } else {
        assert!(result.is_err());
    }
});
