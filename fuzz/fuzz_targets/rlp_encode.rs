#![no_main]

use eth_valkyoth_codec::{
    DecodeLimits, decode_rlp_integer, decode_rlp_list, decode_rlp_scalar, encode_decoded_integer,
    encode_decoded_item, encode_decoded_list, encode_decoded_scalar, encode_rlp_integer,
    encode_rlp_list_payload, encode_rlp_scalar, encoded_rlp_integer_len, encoded_rlp_list_len,
    encoded_rlp_scalar_len,
};
use libfuzzer_sys::fuzz_target;
use std::vec;

fuzz_target!(|data: &[u8]| {
    let limits = DecodeLimits::TEST_FIXTURE;

    if let Ok(required) = encoded_rlp_scalar_len(data) {
        let mut output = vec![0_u8; required];
        let _ = encode_rlp_scalar(data, &mut output);
    }
    if let Ok(required) = encoded_rlp_integer_len(data) {
        let mut output = vec![0_u8; required];
        let _ = encode_rlp_integer(data, &mut output);
    }
    if let Ok(required) = encoded_rlp_list_len(data, limits) {
        let mut output = vec![0_u8; required];
        let _ = encode_rlp_list_payload(data, limits, &mut output);
    }

    if let Ok(scalar) = decode_rlp_scalar(data, limits) {
        let mut output = vec![0_u8; scalar.encoded_len()];
        let _ = encode_decoded_scalar(scalar, &mut output);
    }
    if let Ok(integer) = decode_rlp_integer(data, limits) {
        let mut output = vec![0_u8; integer.encoded_len()];
        let _ = encode_decoded_integer(integer, &mut output);
    }
    if let Ok(list) = decode_rlp_list(data, limits) {
        let mut output = vec![0_u8; list.encoded_len()];
        let _ = encode_decoded_list(list, &mut output);
        for item in list.items().flatten() {
            let mut output = vec![0_u8; item.encoded_len()];
            let _ = encode_decoded_item(item, &mut output);
        }
    }
});
