#![no_main]

use eth_valkyoth_evm_core::{
    EVM_BN254_PAIRING_OUTPUT_BYTES, EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind,
    EvmPrecompilePlan, EvmPrecompileRegistry, parse_bn254_pairing_input,
    testing_bn254_complete_accumulator_pairs, testing_bn254_miller_pairs,
    testing_bn254_post_loop_point_pairs,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let parsed = parse_bn254_pairing_input(data);
    if let Ok(pairs) = parsed {
        assert_eq!(pairs.saturating_mul(192), data.len());
        if let Ok((miller_pairs, _)) = testing_bn254_miller_pairs(data) {
            assert_eq!(miller_pairs, pairs);
        } else {
            panic!("parsed input must be valid for Miller accumulation");
        }
        if let Ok((post_loop_pairs, non_infinity)) = testing_bn254_post_loop_point_pairs(data) {
            assert_eq!(post_loop_pairs, pairs);
            assert!(non_infinity <= pairs);
        } else {
            panic!("parsed input must be valid for post-loop point mapping");
        }
        if let Ok((complete_pairs, _)) = testing_bn254_complete_accumulator_pairs(data) {
            assert_eq!(complete_pairs, pairs);
        } else {
            panic!("parsed input must reach the complete fail-closed accumulator");
        }
    }

    let mut output = [0u8; EVM_BN254_PAIRING_OUTPUT_BYTES];
    if let Ok(len) = execute_bn254_pairing(data, &mut output) {
        assert_eq!(len, EVM_BN254_PAIRING_OUTPUT_BYTES);
        assert_eq!(data.len(), 0);
        assert_eq!(output.last().copied(), Some(1));
    }
});

fn execute_bn254_pairing(
    input: &[u8],
    output: &mut [u8],
) -> Result<usize, eth_valkyoth_evm_core::EvmCoreError> {
    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::ISTANBUL)?
        .descriptor(EvmPrecompileKind::Bn254Pairing)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, input)?;
    let mut gas_meter = EvmGasMeter::try_new(EvmGas::new(10_000_000))?;
    plan.execute_bn254_pairing(&mut gas_meter, input, output)
}
