#![no_main]

use eth_valkyoth_evm_core::{
    EVM_BLS12381_FP2_BYTES, EVM_BLS12381_FP_BYTES, EVM_BLS12381_G1_POINT_BYTES,
    EVM_BLS12381_G2_POINT_BYTES, parse_bls12381_g1_add, parse_bls12381_g1_msm,
    parse_bls12381_g2_add, parse_bls12381_g2_msm, parse_bls12381_map_fp_to_g1,
    parse_bls12381_map_fp2_to_g2, parse_bls12381_pairing,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, input)) = data.split_first() else {
        return;
    };
    match selector % 7 {
        0 => {
            if let Ok(frame) = parse_bls12381_g1_add(input) {
                assert_eq!(frame.left.to_be_bytes().as_slice(), &input[..EVM_BLS12381_G1_POINT_BYTES]);
                assert_eq!(frame.right.to_be_bytes().as_slice(), &input[EVM_BLS12381_G1_POINT_BYTES..]);
            }
        }
        1 => {
            if let Ok(frame) = parse_bls12381_g1_msm(input) {
                assert!(!frame.is_empty());
                assert_eq!(frame.items().count(), frame.len());
                assert!(frame.items().all(|item| item.is_ok()));
            }
        }
        2 => {
            if let Ok(frame) = parse_bls12381_g2_add(input) {
                assert_eq!(frame.left.to_be_bytes().as_slice(), &input[..EVM_BLS12381_G2_POINT_BYTES]);
                assert_eq!(frame.right.to_be_bytes().as_slice(), &input[EVM_BLS12381_G2_POINT_BYTES..]);
            }
        }
        3 => {
            if let Ok(frame) = parse_bls12381_g2_msm(input) {
                assert!(!frame.is_empty());
                assert_eq!(frame.items().count(), frame.len());
                assert!(frame.items().all(|item| item.is_ok()));
            }
        }
        4 => {
            if let Ok(frame) = parse_bls12381_pairing(input) {
                assert!(!frame.is_empty());
                assert_eq!(frame.items().count(), frame.len());
                assert!(frame.items().all(|item| item.is_ok()));
            }
        }
        5 => {
            if let Ok(value) = parse_bls12381_map_fp_to_g1(input) {
                assert_eq!(input.len(), EVM_BLS12381_FP_BYTES);
                assert_eq!(value.to_be_bytes().as_slice(), input);
            }
        }
        _ => {
            if let Ok(value) = parse_bls12381_map_fp2_to_g2(input) {
                assert_eq!(input.len(), EVM_BLS12381_FP2_BYTES);
                assert_eq!(value.to_be_bytes().as_slice(), input);
            }
        }
    }
});
