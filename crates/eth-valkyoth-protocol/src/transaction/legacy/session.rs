use eth_valkyoth_codec::{DecodeError, DecodeSession};

use super::*;
use crate::transaction::envelope::decode_transaction_envelope_in_session;
use crate::transaction::fields::fixed_fields_in_session;

/// Decodes a legacy transaction through one cumulative work session.
pub fn decode_legacy_transaction_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedLegacyTransaction<'a>, LegacyTransactionDecodeError> {
    match decode_transaction_envelope_in_session(input, session)
        .map_err(LegacyTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Legacy(list) => decode_list(list, session),
        TransactionEnvelope::Typed(typed) => Err(LegacyTransactionDecodeError::TypedEnvelope {
            type_byte: typed.transaction_type.get(),
        }),
    }
}

fn decode_list<'a>(
    list: RlpList<'a>,
    session: &mut DecodeSession,
) -> Result<UnvalidatedLegacyTransaction<'a>, LegacyTransactionDecodeError> {
    if list.item_count() != LEGACY_TRANSACTION_FIELD_COUNT {
        return Err(LegacyTransactionDecodeError::WrongFieldCount {
            expected: LEGACY_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }
    let decoded = fixed_fields_in_session::<LEGACY_TRANSACTION_FIELD_COUNT>(list, session)
        .map_err(|source| field_error(LegacyTransactionField::Nonce, source))?;
    let mut fields = decoded
        .into_iter()
        .map(|item| item.ok_or(DecodeError::Malformed));
    let nonce = Nonce::new(decode_u64_field(
        &mut fields,
        LegacyTransactionField::Nonce,
    )?);
    let gas_price = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        LegacyTransactionField::GasPrice,
    )?);
    let gas_limit = Gas::new(decode_u64_field(
        &mut fields,
        LegacyTransactionField::GasLimit,
    )?);
    let to = decode_to(next_scalar(&mut fields, LegacyTransactionField::To)?)?;
    let value = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        LegacyTransactionField::Value,
    )?);
    let input = next_scalar(&mut fields, LegacyTransactionField::Input)?.payload();
    session
        .limits()
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(LegacyTransactionField::Input, source))?;
    let v = decode_u256_field(&mut fields, LegacyTransactionField::V)?;
    let r = decode_u256_field(&mut fields, LegacyTransactionField::R)?;
    let s = decode_u256_field(&mut fields, LegacyTransactionField::S)?;
    Ok(UnvalidatedLegacyTransaction {
        nonce,
        gas_price,
        gas_limit,
        to,
        value,
        input,
        v,
        r,
        s,
    })
}
