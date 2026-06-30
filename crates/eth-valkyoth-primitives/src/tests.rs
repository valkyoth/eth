use super::*;
use eth_valkyoth_codec::{
    DecodeError, DecodeLimits, rlp_integer_payload_to_u64, rlp_integer_payload_to_u256_bytes,
};

#[test]
fn chain_id_round_trips() {
    assert_eq!(u64::from(ChainId::from(1)), 1);
}

#[test]
fn block_number_round_trips() {
    assert_eq!(u64::from(BlockNumber::from(2)), 2);
}

#[test]
fn gas_round_trips() {
    assert_eq!(u64::from(Gas::from(21_000)), 21_000);
}

#[test]
fn nonce_round_trips() {
    assert_eq!(u64::from(Nonce::from(7)), 7);
}

#[test]
fn unix_timestamp_round_trips() {
    assert_eq!(u64::from(UnixTimestamp::from(1_700_000_000)), 1_700_000_000);
}

#[test]
fn id_types_parse_canonical_rlp_integer_payloads() {
    assert_eq!(
        ChainId::try_from_canonical_be_slice(&[]),
        Ok(ChainId::new(0))
    );
    assert_eq!(
        ChainId::try_from_canonical_be_slice(&[0x04, 0x00]),
        Ok(ChainId::new(1024))
    );
    assert_eq!(
        ChainId::try_from_signed_canonical_be_slice(&[]),
        Err(PrimitiveError::ReservedLegacyType)
    );
    assert_eq!(
        ChainId::try_from_signed_canonical_be_slice(&[0x01]),
        Ok(ChainId::new(1))
    );
    assert_eq!(
        Gas::try_from_canonical_be_slice(&[0x00, 0x01]),
        Err(PrimitiveError::NonCanonicalInteger)
    );
    assert_eq!(
        Nonce::try_from_canonical_be_slice(&[1_u8; 9]),
        Err(PrimitiveError::IntegerTooLarge)
    );
}

#[test]
fn id_types_delegate_canonical_payload_parsing_to_codec() {
    let cases: &[&[u8]] = &[&[], &[0x01], &[0x7f], &[0x80], &[0x04, 0x00], &[1_u8; 8]];

    for bytes in cases {
        assert_eq!(
            ChainId::try_from_canonical_be_slice(bytes).map(ChainId::get),
            rlp_integer_payload_to_u64(bytes).map_err(map_rlp_integer_error)
        );
        assert_eq!(
            BlockNumber::try_from_canonical_be_slice(bytes).map(BlockNumber::get),
            rlp_integer_payload_to_u64(bytes).map_err(map_rlp_integer_error)
        );
        assert_eq!(
            Gas::try_from_canonical_be_slice(bytes).map(Gas::get),
            rlp_integer_payload_to_u64(bytes).map_err(map_rlp_integer_error)
        );
        assert_eq!(
            Nonce::try_from_canonical_be_slice(bytes).map(Nonce::get),
            rlp_integer_payload_to_u64(bytes).map_err(map_rlp_integer_error)
        );
        assert_eq!(
            UnixTimestamp::try_from_canonical_be_slice(bytes).map(UnixTimestamp::get),
            rlp_integer_payload_to_u64(bytes).map_err(map_rlp_integer_error)
        );
    }
}

#[test]
fn id_types_map_codec_payload_errors_to_primitive_errors() {
    let noncanonical: &[u8] = &[0x00, 0x01];
    let too_large: &[u8] = &[1_u8; 9];

    assert_eq!(
        rlp_integer_payload_to_u64(noncanonical),
        Err(eth_valkyoth_codec::DecodeError::Malformed)
    );
    assert_eq!(
        ChainId::try_from_canonical_be_slice(noncanonical),
        Err(PrimitiveError::NonCanonicalInteger)
    );

    assert_eq!(
        rlp_integer_payload_to_u64(too_large),
        Err(eth_valkyoth_codec::DecodeError::LengthOverflow)
    );
    assert_eq!(
        ChainId::try_from_canonical_be_slice(too_large),
        Err(PrimitiveError::IntegerTooLarge)
    );
}

#[test]
fn id_types_rlp_round_trip_canonical_integers() -> Result<(), PrimitiveRlpError> {
    let cases: &[(ChainId, &[u8])] = &[
        (ChainId::new(0), &[0x80]),
        (ChainId::new(1), &[0x01]),
        (ChainId::new(127), &[0x7f]),
        (ChainId::new(128), &[0x81, 0x80]),
        (ChainId::new(1024), &[0x82, 0x04, 0x00]),
    ];

    for (value, expected) in cases {
        let mut output = [0_u8; 16];
        assert_eq!(value.encoded_rlp_len()?, expected.len());
        let written = value.encode_rlp(&mut output)?;
        assert_eq!(output.get(..written), Some(*expected));
        assert_eq!(
            ChainId::try_from_rlp(expected, DecodeLimits::TEST_FIXTURE),
            Ok(*value)
        );
    }
    assert_eq!(
        ChainId::try_from_rlp_signed(&[0x80], DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::Primitive(
            PrimitiveError::ReservedLegacyType
        ))
    );
    assert_eq!(
        ChainId::try_from_rlp_signed(&[0x01], DecodeLimits::TEST_FIXTURE),
        Ok(ChainId::new(1))
    );

    assert_eq!(
        BlockNumber::try_from_rlp(&[0x82, 0x04, 0x00], DecodeLimits::TEST_FIXTURE),
        Ok(BlockNumber::new(1024))
    );
    assert_eq!(
        Gas::try_from_rlp(&[0x82, 0x52, 0x08], DecodeLimits::TEST_FIXTURE),
        Ok(Gas::new(21_000))
    );
    assert_eq!(
        Nonce::try_from_rlp(&[0x07], DecodeLimits::TEST_FIXTURE),
        Ok(Nonce::new(7))
    );
    assert_eq!(
        UnixTimestamp::try_from_rlp(&[0x84, 0x65, 0x53, 0xf1, 0x00], DecodeLimits::TEST_FIXTURE),
        Ok(UnixTimestamp::new(1_700_000_000))
    );
    Ok(())
}

#[test]
fn id_types_rlp_reject_malformed_inputs() {
    assert_eq!(
        ChainId::try_from_rlp(&[0x00], DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::Decode(DecodeError::Malformed))
    );
    assert_eq!(
        ChainId::try_from_rlp(&[0x01, 0x02], DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::Decode(DecodeError::TrailingBytes))
    );
    assert_eq!(
        ChainId::try_from_rlp(
            &[0x89, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            DecodeLimits::TEST_FIXTURE
        ),
        Err(PrimitiveRlpError::Decode(DecodeError::LengthOverflow))
    );
}

#[test]
fn rlp_encode_rejects_short_output_without_modifying_it() {
    let mut output = [0xaa_u8; 1];

    assert_eq!(
        ChainId::new(1024).encode_rlp(&mut output),
        Err(PrimitiveRlpError::Decode(DecodeError::OffsetOutOfBounds))
    );
    assert_eq!(output, [0xaa]);
}

#[test]
fn address_round_trips() {
    let bytes = [7_u8; 20];
    assert_eq!(<[u8; 20]>::from(Address::from(bytes)), bytes);
}

#[test]
fn address_constant_time_equality_result_matches_equality() {
    let left = Address::from_bytes([1_u8; 20]);
    let same = Address::from_bytes([1_u8; 20]);
    let different = Address::from_bytes([2_u8; 20]);

    assert!(bool::from(left.ct_eq(&same)));
    assert!(!bool::from(left.ct_eq(&different)));
    assert!(left == same);
    assert!(left != different);
}

#[test]
fn b256_constant_time_equality_result_matches_equality() {
    let left = B256::from_bytes([1_u8; 32]);
    let same = B256::from_bytes([1_u8; 32]);
    let different = B256::from_bytes([2_u8; 32]);
    assert!(bool::from(left.ct_eq(&same)));
    assert!(!bool::from(left.ct_eq(&different)));
    assert!(left == same);
    assert!(left != different);
}

#[test]
fn b256_constant_time_choices_compose_without_short_circuit() {
    let left = B256::from_bytes([1_u8; 32]);
    let same = B256::from_bytes([1_u8; 32]);
    let different = B256::from_bytes([2_u8; 32]);
    let composed = left.ct_eq(&same) & same.ct_eq(&different);

    assert!(!bool::from(composed));
}

#[test]
fn b256_round_trips() {
    let bytes = [3_u8; 32];
    assert_eq!(<[u8; 32]>::from(B256::from(bytes)), bytes);
}

#[test]
fn address_and_hash_rlp_round_trip_fixed_width_scalars() -> Result<(), PrimitiveRlpError> {
    let address = Address::from_bytes([0x11_u8; 20]);
    let hash = B256::from_bytes([0x22_u8; 32]);
    let mut address_output = [0_u8; 21];
    let mut hash_output = [0_u8; 33];

    assert_eq!(address.encoded_rlp_len()?, 21);
    let address_written = address.encode_rlp(&mut address_output)?;
    assert_eq!(address_written, 21);
    assert_eq!(address_output.first(), Some(&0x94));
    assert_eq!(
        Address::try_from_rlp(&address_output, DecodeLimits::TEST_FIXTURE),
        Ok(address)
    );

    assert_eq!(hash.encoded_rlp_len()?, 33);
    let hash_written = hash.encode_rlp(&mut hash_output)?;
    assert_eq!(hash_written, 33);
    assert_eq!(hash_output.first(), Some(&0xa0));
    assert_eq!(
        B256::try_from_rlp(&hash_output, DecodeLimits::TEST_FIXTURE),
        Ok(hash)
    );
    Ok(())
}

#[test]
fn fixed_width_rlp_rejects_wrong_scalar_lengths() {
    assert_eq!(
        Address::try_from_rlp(&[0x80], DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::FixedWidthScalar {
            expected: 20,
            found: 0,
        })
    );
    assert_eq!(
        B256::try_from_rlp(&[0x80], DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::FixedWidthScalar {
            expected: 32,
            found: 0,
        })
    );
}

#[test]
fn fixed_width_rlp_rejects_adjacent_scalar_lengths() {
    let mut address_too_long = [0x11_u8; 22];
    address_too_long[0] = 0x95;
    assert_eq!(
        Address::try_from_rlp(&address_too_long, DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::FixedWidthScalar {
            expected: 20,
            found: 21,
        })
    );

    let mut hash_too_short = [0x22_u8; 32];
    hash_too_short[0] = 0x9f;
    assert_eq!(
        B256::try_from_rlp(&hash_too_short, DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::FixedWidthScalar {
            expected: 32,
            found: 31,
        })
    );

    let mut hash_too_long = [0x22_u8; 34];
    hash_too_long[0] = 0xa1;
    assert_eq!(
        B256::try_from_rlp(&hash_too_long, DecodeLimits::TEST_FIXTURE),
        Err(PrimitiveRlpError::FixedWidthScalar {
            expected: 32,
            found: 33,
        })
    );
}

#[test]
fn wei_round_trips() {
    let bytes = [9_u8; 32];
    assert_eq!(<[u8; 32]>::from(Wei::from(bytes)), bytes);
}

#[test]
fn wei_parses_canonical_rlp_integer_payloads() {
    assert_eq!(Wei::try_from_canonical_be_slice(&[]), Ok(Wei::ZERO));
    assert_eq!(
        Wei::try_from_canonical_be_slice(&[0x04, 0x00]).map(Wei::to_be_bytes),
        Ok(Wei::from_u128(1024).to_be_bytes())
    );
    assert_eq!(
        Wei::try_from_canonical_be_slice(&[0x00]),
        Err(PrimitiveError::NonCanonicalInteger)
    );
    assert_eq!(
        Wei::try_from_canonical_be_slice(&[1_u8; 33]),
        Err(PrimitiveError::IntegerTooLarge)
    );
}

#[test]
fn wei_parses_canonical_rlp_integer_at_max_width() {
    let max_bytes = [0xff_u8; 32];

    assert_eq!(
        Wei::try_from_canonical_be_slice(&max_bytes).map(Wei::to_be_bytes),
        Ok(max_bytes)
    );
}

#[test]
fn wei_delegates_canonical_payload_parsing_to_codec() {
    let cases: &[&[u8]] = &[
        &[],
        &[0x01],
        &[0x7f],
        &[0x80],
        &[0x04, 0x00],
        &[0xff_u8; 32],
    ];

    for bytes in cases {
        assert_eq!(
            Wei::try_from_canonical_be_slice(bytes).map(Wei::to_be_bytes),
            rlp_integer_payload_to_u256_bytes(bytes).map_err(map_rlp_integer_error)
        );
    }

    assert_eq!(
        Wei::try_from_canonical_be_slice(&[0x00, 0x01]),
        Err(PrimitiveError::NonCanonicalInteger)
    );
    assert_eq!(
        Wei::try_from_canonical_be_slice(&[1_u8; 33]),
        Err(PrimitiveError::IntegerTooLarge)
    );
}

#[test]
fn wei_rlp_round_trips_canonical_u256() -> Result<(), PrimitiveRlpError> {
    let cases: &[(Wei, &[u8])] = &[
        (Wei::ZERO, &[0x80]),
        (Wei::from_u128(1), &[0x01]),
        (Wei::from_u128(1024), &[0x82, 0x04, 0x00]),
    ];
    let mut output = [0_u8; 64];

    for (wei, expected) in cases {
        assert_eq!(wei.encoded_rlp_len()?, expected.len());
        let written = wei.encode_rlp(&mut output)?;
        assert_eq!(output.get(..written), Some(*expected));
        assert_eq!(
            Wei::try_from_rlp(expected, DecodeLimits::TEST_FIXTURE),
            Ok(*wei)
        );
    }

    let max = Wei::from_be_bytes([0xff_u8; 32]);
    assert_eq!(max.encoded_rlp_len()?, 33);
    let written = max.encode_rlp(&mut output)?;
    assert_eq!(written, 33);
    assert_eq!(output.first(), Some(&0xa0));
    let encoded = output
        .get(..written)
        .ok_or(PrimitiveRlpError::Decode(DecodeError::OffsetOutOfBounds))?;
    assert_eq!(
        Wei::try_from_rlp(encoded, DecodeLimits::TEST_FIXTURE),
        Ok(max)
    );
    Ok(())
}

#[test]
fn wei_constant_time_equality_result_matches_equality() {
    let left = Wei::from_be_bytes([1_u8; 32]);
    let same = Wei::from_be_bytes([1_u8; 32]);
    let different = Wei::from_be_bytes([2_u8; 32]);

    assert!(bool::from(left.ct_eq(&same)));
    assert!(!bool::from(left.ct_eq(&different)));
    assert!(left == same);
    assert!(left != different);
}

#[test]
fn wei_from_u128_places_bytes_at_low_end() {
    let wei = Wei::from_u128(1);
    let expected = [
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 1_u8,
    ];
    assert_eq!(wei.to_be_bytes(), expected);
}

#[test]
fn transaction_type_accepts_eip_2718_range() {
    let tx_type = TransactionType::try_new_with_legacy(TransactionType::MAX_TYPED);
    assert_eq!(tx_type.map(TransactionType::get), Ok(0x7f));
}

#[test]
fn transaction_type_rejects_reserved_range() {
    assert_eq!(
        TransactionType::try_new_with_legacy(0x80),
        Err(PrimitiveError::TransactionTypeTooLarge)
    );
}

#[test]
fn typed_transaction_type_rejects_legacy_zero() {
    assert_eq!(
        TransactionType::try_new_typed(0),
        Err(PrimitiveError::ReservedLegacyType)
    );
    assert_eq!(TransactionType::try_new_typed(1).map(u8::from), Ok(1));
    assert_eq!(
        TransactionType::try_from(0),
        Err(PrimitiveError::ReservedLegacyType)
    );
}

#[test]
fn transaction_type_round_trips() {
    assert_eq!(TransactionType::try_new_with_legacy(2).map(u8::from), Ok(2));
    assert_eq!(TransactionType::try_new_with_legacy(0).map(u8::from), Ok(0));
}
