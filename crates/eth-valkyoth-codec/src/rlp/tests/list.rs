use crate::{DecodeError, DecodeLimits};

use super::{
    super::{RlpItem, RlpListForm, RlpScalarForm, decode_rlp_list, decode_rlp_list_partial},
    vec,
    vec::Vec,
};

#[test]
fn decodes_empty_list() {
    assert!(matches!(
        decode_rlp_list(&[0xc0], DecodeLimits::TEST_FIXTURE),
        Ok(list)
            if list.payload().is_empty()
                && list.encoded_len() == 1
                && list.header_len() == 1
                && list.item_count() == 0
                && list.form() == RlpListForm::ShortList
                && list.is_empty()
    ));
}

#[test]
fn list_equality_ignores_decode_policy() -> Result<(), DecodeError> {
    let fixture = decode_rlp_list(&[0xc0], DecodeLimits::TEST_FIXTURE)?;
    let deployment = decode_rlp_list(&[0xc0], DecodeLimits::DEPLOYMENT_STARTING_POINT)?;

    assert_eq!(fixture, deployment);
    Ok(())
}

#[test]
fn decodes_short_list_with_scalars() {
    let input = [0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];

    assert!(matches!(
        decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE),
        Ok(list)
            if list.payload() == &input[1..]
                && list.encoded_len() == input.len()
                && list.item_count() == 2
                && list.form() == RlpListForm::ShortList
    ));
}

#[test]
fn decodes_official_nested_list_examples() -> Result<(), DecodeError> {
    let set = [0xc0];
    let empty_set = [0xc1, 0xc0];
    let two_empty_sets = [0xc2, 0xc0, 0xc0];
    let mixed = [0xc7, b'z', 0xc1, b'y', 0xc3, b'x', b'y', b'z'];

    assert!(decode_rlp_list(&set, DecodeLimits::TEST_FIXTURE)?.is_empty());
    assert_eq!(
        decode_rlp_list(&empty_set, DecodeLimits::TEST_FIXTURE)?.item_count(),
        1
    );
    assert_eq!(
        decode_rlp_list(&two_empty_sets, DecodeLimits::TEST_FIXTURE)?.item_count(),
        2
    );

    let list = decode_rlp_list(&mixed, DecodeLimits::TEST_FIXTURE)?;
    let mut items = list.items();
    assert!(matches!(
        items.next(),
        Some(Ok(RlpItem::Scalar(scalar))) if scalar.payload() == b"z"
    ));
    assert!(matches!(
        items.next(),
        Some(Ok(RlpItem::List(child))) if child.item_count() == 1
    ));
    assert!(matches!(
        items.next(),
        Some(Ok(RlpItem::List(child))) if child.item_count() == 3
    ));
    assert!(items.next().is_none());
    Ok(())
}

#[test]
fn decodes_nested_short_lists() {
    let input = [0xc3, 0xc0, 0xc1, 0xc0];

    assert!(matches!(
        decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE),
        Ok(list)
            if list.payload() == &input[1..]
                && list.encoded_len() == input.len()
                && list.item_count() == 2
    ));
}

#[test]
fn iterates_empty_list_items() -> Result<(), DecodeError> {
    let list = decode_rlp_list(&[0xc0], DecodeLimits::TEST_FIXTURE)?;
    let mut items = list.items();

    assert_eq!(items.remaining(), 0);
    assert_eq!(items.size_hint(), (0, Some(0)));
    assert!(items.next().is_none());
    Ok(())
}

#[test]
fn iterates_scalar_list_items() -> Result<(), DecodeError> {
    let input = [0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
    let list = decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE)?;
    let mut items = list.items();

    assert_eq!(items.remaining(), 2);
    assert!(matches!(
        items.next(),
        Some(Ok(RlpItem::Scalar(scalar)))
            if scalar.payload() == b"cat"
                && scalar.encoded_len() == 4
                && scalar.form() == RlpScalarForm::ShortString
    ));
    assert_eq!(items.remaining(), 1);
    assert!(matches!(
        items.next(),
        Some(Ok(RlpItem::Scalar(scalar)))
            if scalar.payload() == b"dog"
                && scalar.encoded_len() == 4
                && scalar.form() == RlpScalarForm::ShortString
    ));
    assert!(items.next().is_none());
    Ok(())
}

#[test]
fn iterates_nested_list_items() -> Result<(), DecodeError> {
    let input = [0xc4, 0xc0, 0xc2, 0x80, 0x01];
    let list = decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE)?;
    let mut items = list.items();

    assert!(matches!(
        items.next(),
        Some(Ok(RlpItem::List(child))) if child.is_empty() && child.item_count() == 0
    ));

    let Some(Ok(RlpItem::List(child))) = items.next() else {
        return Err(DecodeError::Malformed);
    };
    assert_eq!(child.item_count(), 2);
    let mut child_items = child.items();
    assert!(matches!(
        child_items.next(),
        Some(Ok(RlpItem::Scalar(scalar)))
            if scalar.payload().is_empty() && scalar.form() == RlpScalarForm::ShortString
    ));
    assert!(matches!(
        child_items.next(),
        Some(Ok(RlpItem::Scalar(scalar)))
            if scalar.payload() == [0x01] && scalar.form() == RlpScalarForm::SingleByte
    ));
    assert!(items.next().is_none());
    Ok(())
}

#[test]
fn item_accessors_return_expected_variant() -> Result<(), DecodeError> {
    let input = [0xc3, 0x80, 0xc1, 0x01];
    let list = decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE)?;
    let mut items = list.items();

    let Some(Ok(scalar_item)) = items.next() else {
        return Err(DecodeError::Malformed);
    };
    assert!(scalar_item.is_scalar());
    assert!(!scalar_item.is_list());
    assert_eq!(scalar_item.header_len(), 1);
    assert!(scalar_item.as_list().is_none());
    assert!(matches!(
        scalar_item.as_scalar(),
        Some(scalar) if scalar.payload().is_empty()
    ));

    let Some(Ok(list_item)) = items.next() else {
        return Err(DecodeError::Malformed);
    };
    assert!(list_item.is_list());
    assert!(!list_item.is_scalar());
    assert_eq!(list_item.header_len(), 1);
    assert!(list_item.as_scalar().is_none());
    assert!(matches!(
        list_item.as_list(),
        Some(child) if child.item_count() == 1
    ));
    Ok(())
}

#[test]
fn list_item_iterator_is_fused() -> Result<(), DecodeError> {
    fn assert_fused<I: core::iter::FusedIterator>(_iterator: &I) {}

    let list = decode_rlp_list(&[0xc0], DecodeLimits::TEST_FIXTURE)?;
    let mut items = list.items();
    assert_fused(&items);

    assert!(items.next().is_none());
    assert!(items.next().is_none());
    Ok(())
}

#[test]
fn decodes_long_list() {
    let mut input = vec![0xf8, 56];
    input.extend_from_slice(&[0x80; 56]);

    assert!(matches!(
        decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE),
        Ok(list)
            if list.payload().len() == 56
                && list.encoded_len() == 58
                && list.header_len() == 2
                && list.item_count() == 56
                && list.form() == RlpListForm::LongList
    ));
}

#[test]
fn partial_list_decoder_leaves_trailing_bytes_to_caller() {
    let mut accumulator = DecodeLimits::TEST_FIXTURE.accumulator();

    assert!(matches!(
        decode_rlp_list_partial(&[0xc1, 0x80, 0x80], &mut accumulator),
        Ok(list) if list.item_count() == 1 && list.encoded_len() == 2
    ));
    assert_eq!(accumulator.total_items(), 2);
}

#[test]
fn exact_list_decoder_rejects_trailing_bytes() {
    assert_eq!(
        decode_rlp_list(&[0xc1, 0x80, 0x80], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::TrailingBytes)
    );
}

#[test]
fn rejects_scalar_for_list_decoder() {
    assert_eq!(
        decode_rlp_list(&[0x80], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::UnexpectedScalar)
    );
}

#[test]
fn rejects_short_list_missing_payload() {
    assert_eq!(
        decode_rlp_list(&[0xc2, 0x80], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn rejects_long_list_missing_length_bytes() {
    assert_eq!(
        decode_rlp_list(&[0xf9, 0x01], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn rejects_long_list_missing_payload() {
    assert_eq!(
        decode_rlp_list(&[0xf8, 56, 0x80], DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::OffsetOutOfBounds)
    );
}

#[test]
fn rejects_long_list_with_leading_zero_length() {
    let mut input = vec![0xf9, 0, 56];
    input.extend_from_slice(&[0x80; 56]);

    assert_eq!(
        decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn rejects_long_list_length_overflow() {
    let input = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

    assert_eq!(
        decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::LengthOverflow)
    );
}

#[test]
fn rejects_long_list_used_for_short_payload() {
    let mut input = vec![0xf8, 55];
    input.extend_from_slice(&[0x80; 55]);

    assert_eq!(
        decode_rlp_list(&input, DecodeLimits::TEST_FIXTURE),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn enforces_list_item_budget() {
    let limits = DecodeLimits {
        max_list_items: 1,
        ..DecodeLimits::TEST_FIXTURE
    };

    assert_eq!(
        decode_rlp_list(&[0xc2, 0x80, 0x80], limits),
        Err(DecodeError::ListTooLong)
    );
}

#[test]
fn enforces_list_nesting_budget() {
    let limits = DecodeLimits {
        max_nesting_depth: 1,
        ..DecodeLimits::TEST_FIXTURE
    };

    assert_eq!(
        decode_rlp_list(&[0xc1, 0xc0], limits),
        Err(DecodeError::NestingTooDeep)
    );
}

#[test]
fn enforces_deep_list_nesting_budget() -> Result<(), DecodeError> {
    let input = nested_empty_list(65)?;
    let limits = DecodeLimits {
        max_input_bytes: input.len(),
        max_nesting_depth: 64,
        max_total_items: 128,
        ..DecodeLimits::TEST_FIXTURE
    };

    assert_eq!(
        decode_rlp_list(&input, limits),
        Err(DecodeError::NestingTooDeep)
    );
    Ok(())
}

#[test]
fn enforces_list_total_item_budget() {
    let limits = DecodeLimits {
        max_total_items: 1,
        ..DecodeLimits::TEST_FIXTURE
    };

    assert_eq!(
        decode_rlp_list(&[0xc1, 0x80], limits),
        Err(DecodeError::ItemCountExceeded)
    );
}

fn nested_empty_list(depth: usize) -> Result<Vec<u8>, DecodeError> {
    let mut encoded = vec![0xc0];
    for _ in 1..depth {
        encoded = encode_list(encoded)?;
    }
    Ok(encoded)
}

fn encode_list(payload: Vec<u8>) -> Result<Vec<u8>, DecodeError> {
    if payload.len() <= 55 {
        let payload_len = u8::try_from(payload.len()).map_err(|_| DecodeError::LengthOverflow)?;
        let prefix = 0xc0_u8
            .checked_add(payload_len)
            .ok_or(DecodeError::LengthOverflow)?;
        let mut encoded = vec![prefix];
        encoded.extend_from_slice(&payload);
        return Ok(encoded);
    }

    let len_bytes = minimal_be_len(payload.len())?;
    let len_of_len = u8::try_from(len_bytes.len()).map_err(|_| DecodeError::LengthOverflow)?;
    let prefix = 0xf7_u8
        .checked_add(len_of_len)
        .ok_or(DecodeError::LengthOverflow)?;
    let mut encoded = vec![prefix];
    encoded.extend_from_slice(&len_bytes);
    encoded.extend_from_slice(&payload);
    Ok(encoded)
}

fn minimal_be_len(mut len: usize) -> Result<Vec<u8>, DecodeError> {
    let mut out = Vec::new();
    while len > 0 {
        let byte = u8::try_from(len & 0xff).map_err(|_| DecodeError::LengthOverflow)?;
        out.push(byte);
        len >>= 8;
    }
    out.reverse();
    Ok(out)
}
