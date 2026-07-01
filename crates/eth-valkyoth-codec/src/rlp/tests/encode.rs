use super::*;
use std::vec;

#[test]
fn encodes_official_scalar_examples() {
    let lorem = b"Lorem ipsum dolor sit amet, consectetur adipisicing elit";
    let cases: &[(&[u8], &[u8])] = &[
        (b"", &[0x80]),
        (b"dog", &[0x83, b'd', b'o', b'g']),
        (&[0x00], &[0x00]),
        (&[0x7f], &[0x7f]),
        (&[0x80], &[0x81, 0x80]),
        (&[0x04, 0x00], &[0x82, 0x04, 0x00]),
    ];

    for (payload, expected) in cases {
        let mut output = [0_u8; 64];
        let written = encode_rlp_scalar(payload, &mut output);
        assert_eq!(written, Ok(expected.len()));
        assert_eq!(output.get(..expected.len()), Some(*expected));
        assert_eq!(encoded_rlp_scalar_len(payload), Ok(expected.len()));
    }

    let mut output = [0_u8; 64];
    let written = encode_rlp_scalar(lorem, &mut output);
    assert_eq!(written, Ok(58));
    assert_eq!(output.first(), Some(&0xb8));
    assert_eq!(output.get(1), Some(&56));
    assert_eq!(output.get(2..58), Some(lorem.as_slice()));
}

#[test]
fn encodes_integer_payloads() {
    let cases: &[(&[u8], &[u8])] = &[
        (b"", &[0x80]),
        (&[0x0f], &[0x0f]),
        (&[0x80], &[0x81, 0x80]),
        (&[0x04, 0x00], &[0x82, 0x04, 0x00]),
    ];

    for (payload, expected) in cases {
        let mut output = [0_u8; 8];
        let written = encode_rlp_integer(payload, &mut output);
        assert_eq!(written, Ok(expected.len()));
        assert_eq!(output.get(..expected.len()), Some(*expected));
        assert_eq!(encoded_rlp_integer_len(payload), Ok(expected.len()));
    }
}

#[test]
fn rejects_noncanonical_integer_payloads() {
    let mut output = [0_u8; 4];
    assert_eq!(
        encode_rlp_integer(&[0x00], &mut output),
        Err(DecodeError::Malformed)
    );
    assert_eq!(
        encoded_rlp_integer_len(&[0x00, 0x01]),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn encodes_list_payloads() {
    let cat_dog_payload = [0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
    let mut output = [0_u8; 64];
    let written =
        encode_rlp_list_payload(&cat_dog_payload, DecodeLimits::TEST_FIXTURE, &mut output);
    let expected = [0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
    assert_eq!(written, Ok(9));
    assert_eq!(output.get(..9), Some(expected.as_slice()));

    let mut empty_output = [0_u8; 1];
    assert_eq!(
        encode_rlp_list_payload(&[], DecodeLimits::TEST_FIXTURE, &mut empty_output),
        Ok(1)
    );
    assert_eq!(empty_output, [0xc0]);
}

#[test]
fn encodes_list_headers_for_streaming_encoders() {
    let mut short_header = [0_u8; 1];
    assert_eq!(encoded_rlp_list_header_len(8), Ok(1));
    assert_eq!(encode_rlp_list_header(8, &mut short_header), Ok(1));
    assert_eq!(short_header, [0xc8]);

    let mut long_header = [0_u8; 2];
    assert_eq!(encoded_rlp_list_header_len(56), Ok(2));
    assert_eq!(encode_rlp_list_header(56, &mut long_header), Ok(2));
    assert_eq!(long_header, [0xf8, 56]);
}

#[test]
fn encodes_long_list_payloads() {
    let payload = [0x80_u8; 56];
    let mut output = [0_u8; 64];
    let written = encode_rlp_list_payload(&payload, DecodeLimits::TEST_FIXTURE, &mut output);

    assert_eq!(written, Ok(58));
    assert_eq!(output.first(), Some(&0xf8));
    assert_eq!(output.get(1), Some(&56));
    assert_eq!(output.get(2..58), Some(payload.as_slice()));
    assert_eq!(
        encoded_rlp_list_len(&payload, DecodeLimits::TEST_FIXTURE),
        Ok(58)
    );
}

#[test]
fn validates_list_payloads_before_encoding() {
    let mut output = [0_u8; 4];
    assert_eq!(
        encode_rlp_list_payload(&[0x81], DecodeLimits::TEST_FIXTURE, &mut output),
        Err(DecodeError::OffsetOutOfBounds)
    );
    assert_eq!(
        encoded_rlp_list_len(&[0x81], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn decoded_scalars_round_trip_to_canonical_encoding() -> Result<(), DecodeError> {
    let mut long = vec![0xb8, 56];
    long.extend_from_slice(&[b'a'; 56]);
    let inputs: &[&[u8]] = &[
        &[0x00],
        &[0x7f],
        &[0x80],
        &[0x81, 0x80],
        &[0x83, b'd', b'o', b'g'],
        long.as_slice(),
    ];

    for input in inputs {
        let scalar = decode_rlp_scalar(input, DecodeLimits::TEST_FIXTURE)?;
        let mut output = [0_u8; 64];
        let written = encode_decoded_scalar(scalar, &mut output);
        assert_eq!(written, Ok(input.len()));
        assert_eq!(output.get(..input.len()), Some(*input));
    }
    Ok(())
}

#[test]
fn decoded_integers_round_trip_to_canonical_encoding() -> Result<(), DecodeError> {
    let inputs: &[&[u8]] = &[&[0x80], &[0x01], &[0x81, 0x80], &[0x82, 0x04, 0x00]];

    for input in inputs {
        let integer = decode_rlp_integer(input, DecodeLimits::TEST_FIXTURE)?;
        let mut output = [0_u8; 8];
        let written = encode_decoded_integer(integer, &mut output);
        assert_eq!(written, Ok(input.len()));
        assert_eq!(output.get(..input.len()), Some(*input));
    }
    Ok(())
}

#[test]
fn decoded_lists_and_items_round_trip_to_canonical_encoding() -> Result<(), DecodeError> {
    let input = [0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
    let list = decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE)?;

    let mut output = [0_u8; 16];
    let written = encode_decoded_list(list, &mut output);
    assert_eq!(written, Ok(input.len()));
    assert_eq!(output.get(..input.len()), Some(input.as_slice()));

    for item in list.items() {
        let item = item?;
        let mut item_output = [0_u8; 8];
        let item_written = encode_decoded_item(item, &mut item_output)?;
        assert_eq!(item_written, item.encoded_len());
    }
    Ok(())
}

#[test]
fn output_buffers_must_be_large_enough() {
    let mut output = [0_u8; 3];
    assert_eq!(
        encode_rlp_scalar(b"dog", &mut output),
        Err(DecodeError::OffsetOutOfBounds)
    );
    assert_eq!(output, [0_u8; 3]);

    let mut empty_output = [];
    assert_eq!(
        encode_rlp_list_payload(&[], DecodeLimits::TEST_FIXTURE, &mut empty_output),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn output_buffers_are_not_modified_on_encode_error() {
    let mut short_output = [0xaa_u8; 1];
    assert_eq!(
        encode_rlp_scalar(b"dog", &mut short_output),
        Err(DecodeError::OffsetOutOfBounds)
    );
    assert_eq!(short_output, [0xaa]);

    let payload = [0x80_u8; 56];
    let mut long_output = [0xbb_u8; 1];
    assert_eq!(
        encode_rlp_list_payload(&payload, DecodeLimits::TEST_FIXTURE, &mut long_output),
        Err(DecodeError::OffsetOutOfBounds)
    );
    assert_eq!(long_output, [0xbb]);

    let mut integer_output = [0xcc_u8; 4];
    assert_eq!(
        encode_rlp_integer(&[0x00], &mut integer_output),
        Err(DecodeError::Malformed)
    );
    assert_eq!(integer_output, [0xcc_u8; 4]);
}

#[test]
fn noncanonical_inputs_still_fail_before_reencoding() {
    assert_eq!(
        decode_rlp_scalar(&[0x81, 0x7f], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
    assert_eq!(
        decode_rlp_integer(&[0x00], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
    assert_eq!(
        decode_rlp_list(&[0xf8, 0x00], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}
