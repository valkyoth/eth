use super::*;

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
        Gas::try_from_canonical_be_slice(&[0x00, 0x01]),
        Err(PrimitiveError::NonCanonicalInteger)
    );
    assert_eq!(
        Nonce::try_from_canonical_be_slice(&[1_u8; 9]),
        Err(PrimitiveError::IntegerTooLarge)
    );
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
    let tx_type = TransactionType::try_new(TransactionType::MAX_TYPED);
    assert_eq!(tx_type.map(TransactionType::get), Ok(0x7f));
}

#[test]
fn transaction_type_rejects_reserved_range() {
    assert_eq!(
        TransactionType::try_new(0x80),
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
    assert_eq!(TransactionType::try_new(2).map(u8::from), Ok(2));
    assert_eq!(TransactionType::try_new_with_legacy(0).map(u8::from), Ok(0));
}
