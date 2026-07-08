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

    if let Some(point) = data.get(..EVM_BN254_POINT_BYTES) {
        let mut add_input = [0u8; 128];
        if let Some(left) = add_input.get_mut(..64) {
            left.copy_from_slice(point);
        }
        if let Some(right) = add_input.get_mut(64..) {
            right.copy_from_slice(point);
        }
        let mut mul_input = [0u8; 96];
        if let Some(target) = mul_input.get_mut(..64) {
            target.copy_from_slice(point);
        }
        if let Some(scalar_low) = mul_input.get_mut(95) {
            *scalar_low = 2;
        }

        let mut add_output = [0u8; EVM_BN254_POINT_BYTES];
        let mut mul_output = [0u8; EVM_BN254_POINT_BYTES];
        if execute_bn254_add(&add_input, &mut add_output).is_ok()
            && execute_bn254_mul(&mul_input, &mut mul_output).is_ok()
        {
            assert_eq!(add_output, mul_output);
        }
    }
});
