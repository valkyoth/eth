use eth_valkyoth_codec::{DecodeError, DecodeSession, RlpItem, RlpList};
use eth_valkyoth_primitives::{Gas, Nonce, Wei};

use super::*;
use crate::transaction::envelope::decode_transaction_envelope_in_session;
use crate::transaction::fields::{
    decode_chain_id as decode_shared_chain_id, decode_u64_field as decode_shared_u64_field,
    decode_u256_field as decode_shared_u256_field, fixed_fields_in_session,
    next_list as next_shared_list, next_scalar as next_shared_scalar,
};

impl<'a> AccessList<'a> {
    /// Iterates entries while charging every borrowed-model reparse.
    pub fn entries_in_session<'s>(
        self,
        session: &'s mut DecodeSession,
    ) -> AccessListSessionEntries<'a, 's> {
        AccessListSessionEntries {
            inner: self.entries(),
            session,
        }
    }
}

impl<'a> AccessListEntries<'a> {
    /// Advances one entry while borrowing the session only for this step.
    pub fn next_in_session(
        &mut self,
        session: &mut DecodeSession,
    ) -> Option<Result<AccessListEntry<'a>, AccessListTransactionDecodeError>> {
        let item = self.items.next_in_session(session)?;
        Some(decode_entry(item, session).map_err(map_access_list_decode_error))
    }
}

/// Accounted iterator over borrowed access-list entries.
pub struct AccessListSessionEntries<'a, 's> {
    inner: AccessListEntries<'a>,
    session: &'s mut DecodeSession,
}

impl<'a> Iterator for AccessListSessionEntries<'a, '_> {
    type Item = Result<AccessListEntry<'a>, AccessListTransactionDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_in_session(self.session)
    }
}

impl core::iter::FusedIterator for AccessListSessionEntries<'_, '_> {}

impl<'a> AccessListStorageKeys<'a> {
    /// Iterates storage keys while charging every borrowed-model reparse.
    pub fn keys_in_session<'s>(
        self,
        session: &'s mut DecodeSession,
    ) -> AccessListStorageKeysSessionItems<'a, 's> {
        AccessListStorageKeysSessionItems {
            inner: self.keys(),
            session,
        }
    }
}

impl<'a> AccessListStorageKeyItems<'a> {
    /// Advances one key while borrowing the session only for this step.
    pub fn next_in_session(
        &mut self,
        session: &mut DecodeSession,
    ) -> Option<Result<B256, AccessListTransactionDecodeError>> {
        let item = self.items.next_in_session(session)?;
        Some(decode_storage_key_item(item).map_err(map_access_list_decode_error))
    }
}

/// Accounted iterator over borrowed access-list storage keys.
pub struct AccessListStorageKeysSessionItems<'a, 's> {
    inner: AccessListStorageKeyItems<'a>,
    session: &'s mut DecodeSession,
}

impl Iterator for AccessListStorageKeysSessionItems<'_, '_> {
    type Item = Result<B256, AccessListTransactionDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_in_session(self.session)
    }
}

impl core::iter::FusedIterator for AccessListStorageKeysSessionItems<'_, '_> {}

/// Decodes an EIP-2930 transaction through one cumulative work session.
pub fn decode_access_list_transaction_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedAccessListTransaction<'a>, AccessListTransactionDecodeError> {
    match decode_transaction_envelope_in_session(input, session)
        .map_err(AccessListTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == ACCESS_LIST_TRANSACTION_TYPE =>
        {
            decode_payload(typed.payload, session)
        }
        TransactionEnvelope::Typed(typed) => {
            Err(AccessListTransactionDecodeError::WrongTransactionType {
                type_byte: typed.transaction_type.get(),
            })
        }
        TransactionEnvelope::Legacy(_) => {
            Err(AccessListTransactionDecodeError::WrongTransactionType { type_byte: 0 })
        }
    }
}

fn decode_payload<'a>(
    payload: &'a [u8],
    session: &mut DecodeSession,
) -> Result<UnvalidatedAccessListTransaction<'a>, AccessListTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list_in_session(payload, session)
        .map_err(|source| field_error(AccessListTransactionField::Payload, source))?;
    if list.item_count() != ACCESS_LIST_TRANSACTION_FIELD_COUNT {
        return Err(AccessListTransactionDecodeError::WrongFieldCount {
            expected: ACCESS_LIST_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }
    let decoded = fixed_fields_in_session::<ACCESS_LIST_TRANSACTION_FIELD_COUNT>(list, session)
        .map_err(|source| field_error(AccessListTransactionField::Payload, source))?;
    let mut fields = decoded
        .into_iter()
        .map(|item| item.ok_or(DecodeError::Malformed));

    let chain_id = decode_shared_chain_id(
        &mut fields,
        AccessListTransactionField::ChainId,
        field_error,
    )?;
    let nonce = Nonce::new(decode_shared_u64_field(
        &mut fields,
        AccessListTransactionField::Nonce,
        field_error,
    )?);
    let gas_price = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        AccessListTransactionField::GasPrice,
        field_error,
    )?);
    let gas_limit = Gas::new(decode_shared_u64_field(
        &mut fields,
        AccessListTransactionField::GasLimit,
        field_error,
    )?);
    let to = decode_shared_to(
        next_shared_scalar(&mut fields, AccessListTransactionField::To, field_error)?,
        |found| AccessListTransactionDecodeError::InvalidToLength { found },
    )?;
    let value = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        AccessListTransactionField::Value,
        field_error,
    )?);
    let input =
        next_shared_scalar(&mut fields, AccessListTransactionField::Data, field_error)?.payload();
    session
        .limits()
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(AccessListTransactionField::Data, source))?;
    let access_list = decode_access_list_in_session(
        next_shared_list(
            &mut fields,
            AccessListTransactionField::AccessList,
            field_error,
        )?,
        session,
    )
    .map_err(map_access_list_decode_error)?;
    let y_parity = SignatureYParity::try_new(decode_shared_u64_field(
        &mut fields,
        AccessListTransactionField::SignatureYParity,
        field_error,
    )?)
    .map_err(|error| AccessListTransactionDecodeError::InvalidYParity {
        value: error.value(),
    })?;
    let r = decode_shared_u256_field(
        &mut fields,
        AccessListTransactionField::SignatureR,
        field_error,
    )?;
    let s = decode_shared_u256_field(
        &mut fields,
        AccessListTransactionField::SignatureS,
        field_error,
    )?;

    Ok(UnvalidatedAccessListTransaction {
        chain_id,
        nonce,
        gas_price,
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

pub(crate) fn decode_access_list_in_session<'a>(
    list: RlpList<'a>,
    session: &mut DecodeSession,
) -> Result<AccessList<'a>, AccessListDecodeError> {
    let mut storage_key_count = 0usize;
    let mut items = list.items();
    while let Some(item) = items.next_in_session(session) {
        let entry = decode_entry(item, session)?;
        storage_key_count =
            eth_valkyoth_codec::checked_len_add(storage_key_count, entry.storage_keys.len())
                .map_err(AccessListDecodeError::FieldDecode)?;
    }
    Ok(AccessList {
        list,
        storage_key_count,
    })
}

fn decode_entry<'a>(
    item: Result<RlpItem<'a>, DecodeError>,
    session: &mut DecodeSession,
) -> Result<AccessListEntry<'a>, AccessListDecodeError> {
    let RlpItem::List(list) = item.map_err(AccessListDecodeError::FieldDecode)? else {
        return Err(AccessListDecodeError::FieldDecode(
            DecodeError::UnexpectedScalar,
        ));
    };
    if list.item_count() != ACCESS_LIST_ENTRY_FIELD_COUNT {
        return Err(AccessListDecodeError::InvalidAccessListEntryFieldCount {
            found: list.item_count(),
        });
    }
    let decoded = fixed_fields_in_session::<ACCESS_LIST_ENTRY_FIELD_COUNT>(list, session)
        .map_err(AccessListDecodeError::FieldDecode)?;
    let mut fields = decoded
        .into_iter()
        .map(|item| item.ok_or(DecodeError::Malformed));
    let address = decode_access_list_address(next_shared_scalar(
        &mut fields,
        AccessListTransactionField::AccessList,
        |_, source| AccessListDecodeError::FieldDecode(source),
    )?)?;
    let storage_keys = AccessListStorageKeys {
        list: next_shared_list(
            &mut fields,
            AccessListTransactionField::AccessList,
            |_, source| AccessListDecodeError::FieldDecode(source),
        )?,
    };
    let mut keys = storage_keys.list.items();
    while let Some(key) = keys.next_in_session(session) {
        let _ = decode_storage_key_item(key)?;
    }
    Ok(AccessListEntry {
        address,
        storage_keys,
    })
}
