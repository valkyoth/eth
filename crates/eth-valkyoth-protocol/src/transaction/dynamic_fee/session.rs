use eth_valkyoth_codec::{DecodeError, DecodeSession};

use super::*;
use crate::transaction::access_list::decode_access_list_in_session;
use crate::transaction::envelope::decode_transaction_envelope_in_session;
use crate::transaction::fields::fixed_fields_in_session;

/// Decodes an EIP-1559 transaction through one cumulative work session.
pub fn decode_dynamic_fee_transaction_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedDynamicFeeTransaction<'a>, DynamicFeeTransactionDecodeError> {
    match decode_transaction_envelope_in_session(input, session)
        .map_err(DynamicFeeTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == DYNAMIC_FEE_TRANSACTION_TYPE =>
        {
            decode_payload(typed.payload, session)
        }
        TransactionEnvelope::Typed(typed) => {
            Err(DynamicFeeTransactionDecodeError::WrongTransactionType {
                type_byte: typed.transaction_type.get(),
            })
        }
        TransactionEnvelope::Legacy(_) => {
            Err(DynamicFeeTransactionDecodeError::WrongTransactionType { type_byte: 0 })
        }
    }
}

fn decode_payload<'a>(
    payload: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedDynamicFeeTransaction<'a>, DynamicFeeTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list_in_session(payload, session)
        .map_err(|source| field_error(DynamicFeeTransactionField::Payload, source))?;
    if list.item_count() != DYNAMIC_FEE_TRANSACTION_FIELD_COUNT {
        return Err(DynamicFeeTransactionDecodeError::WrongFieldCount {
            expected: DYNAMIC_FEE_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }
    let decoded = fixed_fields_in_session::<DYNAMIC_FEE_TRANSACTION_FIELD_COUNT>(list, session)
        .map_err(|source| field_error(DynamicFeeTransactionField::Payload, source))?;
    let mut fields = decoded
        .into_iter()
        .map(|item| item.ok_or(DecodeError::Malformed));
    let chain_id = decode_shared_chain_id(
        &mut fields,
        DynamicFeeTransactionField::ChainId,
        field_error,
    )?;
    let nonce = Nonce::new(decode_shared_u64_field(
        &mut fields,
        DynamicFeeTransactionField::Nonce,
        field_error,
    )?);
    let max_priority_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        DynamicFeeTransactionField::MaxPriorityFeePerGas,
        field_error,
    )?);
    let max_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        DynamicFeeTransactionField::MaxFeePerGas,
        field_error,
    )?);
    let gas_limit = Gas::new(decode_shared_u64_field(
        &mut fields,
        DynamicFeeTransactionField::GasLimit,
        field_error,
    )?);
    let to = decode_shared_to(
        next_shared_scalar(&mut fields, DynamicFeeTransactionField::To, field_error)?,
        |found| DynamicFeeTransactionDecodeError::InvalidToLength { found },
    )?;
    let value = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        DynamicFeeTransactionField::Value,
        field_error,
    )?);
    let input =
        next_shared_scalar(&mut fields, DynamicFeeTransactionField::Data, field_error)?.payload();
    session
        .limits()
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(DynamicFeeTransactionField::Data, source))?;
    let access_list = decode_access_list_in_session(
        next_shared_list(
            &mut fields,
            DynamicFeeTransactionField::AccessList,
            field_error,
        )?,
        session,
    )
    .map_err(map_access_list_error)?;
    let y_parity = SignatureYParity::try_new(decode_shared_u64_field(
        &mut fields,
        DynamicFeeTransactionField::SignatureYParity,
        field_error,
    )?)
    .map_err(|error| DynamicFeeTransactionDecodeError::InvalidYParity {
        value: error.value(),
    })?;
    let r = decode_shared_u256_field(
        &mut fields,
        DynamicFeeTransactionField::SignatureR,
        field_error,
    )?;
    let s = decode_shared_u256_field(
        &mut fields,
        DynamicFeeTransactionField::SignatureS,
        field_error,
    )?;
    Ok(UnvalidatedDynamicFeeTransaction {
        chain_id,
        nonce,
        max_priority_fee_per_gas,
        max_fee_per_gas,
        gas_limit,
        to,
        value,
        input,
        access_list,
        y_parity,
        r,
        s,
    })
}
