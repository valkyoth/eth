#![no_main]

use eth_valkyoth_evm_core::{
    EVM_MODEXP_MAX_OPERAND_BYTES, EvmFork, execute_modexp, parse_modexp_input,
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
    let result = execute_modexp(data, &mut output);
    if let Ok(len) = result {
        assert!(len <= EVM_MODEXP_MAX_OPERAND_BYTES);
    }

    let _ = testing_modexp_gas_cost(EvmFork::BYZANTIUM, data);
    let _ = testing_modexp_gas_cost(EvmFork::BERLIN, data);
});
