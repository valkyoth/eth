use super::*;

pub(super) fn legacy_payload_len(
    tx: &UnvalidatedLegacyTransaction<'_>,
) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.gas_price.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_legacy_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        encoded_u256_len(tx.v)?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

pub(super) fn access_list_payload_len(
    tx: &UnvalidatedAccessListTransaction<'_>,
) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.gas_price.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_access_list_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        encoded_u64_len(u64::from(tx.y_parity.get()))?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

pub(super) fn dynamic_fee_payload_len(
    tx: &UnvalidatedDynamicFeeTransaction<'_>,
) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_access_list_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        encoded_u64_len(u64::from(tx.y_parity.get()))?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

pub(super) fn blob_payload_len(tx: &UnvalidatedBlobTransaction<'_>) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_rlp_scalar_len(&tx.to.to_bytes())?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        encoded_u256_len(tx.max_fee_per_blob_gas.to_be_bytes())?,
        tx.blob_versioned_hashes.encoded_rlp_len(),
        encoded_u64_len(u64::from(tx.y_parity.get()))?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

pub(super) fn set_code_payload_len(
    tx: &UnvalidatedSetCodeTransaction<'_>,
) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_rlp_scalar_len(&tx.to.to_bytes())?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        tx.authorization_list.encoded_rlp_len(),
        encoded_u64_len(u64::from(tx.y_parity.get()))?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}
