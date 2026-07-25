use eth_valkyoth_codec::{
    DecodeSession, RlpInteger, RlpItem, RlpScalar, decode_rlp_list_in_session,
};
use eth_valkyoth_primitives::{B256, Nonce, Wei};

use super::{AccountDecodeError, AccountField, EthereumAccount, StorageTrieRoot};

pub(super) fn decode_account(
    encoded: &[u8],
    session: &mut DecodeSession,
) -> Result<EthereumAccount, AccountDecodeError> {
    let list = decode_rlp_list_in_session(encoded, session).map_err(AccountDecodeError::Rlp)?;
    if list.item_count() != EthereumAccount::FIELD_COUNT {
        return Err(AccountDecodeError::FieldCount {
            found: list.item_count(),
        });
    }

    let mut items = list.items();
    let nonce = integer(
        next_scalar(&mut items, AccountField::Nonce, session)?,
        AccountField::Nonce,
    )?
    .to_u64()
    .map(Nonce::new)
    .map_err(|source| AccountDecodeError::FieldRlp {
        field: AccountField::Nonce,
        source,
    })?;
    let balance = integer(
        next_scalar(&mut items, AccountField::Balance, session)?,
        AccountField::Balance,
    )?
    .to_be_bytes32()
    .map(Wei::from_be_bytes)
    .map_err(|source| AccountDecodeError::FieldRlp {
        field: AccountField::Balance,
        source,
    })?;
    let storage_root = StorageTrieRoot::from_b256(fixed_hash(
        next_scalar(&mut items, AccountField::StorageRoot, session)?,
        AccountField::StorageRoot,
    )?);
    let code_hash = fixed_hash(
        next_scalar(&mut items, AccountField::CodeHash, session)?,
        AccountField::CodeHash,
    )?;

    Ok(EthereumAccount::new(
        nonce,
        balance,
        storage_root,
        code_hash,
    ))
}

fn next_scalar<'a>(
    items: &mut eth_valkyoth_codec::RlpListItems<'a>,
    field: AccountField,
    session: &mut DecodeSession,
) -> Result<RlpScalar<'a>, AccountDecodeError> {
    match items.next_in_session(session) {
        Some(Ok(RlpItem::Scalar(scalar))) => Ok(scalar),
        Some(Ok(RlpItem::List(_))) => Err(AccountDecodeError::UnexpectedList { field }),
        Some(Err(source)) => Err(AccountDecodeError::FieldRlp { field, source }),
        None => Err(AccountDecodeError::FieldCount { found: 0 }),
    }
}

fn integer(
    scalar: RlpScalar<'_>,
    field: AccountField,
) -> Result<RlpInteger<'_>, AccountDecodeError> {
    RlpInteger::try_from_scalar(scalar)
        .map_err(|source| AccountDecodeError::FieldRlp { field, source })
}

fn fixed_hash(scalar: RlpScalar<'_>, field: AccountField) -> Result<B256, AccountDecodeError> {
    let found = scalar.payload().len();
    let bytes = scalar
        .payload()
        .try_into()
        .map_err(|_| AccountDecodeError::FixedWidth {
            field,
            expected: 32,
            found,
        })?;
    Ok(B256::from_bytes(bytes))
}
