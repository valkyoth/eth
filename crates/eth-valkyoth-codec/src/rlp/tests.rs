use super::*;
extern crate std;
use std::vec;

mod integer;
mod list;

#[test]
fn decodes_single_byte_scalar() {
    assert_eq!(
        decode_rlp_scalar(&[0x7f], DecodeLimits::TEST_FIXTURE),
        Ok(RlpScalar {
            payload: &[0x7f],
            encoded_len: 1,
            header_len: 0,
            form: RlpScalarForm::SingleByte,
        })
    );
}

#[test]
fn decodes_empty_string() {
    assert_eq!(
        decode_rlp_scalar(&[0x80], DecodeLimits::TEST_FIXTURE),
        Ok(RlpScalar {
            payload: &[],
            encoded_len: 1,
            header_len: 1,
            form: RlpScalarForm::ShortString,
        })
    );
}

#[test]
fn decodes_short_string() {
    assert_eq!(
        decode_rlp_scalar(&[0x83, b'd', b'o', b'g'], DecodeLimits::TEST_FIXTURE),
        Ok(RlpScalar {
            payload: b"dog",
            encoded_len: 4,
            header_len: 1,
            form: RlpScalarForm::ShortString,
        })
    );
}

#[test]
fn decodes_multi_byte_payload() {
    assert_eq!(
        decode_rlp_scalar(&[0x82, 0x04, 0x00], DecodeLimits::TEST_FIXTURE),
        Ok(RlpScalar {
            payload: &[0x04, 0x00],
            encoded_len: 3,
            header_len: 1,
            form: RlpScalarForm::ShortString,
        })
    );
}

#[test]
fn decodes_official_scalar_examples() {
    let lorem = b"Lorem ipsum dolor sit amet, consectetur adipisicing elit";
    let mut long = vec![0xb8, 0x38];
    long.extend_from_slice(lorem);

    let cases: &[(&[u8], &[u8], RlpScalarForm)] = &[
        (
            &[0x83, b'd', b'o', b'g'],
            b"dog",
            RlpScalarForm::ShortString,
        ),
        (&[0x80], b"", RlpScalarForm::ShortString),
        (&[0x00], &[0x00], RlpScalarForm::SingleByte),
        (&[0x0f], &[0x0f], RlpScalarForm::SingleByte),
        (
            &[0x82, 0x04, 0x00],
            &[0x04, 0x00],
            RlpScalarForm::ShortString,
        ),
        (long.as_slice(), lorem, RlpScalarForm::LongString),
    ];

    for (input, expected_payload, expected_form) in cases {
        assert!(matches!(
            decode_rlp_scalar(input, DecodeLimits::TEST_FIXTURE),
            Ok(scalar)
                if scalar.payload() == *expected_payload
                    && scalar.encoded_len() == input.len()
                    && scalar.form() == *expected_form
        ));
    }
}

#[test]
fn decodes_long_string() {
    let mut input = vec![0xb8, 56];
    input.extend_from_slice(&[b'a'; 56]);

    assert!(matches!(
        decode_rlp_scalar(&input, DecodeLimits::TEST_FIXTURE),
        Ok(scalar)
            if scalar.payload().len() == 56
                && scalar.encoded_len() == 58
                && scalar.header_len() == 2
                && scalar.form() == RlpScalarForm::LongString
    ));
}

#[test]
fn partial_decoder_leaves_trailing_bytes_to_caller() {
    let mut accumulator = DecodeLimits::TEST_FIXTURE.accumulator();

    assert!(matches!(
        decode_rlp_scalar_partial(&[0x83, b'd', b'o', b'g', 0x80], &mut accumulator),
        Ok(scalar) if scalar.payload() == b"dog" && scalar.encoded_len() == 4
    ));
    assert_eq!(accumulator.total_items(), 1);
}

#[test]
fn exact_decoder_rejects_trailing_bytes() {
    assert_eq!(
        decode_rlp_scalar(&[0x83, b'd', b'o', b'g', 0x80], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::TrailingBytes)
    );
}

#[test]
fn rejects_empty_input() {
    assert_eq!(
        decode_rlp_scalar(&[], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn rejects_short_string_missing_payload() {
    assert_eq!(
        decode_rlp_scalar(&[0x82, 0x04], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn rejects_noncanonical_single_byte_string() {
    assert_eq!(
        decode_rlp_scalar(&[0x81, 0x7f], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn rejects_long_string_missing_length_bytes() {
    assert_eq!(
        decode_rlp_scalar(&[0xb9, 0x01], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn rejects_long_string_missing_payload() {
    assert_eq!(
        decode_rlp_scalar(&[0xb8, 56, b'a'], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn rejects_long_string_with_leading_zero_length() {
    let mut input = vec![0xb9, 0, 56];
    input.extend_from_slice(&[b'a'; 56]);

    assert_eq!(
        decode_rlp_scalar(&input, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn rejects_long_string_length_overflow() {
    let input = [0xbf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

    assert_eq!(
        decode_rlp_scalar(&input, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::LengthOverflow)
    );
}

#[test]
fn rejects_long_string_used_for_short_payload() {
    let mut input = vec![0xb8, 55];
    input.extend_from_slice(&[b'a'; 55]);

    assert_eq!(
        decode_rlp_scalar(&input, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn rejects_list_prefix_for_scalar_decoder() {
    assert_eq!(
        decode_rlp_scalar(&[0xc0], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::UnexpectedList)
    );
}

#[test]
fn enforces_input_budget() {
    let limits = DecodeLimits {
        max_input_bytes: 1,
        ..DecodeLimits::TEST_FIXTURE
    };

    assert_eq!(
        decode_rlp_scalar(&[0x81, 0x80], limits),
        Err(DecodeError::InputTooLarge)
    );
}

#[test]
fn enforces_item_budget() {
    let limits = DecodeLimits {
        max_total_items: 0,
        ..DecodeLimits::TEST_FIXTURE
    };

    assert_eq!(
        decode_rlp_scalar(&[0x80], limits),
        Err(DecodeError::ItemCountExceeded)
    );
}
