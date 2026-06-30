#![no_main]

use eth_valkyoth_codec::{
    DecodeLimits, decode_rlp_integer, decode_rlp_list, decode_rlp_scalar, encode_decoded_integer,
    encode_decoded_item, encode_decoded_list, encode_decoded_scalar, encode_rlp_integer,
    encode_rlp_list_payload, encode_rlp_scalar, encoded_rlp_integer_len, encoded_rlp_list_len,
    encoded_rlp_scalar_len,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = DecodeLimits::TEST_FIXTURE;
    let mut output = [0_u8; 512];

    let _ = encoded_rlp_scalar_len(data);
    let _ = encoded_rlp_integer_len(data);
    let _ = encoded_rlp_list_len(data);
    let _ = encode_rlp_scalar(data, &mut output);
    let _ = encode_rlp_integer(data, &mut output);
    let _ = encode_rlp_list_payload(data, &mut output);

    if let Ok(scalar) = decode_rlp_scalar(data, limits) {
        let _ = encode_decoded_scalar(scalar, &mut output);
    }
    if let Ok(integer) = decode_rlp_integer(data, limits) {
        let _ = encode_decoded_integer(integer, &mut output);
    }
    if let Ok(list) = decode_rlp_list(data, limits) {
        let _ = encode_decoded_list(list, &mut output);
        for item in list.items() {
            if let Ok(item) = item {
                let _ = encode_decoded_item(item, &mut output);
            }
        }
    }
});
