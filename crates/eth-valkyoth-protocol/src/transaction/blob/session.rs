use eth_valkyoth_codec::{DecodeError, DecodeSession};

use super::*;
use crate::transaction::access_list::decode_access_list_in_session;
use crate::transaction::envelope::decode_transaction_envelope_in_session;
use crate::transaction::fields::fixed_fields_in_session;

/// Decodes an EIP-4844 transaction through one cumulative work session.
pub fn decode_blob_transaction_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedBlobTransaction<'a>, BlobTransactionDecodeError> {
    match decode_transaction_envelope_in_session(input, session)
        .map_err(BlobTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == BLOB_TRANSACTION_TYPE =>
        {
            decode_payload(typed.payload, session)
        }
        TransactionEnvelope::Typed(typed) => {
            Err(BlobTransactionDecodeError::WrongTransactionType {
                type_byte: typed.transaction_type.get(),
            })
        }
        TransactionEnvelope::Legacy(_) => {
            Err(BlobTransactionDecodeError::WrongTransactionType { type_byte: 0 })
        }
    }
}

fn decode_payload<'a>(
    payload: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedBlobTransaction<'a>, BlobTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list_in_session(payload, session)
        .map_err(|source| field_error(BlobTransactionField::Payload, source))?;
    if list.item_count() != BLOB_TRANSACTION_FIELD_COUNT {
        return Err(BlobTransactionDecodeError::WrongFieldCount {
            expected: BLOB_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }
    let decoded = fixed_fields_in_session::<BLOB_TRANSACTION_FIELD_COUNT>(list, session)
        .map_err(|source| field_error(BlobTransactionField::Payload, source))?;
    let mut fields = decoded
        .into_iter()
        .map(|item| item.ok_or(DecodeError::Malformed));
    let chain_id = decode_shared_chain_id(&mut fields, BlobTransactionField::ChainId, field_error)?;
    let nonce = Nonce::new(decode_shared_u64_field(
        &mut fields,
        BlobTransactionField::Nonce,
        field_error,
    )?);
    let max_priority_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::MaxPriorityFeePerGas,
        field_error,
    )?);
    let max_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::MaxFeePerGas,
        field_error,
    )?);
    let gas_limit = Gas::new(decode_shared_u64_field(
        &mut fields,
        BlobTransactionField::GasLimit,
        field_error,
    )?);
    let to = decode_required_address(
        next_shared_scalar(&mut fields, BlobTransactionField::To, field_error)?,
        |found| BlobTransactionDecodeError::InvalidToLength { found },
    )?;
    let value = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::Value,
        field_error,
    )?);
    let input = next_shared_scalar(&mut fields, BlobTransactionField::Data, field_error)?.payload();
    session
        .limits()
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(BlobTransactionField::Data, source))?;
    let access_list = decode_access_list_in_session(
        next_shared_list(&mut fields, BlobTransactionField::AccessList, field_error)?,
        session,
    )
    .map_err(map_access_list_error)?;
    let max_fee_per_blob_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::MaxFeePerBlobGas,
        field_error,
    )?);
    let blob_versioned_hashes = decode_hashes(
        next_shared_list(
            &mut fields,
            BlobTransactionField::BlobVersionedHashes,
            field_error,
        )?,
        session,
    )?;
    let y_parity = SignatureYParity::try_new(decode_shared_u64_field(
        &mut fields,
        BlobTransactionField::SignatureYParity,
        field_error,
    )?)
    .map_err(|error| BlobTransactionDecodeError::InvalidYParity {
        value: error.value(),
    })?;
    let r = decode_shared_u256_field(&mut fields, BlobTransactionField::SignatureR, field_error)?;
    let s = decode_shared_u256_field(&mut fields, BlobTransactionField::SignatureS, field_error)?;
    Ok(UnvalidatedBlobTransaction {
        chain_id,
        nonce,
        max_priority_fee_per_gas,
        max_fee_per_gas,
        gas_limit,
        to,
        value,
        input,
        access_list,
        max_fee_per_blob_gas,
        blob_versioned_hashes,
        y_parity,
        r,
        s,
    })
}

fn decode_hashes<'a>(
    list: RlpList<'a>,
    session: &mut DecodeSession,
) -> Result<BlobVersionedHashes<'a>, BlobTransactionDecodeError> {
    let mut items = list.items();
    while let Some(item) = items.next_in_session(session) {
        let _ = decode_blob_versioned_hash_item(item)?;
    }
    Ok(BlobVersionedHashes { list })
}
