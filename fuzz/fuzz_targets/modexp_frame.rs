#![no_main]

use eth_valkyoth_evm_core::{
    EVM_MAX_GAS_LIMIT, EvmFork, EvmGas, EvmGasMeter, EvmModExpWorkspace, EvmModexp,
    EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus, parse_modexp_input,
    testing_modexp_gas_cost,
};
use libfuzzer_sys::fuzz_target;

const FUZZ_EXECUTION_LIMBS: usize = 4_096;
const FUZZ_OUTPUT_BYTES: usize = 4_096;

fuzz_target!(|data: &[u8]| {
    let parsed = parse_modexp_input(data);
    if let Ok(input) = parsed {
        let _ = input.base_len().to_be_bytes();
        let _ = input.exponent_len().to_be_bytes();
        let _ = input.modulus_len().to_be_bytes();
    }

    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::BERLIN)
        .and_then(|registry| registry.descriptor(EvmPrecompileKind::Modexp))
        .expect("Berlin ModExp descriptor exists");
    if let Ok(quote) = descriptor.quote::<EvmModexp>(data) {
        let output_len = quote.output_len();
        let workspace_limbs = quote.modexp_workspace_limbs();
        if output_len <= FUZZ_OUTPUT_BYTES
            && let Ok(workspace_limbs) = workspace_limbs
            && workspace_limbs <= FUZZ_EXECUTION_LIMBS
        {
            let mut output = vec![0_u8; output_len];
            let mut storage = vec![0_u32; workspace_limbs];
            let mut workspace = EvmModExpWorkspace::new(&mut storage);
            let mut gas = EvmGasMeter::try_new(EvmGas::new(EVM_MAX_GAS_LIMIT))
                .expect("maximum execution gas is valid");
            if let Ok(outcome) = quote.authorize_and_execute_modexp(
                &mut gas,
                &mut output,
                &mut workspace,
            ) && outcome.status() == EvmPrecompileStatus::Success
            {
                assert_eq!(outcome.output_len(), output_len);
            }
        }
    }

    let _ = testing_modexp_gas_cost(EvmFork::BYZANTIUM, data);
    let _ = testing_modexp_gas_cost(EvmFork::BERLIN, data);
});
