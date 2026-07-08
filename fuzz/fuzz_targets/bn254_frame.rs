#![no_main]

use eth_valkyoth_evm_core::{
    EVM_BN254_POINT_BYTES, execute_bn254_add, execute_bn254_mul,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut output = [0u8; EVM_BN254_POINT_BYTES];
    if let Ok(len) = execute_bn254_add(data, &mut output) {
        assert_eq!(len, EVM_BN254_POINT_BYTES);
    }

    output.fill(0);
    if let Ok(len) = execute_bn254_mul(data, &mut output) {
        assert_eq!(len, EVM_BN254_POINT_BYTES);
    }
});
