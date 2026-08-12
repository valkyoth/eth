#![no_main]

use eth_valkyoth_evm_core::{
    EVM_BLAKE2F_INPUT_BYTES, EVM_BLAKE2F_OUTPUT_BYTES, EvmBlake2F, EvmFork, EvmGas,
    EvmGasMeter, EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::ISTANBUL)
        .and_then(|registry| registry.descriptor(EvmPrecompileKind::Blake2F))
        .expect("Istanbul BLAKE2F descriptor exists");
    let quote = descriptor.quote::<EvmBlake2F>(data);
    if data.len() != EVM_BLAKE2F_INPUT_BYTES {
        assert!(quote.is_err());
        return;
    }

    let quote = quote.expect("exact BLAKE2F frame quotes gas");
    let rounds = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    assert_eq!(quote.gas_cost(), EvmGas::new(u64::from(rounds)));
    if rounds > 16 {
        return;
    }

    let mut output = [0u8; EVM_BLAKE2F_OUTPUT_BYTES];
    let mut gas = EvmGasMeter::try_new(EvmGas::new(u64::from(rounds).saturating_add(1)))
        .expect("positive fuzz gas limit");
    let outcome = quote
        .authorize(&mut gas, &mut output)
        .expect("quoted frame fits output and gas")
        .execute_blake2f();
    if matches!(data[EVM_BLAKE2F_INPUT_BYTES - 1], 0 | 1) {
        assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
        assert_eq!(outcome.output_len(), EVM_BLAKE2F_OUTPUT_BYTES);
    } else {
        assert_eq!(outcome.status(), EvmPrecompileStatus::CallFailure);
    }
});
