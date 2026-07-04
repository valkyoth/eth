#![no_main]

use alloy_rlp::Header;
use eth_valkyoth_codec::{
    DecodeError, DecodeErrorCategory, DecodeLimits, decode_rlp_list, decode_rlp_scalar,
    encode_decoded_list, encode_decoded_scalar,
};
use libfuzzer_sys::fuzz_target;
use std::vec;

fuzz_target!(|data: &[u8]| {
    let alloy = alloy_accepts(data);
    let valkyoth = valkyoth_round_trip(data);

    match (alloy, valkyoth) {
        (true, Ok(round_trip)) => assert_eq!(round_trip, data),
        (true, Err(error)) if is_budget_rejection(error) => {}
        (true, Err(error)) => panic!("valkyoth rejected alloy-accepted RLP: {error:?}"),
        (false, Ok(_)) => panic!("valkyoth accepted alloy-rejected RLP"),
        (false, Err(_)) => {}
    }
});

fn alloy_accepts(input: &[u8]) -> bool {
    let mut remaining = input;
    match Header::decode_raw(&mut remaining) {
        Ok(_) => remaining.is_empty(),
        Err(_) => false,
    }
}

fn valkyoth_round_trip(input: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let prefix = *input.first().ok_or(DecodeError::Malformed)?;
    let mut output = vec![0_u8; input.len()];
    let written = if prefix <= 0xbf {
        let scalar = decode_rlp_scalar(input, DecodeLimits::TEST_FIXTURE)?;
        encode_decoded_scalar(scalar, &mut output)?
    } else {
        let list = decode_rlp_list(input, DecodeLimits::TEST_FIXTURE)?;
        encode_decoded_list(list, &mut output)?
    };
    output.truncate(written);
    Ok(output)
}

fn is_budget_rejection(error: DecodeError) -> bool {
    error.category() == DecodeErrorCategory::ResourceExhaustion
}
