#![no_main]

use eth_valkyoth_evm_core::{
    EVM_MAX_GAS_LIMIT, EVM_MODEXP_MAX_OPERAND_BYTES, EvmFork, EvmGas, EvmGasMeter, EvmModexp,
    EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus, parse_modexp_input,
    testing_modexp_gas_cost,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let parsed = parse_modexp_input(data);
    if let Ok(input) = parsed {
        assert!(input.base_len() <= EVM_MODEXP_MAX_OPERAND_BYTES);
        assert!(input.exponent_len() <= EVM_MODEXP_MAX_OPERAND_BYTES);
        assert!(input.modulus_len() <= EVM_MODEXP_MAX_OPERAND_BYTES);
    }

    let mut output = [0u8; EVM_MODEXP_MAX_OPERAND_BYTES];
    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::BERLIN)
        .and_then(|registry| registry.descriptor(EvmPrecompileKind::Modexp))
        .expect("Berlin ModExp descriptor exists");
    if let Ok(quote) = descriptor.quote::<EvmModexp>(data) {
        let mut gas = EvmGasMeter::try_new(EvmGas::new(EVM_MAX_GAS_LIMIT))
            .expect("maximum execution gas is valid");
        if let Ok(outcome) = quote.authorize_and_execute_modexp(&mut gas, &mut output)
            && outcome.status() == EvmPrecompileStatus::Success
        {
            assert!(outcome.output_len() <= EVM_MODEXP_MAX_OPERAND_BYTES);
        }
    }

    let _ = testing_modexp_gas_cost(EvmFork::BYZANTIUM, data);
    let _ = testing_modexp_gas_cost(EvmFork::BERLIN, data);
});
