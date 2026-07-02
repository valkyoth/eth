//! Canonical transaction signing preimage encoding.

use eth_valkyoth_primitives::ChainId;

use super::encode::{
    TransactionEncodeError, encode_list_envelope, encode_typed_envelope,
    encoded_access_list_to_len, encoded_legacy_to_len, encoded_list_envelope_len,
    encoded_typed_envelope_len, encoded_u64_len, encoded_u256_len, sum_lengths, write_access_list,
    write_address, write_authorization_list, write_blob_hashes, write_legacy_to, write_scalar,
    write_transaction_to, write_u64, write_u256, write_wei,
};
use super::{
    ACCESS_LIST_TRANSACTION_TYPE, BLOB_TRANSACTION_TYPE, DYNAMIC_FEE_TRANSACTION_TYPE,
    SET_CODE_TRANSACTION_TYPE, SetCodeAuthorization, UnvalidatedAccessListTransaction,
    UnvalidatedBlobTransaction, UnvalidatedDynamicFeeTransaction, UnvalidatedLegacyTransaction,
    UnvalidatedSetCodeTransaction,
};

/// EIP-7702 authorization signing magic byte.
pub const SET_CODE_AUTHORIZATION_MAGIC: u8 = 0x05;

/// Returns the EIP-155 legacy signing preimage length.
pub fn encoded_legacy_eip155_signing_preimage_len(
    transaction: &UnvalidatedLegacyTransaction<'_>,
    chain_id: ChainId,
) -> Result<usize, TransactionEncodeError> {
    encoded_list_envelope_len(legacy_eip155_payload_len(transaction, chain_id)?).map_err(Into::into)
}

/// Encodes the EIP-155 legacy signing preimage.
pub fn encode_legacy_eip155_signing_preimage(
    transaction: &UnvalidatedLegacyTransaction<'_>,
    chain_id: ChainId,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = legacy_eip155_payload_len(transaction, chain_id)?;
    encode_list_envelope(payload_len, output, |fields| {
        write_u64(transaction.nonce.get(), fields)?;
        write_wei(transaction.gas_price.to_be_bytes(), fields)?;
        write_u64(transaction.gas_limit.get(), fields)?;
        write_legacy_to(transaction.to, fields)?;
        write_wei(transaction.value.to_be_bytes(), fields)?;
        write_scalar(transaction.input, fields)?;
        write_u64(chain_id.get(), fields)?;
        write_u64(0, fields)?;
        write_u64(0, fields)
    })
    .map_err(Into::into)
}

/// Returns the EIP-2930 signing preimage length.
pub fn encoded_access_list_signing_preimage_len(
    transaction: &UnvalidatedAccessListTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_typed_envelope_len(access_list_signing_payload_len(transaction)?).map_err(Into::into)
}

/// Encodes the EIP-2930 signing preimage.
pub fn encode_access_list_signing_preimage(
    transaction: &UnvalidatedAccessListTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = access_list_signing_payload_len(transaction)?;
    encode_typed_envelope(
        ACCESS_LIST_TRANSACTION_TYPE,
        payload_len,
        output,
        |fields| {
            write_u64(transaction.chain_id.get(), fields)?;
            write_u64(transaction.nonce.get(), fields)?;
            write_wei(transaction.gas_price.to_be_bytes(), fields)?;
            write_u64(transaction.gas_limit.get(), fields)?;
            write_transaction_to(transaction.to, fields)?;
            write_wei(transaction.value.to_be_bytes(), fields)?;
            write_scalar(transaction.input, fields)?;
            write_access_list(transaction.access_list, fields)
        },
    )
    .map_err(Into::into)
}

/// Returns the EIP-1559 signing preimage length.
pub fn encoded_dynamic_fee_signing_preimage_len(
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_typed_envelope_len(dynamic_fee_signing_payload_len(transaction)?).map_err(Into::into)
}

/// Encodes the EIP-1559 signing preimage.
pub fn encode_dynamic_fee_signing_preimage(
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = dynamic_fee_signing_payload_len(transaction)?;
    encode_typed_envelope(
        DYNAMIC_FEE_TRANSACTION_TYPE,
        payload_len,
        output,
        |fields| {
            write_u64(transaction.chain_id.get(), fields)?;
            write_u64(transaction.nonce.get(), fields)?;
            write_wei(transaction.max_priority_fee_per_gas.to_be_bytes(), fields)?;
            write_wei(transaction.max_fee_per_gas.to_be_bytes(), fields)?;
            write_u64(transaction.gas_limit.get(), fields)?;
            write_transaction_to(transaction.to, fields)?;
            write_wei(transaction.value.to_be_bytes(), fields)?;
            write_scalar(transaction.input, fields)?;
            write_access_list(transaction.access_list, fields)
        },
    )
    .map_err(Into::into)
}

/// Returns the EIP-4844 signing preimage length.
pub fn encoded_blob_signing_preimage_len(
    transaction: &UnvalidatedBlobTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_typed_envelope_len(blob_signing_payload_len(transaction)?).map_err(Into::into)
}

/// Encodes the EIP-4844 signing preimage.
pub fn encode_blob_signing_preimage(
    transaction: &UnvalidatedBlobTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = blob_signing_payload_len(transaction)?;
    encode_typed_envelope(BLOB_TRANSACTION_TYPE, payload_len, output, |fields| {
        write_u64(transaction.chain_id.get(), fields)?;
        write_u64(transaction.nonce.get(), fields)?;
        write_wei(transaction.max_priority_fee_per_gas.to_be_bytes(), fields)?;
        write_wei(transaction.max_fee_per_gas.to_be_bytes(), fields)?;
        write_u64(transaction.gas_limit.get(), fields)?;
        write_scalar(&transaction.to.to_bytes(), fields)?;
        write_wei(transaction.value.to_be_bytes(), fields)?;
        write_scalar(transaction.input, fields)?;
        write_access_list(transaction.access_list, fields)?;
        write_wei(transaction.max_fee_per_blob_gas.to_be_bytes(), fields)?;
        write_blob_hashes(transaction.blob_versioned_hashes, fields)
    })
    .map_err(Into::into)
}

/// Returns the EIP-7702 set-code transaction signing preimage length.
pub fn encoded_set_code_signing_preimage_len(
    transaction: &UnvalidatedSetCodeTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_typed_envelope_len(set_code_signing_payload_len(transaction)?).map_err(Into::into)
}

/// Encodes the EIP-7702 set-code transaction signing preimage.
pub fn encode_set_code_signing_preimage(
    transaction: &UnvalidatedSetCodeTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = set_code_signing_payload_len(transaction)?;
    encode_typed_envelope(SET_CODE_TRANSACTION_TYPE, payload_len, output, |fields| {
        write_u64(transaction.chain_id.get(), fields)?;
        write_u64(transaction.nonce.get(), fields)?;
        write_wei(transaction.max_priority_fee_per_gas.to_be_bytes(), fields)?;
        write_wei(transaction.max_fee_per_gas.to_be_bytes(), fields)?;
        write_u64(transaction.gas_limit.get(), fields)?;
        write_address(transaction.to, fields)?;
        write_wei(transaction.value.to_be_bytes(), fields)?;
        write_scalar(transaction.input, fields)?;
        write_access_list(transaction.access_list, fields)?;
        write_authorization_list(transaction.authorization_list, fields)
    })
    .map_err(Into::into)
}

/// Returns the EIP-7702 authorization signing preimage length.
pub fn encoded_set_code_authorization_signing_preimage_len(
    authorization: SetCodeAuthorization,
) -> Result<usize, TransactionEncodeError> {
    eth_valkyoth_codec::checked_len_add(
        1,
        encoded_list_envelope_len(set_code_authorization_signing_payload_len(authorization)?)?,
    )
    .map_err(Into::into)
}

/// Encodes the EIP-7702 authorization signing preimage.
pub fn encode_set_code_authorization_signing_preimage(
    authorization: SetCodeAuthorization,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = set_code_authorization_signing_payload_len(authorization)?;
    let total_len = encoded_set_code_authorization_signing_preimage_len(authorization)?;
    let payload = output
        .get_mut(1..total_len)
        .ok_or(eth_valkyoth_codec::DecodeError::OffsetOutOfBounds)?;
    encode_list_envelope(payload_len, payload, |fields| {
        write_u256(authorization.chain_id.to_be_bytes(), fields)?;
        write_address(authorization.address, fields)?;
        write_u64(authorization.nonce.get(), fields)
    })?;
    let magic = output
        .first_mut()
        .ok_or(eth_valkyoth_codec::DecodeError::OffsetOutOfBounds)?;
    *magic = SET_CODE_AUTHORIZATION_MAGIC;
    Ok(total_len)
}

fn legacy_eip155_payload_len(
    tx: &UnvalidatedLegacyTransaction<'_>,
    chain_id: ChainId,
) -> Result<usize, TransactionEncodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.gas_price.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_legacy_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(tx.input)?,
        encoded_u64_len(chain_id.get())?,
        encoded_u64_len(0)?,
        encoded_u64_len(0)?,
    ])
    .map_err(Into::into)
}

fn access_list_signing_payload_len(
    tx: &UnvalidatedAccessListTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.gas_price.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_access_list_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
    ])
    .map_err(Into::into)
}

fn dynamic_fee_signing_payload_len(
    tx: &UnvalidatedDynamicFeeTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_access_list_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
    ])
    .map_err(Into::into)
}

fn blob_signing_payload_len(
    tx: &UnvalidatedBlobTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(&tx.to.to_bytes())?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        encoded_u256_len(tx.max_fee_per_blob_gas.to_be_bytes())?,
        tx.blob_versioned_hashes.encoded_rlp_len(),
    ])
    .map_err(Into::into)
}

fn set_code_signing_payload_len(
    tx: &UnvalidatedSetCodeTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(&tx.to.to_bytes())?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        tx.authorization_list.encoded_rlp_len(),
    ])
    .map_err(Into::into)
}

fn set_code_authorization_signing_payload_len(
    authorization: SetCodeAuthorization,
) -> Result<usize, TransactionEncodeError> {
    sum_lengths(&[
        encoded_u256_len(authorization.chain_id.to_be_bytes())?,
        eth_valkyoth_codec::encoded_rlp_scalar_len(&authorization.address.to_bytes())?,
        encoded_u64_len(authorization.nonce.get())?,
    ])
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth_valkyoth_codec::DecodeLimits;
    use eth_valkyoth_primitives::{Address, B256, Gas, Nonce, Wei};

    const TEST_LIMITS: DecodeLimits = DecodeLimits {
        max_input_bytes: 128,
        max_list_items: 16,
        max_nesting_depth: 8,
        max_total_allocation: 128,
        max_proof_nodes: 4,
        max_total_items: 32,
    };
    const EIP155_SIGNING_DATA: &[u8] = &[
        0xec, 0x09, 0x85, 0x04, 0xa8, 0x17, 0xc8, 0x00, 0x82, 0x52, 0x08, 0x94, 0x35, 0x35, 0x35,
        0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35,
        0x35, 0x35, 0x88, 0x0d, 0xe0, 0xb6, 0xb3, 0xa7, 0x64, 0x00, 0x00, 0x80, 0x01, 0x80, 0x80,
    ];

    #[test]
    fn encodes_eip155_published_signing_data() {
        let tx = UnvalidatedLegacyTransaction {
            nonce: Nonce::new(9),
            gas_price: Wei::from_u128(20_000_000_000),
            gas_limit: Gas::new(21_000),
            to: super::super::LegacyTransactionTo::Call(Address::from_bytes([0x35; 20])),
            value: Wei::from_u128(1_000_000_000_000_000_000),
            input: &[],
            v: [0_u8; 32],
            r: [0_u8; 32],
            s: [0_u8; 32],
        };
        let mut output = [0_u8; 64];

        assert_eq!(
            encoded_legacy_eip155_signing_preimage_len(&tx, ChainId::new(1)),
            Ok(EIP155_SIGNING_DATA.len())
        );
        let written = encode_legacy_eip155_signing_preimage(&tx, ChainId::new(1), &mut output);
        assert_eq!(written, Ok(EIP155_SIGNING_DATA.len()));
        assert_eq!(
            output.get(..EIP155_SIGNING_DATA.len()),
            Some(EIP155_SIGNING_DATA)
        );
    }

    #[test]
    fn typed_preimages_drop_signature_fields() {
        let access_list_tx = [
            0x01, 0xcd, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01,
            0x02,
        ];
        let tx = super::super::decode_access_list_transaction(&access_list_tx, TEST_LIMITS);
        assert!(tx.is_ok());
        if let Ok(tx) = tx {
            let mut output = [0_u8; 32];
            let expected = [
                0x01, 0xca, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0,
            ];
            let written = encode_access_list_signing_preimage(&tx, &mut output);
            assert_eq!(written, Ok(expected.len()));
            assert_eq!(output.get(..expected.len()), Some(expected.as_slice()));
        }
    }

    #[test]
    fn dynamic_and_blob_preimages_include_type_byte() {
        let dynamic_fee_tx = [
            0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01,
            0x01, 0x02,
        ];
        let blob_tx = [
            0x03, 0xf8, 0x45, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x05, 0x80, 0xc0, 0x06, 0xe1, 0xa0, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x02,
        ];
        let dynamic = super::super::decode_dynamic_fee_transaction(&dynamic_fee_tx, TEST_LIMITS);
        let blob = super::super::decode_blob_transaction(&blob_tx, TEST_LIMITS);
        assert!(dynamic.is_ok());
        assert!(blob.is_ok());
        if let (Ok(dynamic), Ok(blob)) = (dynamic, blob) {
            let mut dynamic_output = [0_u8; 32];
            let mut blob_output = [0_u8; 96];
            let dynamic_written =
                encode_dynamic_fee_signing_preimage(&dynamic, &mut dynamic_output);
            let blob_written = encode_blob_signing_preimage(&blob, &mut blob_output);

            assert_eq!(dynamic_written, Ok(13));
            assert_eq!(blob_written, Ok(69));
            assert_eq!(dynamic_output.first(), Some(&DYNAMIC_FEE_TRANSACTION_TYPE));
            assert_eq!(blob_output.first(), Some(&BLOB_TRANSACTION_TYPE));
            assert_ne!(
                dynamic_output.get(1..dynamic_written.unwrap_or(1)),
                dynamic_fee_tx.get(1..dynamic_fee_tx.len())
            );
            assert_eq!(blob.blob_versioned_hashes.hashes().count(), 1);
            assert_eq!(B256::from_bytes([1_u8; 32]).to_bytes()[0], 1);
        }
    }
}
