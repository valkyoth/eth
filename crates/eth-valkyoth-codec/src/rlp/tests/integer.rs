use crate::{DecodeError, DecodeLimits};

use super::super::{
    RlpScalarForm, decode_rlp_integer, decode_rlp_integer_partial, decode_rlp_scalar,
    decode_rlp_u64, decode_rlp_u128, decode_rlp_u256_bytes,
};

#[test]
fn decodes_zero_as_empty_payload() -> Result<(), DecodeError> {
    let integer = decode_rlp_integer(&[0x80], DecodeLimits::TEST_FIXTURE)?;

    assert!(integer.is_zero());
    assert_eq!(integer.payload(), &[]);
    assert_eq!(integer.to_u64(), Ok(0));
    assert_eq!(integer.form(), RlpScalarForm::ShortString);
    Ok(())
}

#[test]
fn decodes_positive_integer_examples() {
    let cases: &[(&[u8], u64)] = &[
        (&[0x01], 1),
        (&[0x0f], 15),
        (&[0x7f], 127),
        (&[0x81, 0x80], 128),
        (&[0x82, 0x04, 0x00], 1024),
    ];

    for (input, expected) in cases {
        assert_eq!(
            decode_rlp_u64(input, DecodeLimits::TEST_FIXTURE),
            Ok(*expected)
        );
    }
}

#[test]
fn converts_integer_to_larger_widths() -> Result<(), DecodeError> {
    assert_eq!(
        decode_rlp_u128(
            &[0x88, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            DecodeLimits::TEST_FIXTURE
        ),
        Ok(u128::from(u64::MAX))
    );

    let bytes = decode_rlp_u256_bytes(&[0x82, 0x04, 0x00], DecodeLimits::TEST_FIXTURE);
    let expected = {
        let mut output = [0_u8; 32];
        let suffix = output.get_mut(30..).ok_or(DecodeError::OffsetOutOfBounds);
        suffix.map(|target| target.copy_from_slice(&[0x04, 0x00]))?;
        output
    };

    assert_eq!(bytes, Ok(expected));
    Ok(())
}

#[test]
fn decodes_u64_max_exactly() {
    let encoded = [0x88, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

    assert_eq!(
        decode_rlp_u64(&encoded, DecodeLimits::TEST_FIXTURE),
        Ok(u64::MAX)
    );
}

#[test]
fn decodes_u128_max_exactly() -> Result<(), DecodeError> {
    let mut encoded = [0xff_u8; 17];
    encoded
        .first_mut()
        .ok_or(DecodeError::OffsetOutOfBounds)
        .map(|prefix| *prefix = 0x90)?;

    assert_eq!(
        decode_rlp_u128(&encoded, DecodeLimits::TEST_FIXTURE),
        Ok(u128::MAX)
    );
    Ok(())
}

#[test]
fn validates_existing_scalar_as_integer() -> Result<(), DecodeError> {
    let scalar = decode_rlp_scalar(&[0x82, 0x04, 0x00], DecodeLimits::TEST_FIXTURE)?;
    let integer = super::super::RlpInteger::try_from_scalar(scalar)?;

    assert_eq!(integer.scalar(), scalar);
    assert_eq!(integer.to_u64(), Ok(1024));
    Ok(())
}

#[test]
fn rejects_single_byte_zero_integer() {
    assert_eq!(
        decode_rlp_integer(&[0x00], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn rejects_leading_zero_integer_payload() {
    assert_eq!(
        decode_rlp_integer(&[0x82, 0x00, 0x01], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn rejects_list_for_integer_decoder() {
    assert_eq!(
        decode_rlp_integer(&[0xc0], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::UnexpectedList)
    );
}

#[test]
fn exact_integer_decoder_rejects_trailing_bytes() {
    assert_eq!(
        decode_rlp_integer(&[0x01, 0x02], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::TrailingBytes)
    );
}

#[test]
fn partial_integer_decoder_leaves_trailing_bytes_to_caller() {
    let mut accumulator = DecodeLimits::TEST_FIXTURE.accumulator();

    assert!(matches!(
        decode_rlp_integer_partial(&[0x01, 0x02], &mut accumulator),
        Ok(integer) if integer.to_u64() == Ok(1) && integer.encoded_len() == 1
    ));
    assert_eq!(accumulator.total_items(), 1);
}

#[test]
fn rejects_integer_overflow_for_target_width() -> Result<(), DecodeError> {
    let nine_byte_integer = [0x89, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let seventeen_byte_integer = [
        0x91, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
    ];
    let mut thirty_three = [1_u8; 34];
    thirty_three
        .first_mut()
        .ok_or(DecodeError::OffsetOutOfBounds)
        .map(|prefix| *prefix = 0xa1)?;

    assert_eq!(
        decode_rlp_u64(&nine_byte_integer, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::LengthOverflow)
    );
    assert_eq!(
        decode_rlp_u128(&seventeen_byte_integer, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::LengthOverflow)
    );
    assert_eq!(
        decode_rlp_u256_bytes(&thirty_three, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::LengthOverflow)
    );
    Ok(())
}
