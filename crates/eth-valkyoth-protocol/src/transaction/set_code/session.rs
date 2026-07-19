use eth_valkyoth_codec::{DecodeError, DecodeSession, RlpItem, RlpList};

use super::*;
use crate::transaction::access_list::decode_access_list_in_session;
use crate::transaction::envelope::decode_transaction_envelope_in_session;
use crate::transaction::fields::fixed_fields_in_session;

impl<'a> SetCodeAuthorizationList<'a> {
    /// Iterates authorizations while charging every borrowed-model reparse.
    pub fn authorizations_in_session<'s>(
        self,
        session: &'s mut DecodeSession,
    ) -> SetCodeAuthorizationSessionItems<'a, 's> {
        SetCodeAuthorizationSessionItems {
            inner: self.authorizations(),
            session,
        }
    }
}

impl<'a> SetCodeAuthorizationItems<'a> {
    /// Advances one authorization while borrowing the session for this step.
    pub fn next_in_session(
        &mut self,
        session: &mut DecodeSession,
    ) -> Option<Result<SetCodeAuthorization, SetCodeTransactionDecodeError>> {
        let item = self.items.next_in_session(session)?;
        Some(decode_authorization(item, session).map_err(map_authorization_error))
    }
}

/// Accounted iterator over borrowed EIP-7702 authorization tuples.
pub struct SetCodeAuthorizationSessionItems<'a, 's> {
    inner: SetCodeAuthorizationItems<'a>,
    session: &'s mut DecodeSession,
}

impl Iterator for SetCodeAuthorizationSessionItems<'_, '_> {
    type Item = Result<SetCodeAuthorization, SetCodeTransactionDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_in_session(self.session)
    }
}

impl core::iter::FusedIterator for SetCodeAuthorizationSessionItems<'_, '_> {}

/// Decodes an EIP-7702 transaction through one cumulative work session.
pub fn decode_set_code_transaction_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedSetCodeTransaction<'a>, SetCodeTransactionDecodeError> {
    match decode_transaction_envelope_in_session(input, session)
        .map_err(SetCodeTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == SET_CODE_TRANSACTION_TYPE =>
        {
            decode_payload(typed.payload, session)
        }
        TransactionEnvelope::Typed(typed) => {
            Err(SetCodeTransactionDecodeError::WrongTransactionType {
                type_byte: typed.transaction_type.get(),
            })
        }
        TransactionEnvelope::Legacy(_) => {
            Err(SetCodeTransactionDecodeError::WrongTransactionType { type_byte: 0 })
        }
    }
}

fn decode_payload<'a>(
    payload: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedSetCodeTransaction<'a>, SetCodeTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list_in_session(payload, session)
        .map_err(|source| field_error(SetCodeTransactionField::Payload, source))?;
    if list.item_count() != SET_CODE_TRANSACTION_FIELD_COUNT {
        return Err(SetCodeTransactionDecodeError::WrongFieldCount {
            expected: SET_CODE_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }
    let decoded = fixed_fields_in_session::<SET_CODE_TRANSACTION_FIELD_COUNT>(list, session)
        .map_err(|source| field_error(SetCodeTransactionField::Payload, source))?;
    let mut fields = decoded
        .into_iter()
        .map(|item| item.ok_or(DecodeError::Malformed));
    let chain_id =
        decode_shared_chain_id(&mut fields, SetCodeTransactionField::ChainId, field_error)?;
    let nonce = Nonce::new(decode_shared_u64_field(
        &mut fields,
        SetCodeTransactionField::Nonce,
        field_error,
    )?);
    let max_priority_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        SetCodeTransactionField::MaxPriorityFeePerGas,
        field_error,
    )?);
    let max_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        SetCodeTransactionField::MaxFeePerGas,
        field_error,
    )?);
    let gas_limit = Gas::new(decode_shared_u64_field(
        &mut fields,
        SetCodeTransactionField::GasLimit,
        field_error,
    )?);
    let to = decode_required_address(
        next_shared_scalar(&mut fields, SetCodeTransactionField::To, field_error)?,
        |found| SetCodeTransactionDecodeError::InvalidToLength { found },
    )?;
    let value = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        SetCodeTransactionField::Value,
        field_error,
    )?);
    let input =
        next_shared_scalar(&mut fields, SetCodeTransactionField::Data, field_error)?.payload();
    session
        .limits()
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(SetCodeTransactionField::Data, source))?;
    let access_list = decode_access_list_in_session(
        next_shared_list(
            &mut fields,
            SetCodeTransactionField::AccessList,
            field_error,
        )?,
        session,
    )
    .map_err(map_access_list_error)?;
    let authorization_list = decode_authorizations(
        next_shared_list(
            &mut fields,
            SetCodeTransactionField::AuthorizationList,
            field_error,
        )?,
        session,
    )?;
    let y_parity = SignatureYParity::try_new(decode_shared_u64_field(
        &mut fields,
        SetCodeTransactionField::SignatureYParity,
        field_error,
    )?)
    .map_err(|error| SetCodeTransactionDecodeError::InvalidYParity {
        value: error.value(),
    })?;
    let r = decode_shared_u256_field(
        &mut fields,
        SetCodeTransactionField::SignatureR,
        field_error,
    )?;
    let s = decode_shared_u256_field(
        &mut fields,
        SetCodeTransactionField::SignatureS,
        field_error,
    )?;
    Ok(UnvalidatedSetCodeTransaction {
        chain_id,
        nonce,
        max_priority_fee_per_gas,
        max_fee_per_gas,
        gas_limit,
        to,
        value,
        input,
        access_list,
        authorization_list,
        y_parity,
        r,
        s,
    })
}

fn decode_authorizations<'a>(
    list: RlpList<'a>,
    session: &mut DecodeSession,
) -> Result<SetCodeAuthorizationList<'a>, SetCodeTransactionDecodeError> {
    let mut items = list.items();
    while let Some(item) = items.next_in_session(session) {
        decode_authorization(item, session).map_err(map_authorization_error)?;
    }
    Ok(SetCodeAuthorizationList { list })
}

fn decode_authorization(
    item: Result<RlpItem<'_>, DecodeError>,
    session: &mut DecodeSession,
) -> Result<SetCodeAuthorization, SetCodeAuthorizationDecodeError> {
    let RlpItem::List(list) = item.map_err(SetCodeAuthorizationDecodeError::TupleDecode)? else {
        return Err(SetCodeAuthorizationDecodeError::TupleDecode(
            DecodeError::UnexpectedScalar,
        ));
    };
    if list.item_count() != SET_CODE_AUTHORIZATION_FIELD_COUNT {
        return Err(
            SetCodeAuthorizationDecodeError::InvalidAuthorizationFieldCount {
                found: list.item_count(),
            },
        );
    }
    let decoded = fixed_fields_in_session::<SET_CODE_AUTHORIZATION_FIELD_COUNT>(list, session)
        .map_err(SetCodeAuthorizationDecodeError::TupleDecode)?;
    let mut fields = decoded
        .into_iter()
        .map(|item| item.ok_or(DecodeError::Malformed));
    decode_authorization_fields(&mut fields)
}

pub(super) fn decode_authorization_fields<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
) -> Result<SetCodeAuthorization, SetCodeAuthorizationDecodeError> {
    let chain_id = decode_authorization_chain_id(next_shared_scalar(
        fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::ChainId, source),
    )?)?;
    let address = decode_authorization_address(next_shared_scalar(
        fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::Address, source),
    )?)?;
    let nonce = Nonce::new(decode_shared_u64_field(
        fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::Nonce, source),
    )?);
    let y_parity = SignatureYParity::try_new(decode_shared_u64_field(
        fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::YParity, source),
    )?)
    .map_err(
        |error| SetCodeAuthorizationDecodeError::InvalidAuthorizationYParity {
            value: error.value(),
        },
    )?;
    let r = decode_shared_u256_field(
        fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::R, source),
    )?;
    let s = decode_shared_u256_field(
        fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::S, source),
    )?;
    Ok(SetCodeAuthorization {
        chain_id,
        address,
        nonce,
        y_parity,
        r,
        s,
    })
}
