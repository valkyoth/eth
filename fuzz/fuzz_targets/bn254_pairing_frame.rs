#![no_main]

use eth_valkyoth_evm_core::{
    EVM_BN254_PAIRING_OUTPUT_BYTES, execute_bn254_pairing, parse_bn254_pairing_input,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let parsed = parse_bn254_pairing_input(data);
    if let Ok(pairs) = parsed {
        assert_eq!(pairs.saturating_mul(192), data.len());
    }

    let mut output = [0u8; EVM_BN254_PAIRING_OUTPUT_BYTES];
    if let Ok(len) = execute_bn254_pairing(data, &mut output) {
        assert_eq!(len, EVM_BN254_PAIRING_OUTPUT_BYTES);
        assert_eq!(data.len(), 0);
        assert_eq!(output.last().copied(), Some(1));
    }
});
