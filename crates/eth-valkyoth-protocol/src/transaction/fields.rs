use eth_valkyoth_codec::{DecodeError, RlpInteger, RlpItem, RlpList, RlpScalar};
use eth_valkyoth_primitives::{Address, ChainId};

use super::access_list::AccessListTransactionTo;

pub(crate) const ADDRESS_BYTES: usize = 20;

pub(crate) fn decode_chain_id<'a, F, E>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: F,
    err: impl Copy + Fn(F, DecodeError) -> E,
) -> Result<ChainId, E>
where
    F: Copy,
{
    let integer = RlpInteger::try_from_scalar(next_scalar(fields, field, err)?)
        .map_err(|source| err(field, source))?;
    ChainId::try_from_signed_canonical_be_slice(integer.payload())
        .map_err(|_| err(field, DecodeError::Malformed))
}

pub(crate) fn decode_to<E>(
    scalar: RlpScalar<'_>,
    invalid_len: impl Fn(usize) -> E,
) -> Result<AccessListTransactionTo, E> {
    let payload = scalar.payload();
    if payload.is_empty() {
        return Ok(AccessListTransactionTo::Create);
    }
    let found = payload.len();
    let bytes: [u8; ADDRESS_BYTES] = payload.try_into().map_err(|_| invalid_len(found))?;
    Ok(AccessListTransactionTo::Call(Address::from_bytes(bytes)))
}

pub(crate) fn decode_required_address<E>(
    scalar: RlpScalar<'_>,
    invalid_len: impl Fn(usize) -> E,
) -> Result<Address, E> {
    let found = scalar.payload().len();
    let bytes: [u8; ADDRESS_BYTES] = scalar
        .payload()
        .try_into()
        .map_err(|_| invalid_len(found))?;
    Ok(Address::from_bytes(bytes))
}

pub(crate) fn next_scalar<'a, F, E>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: F,
    err: impl Fn(F, DecodeError) -> E,
) -> Result<RlpScalar<'a>, E>
where
    F: Copy,
{
    let item = fields
        .next()
        .ok_or(err(field, DecodeError::Malformed))?
        .map_err(|source| err(field, source))?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(err(field, DecodeError::UnexpectedList)),
    }
}

pub(crate) fn next_list<'a, F, E>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: F,
    err: impl Fn(F, DecodeError) -> E,
) -> Result<RlpList<'a>, E>
where
    F: Copy,
{
    let item = fields
        .next()
        .ok_or(err(field, DecodeError::Malformed))?
        .map_err(|source| err(field, source))?;
    match item {
        RlpItem::List(list) => Ok(list),
        RlpItem::Scalar(_) => Err(err(field, DecodeError::UnexpectedScalar)),
    }
}

pub(crate) fn decode_u64_field<'a, F, E>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: F,
    err: impl Copy + Fn(F, DecodeError) -> E,
) -> Result<u64, E>
where
    F: Copy,
{
    RlpInteger::try_from_scalar(next_scalar(fields, field, err)?)
        .and_then(RlpInteger::to_u64)
        .map_err(|source| err(field, source))
}

pub(crate) fn decode_u256_field<'a, F, E>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: F,
    err: impl Copy + Fn(F, DecodeError) -> E,
) -> Result<[u8; 32], E>
where
    F: Copy,
{
    RlpInteger::try_from_scalar(next_scalar(fields, field, err)?)
        .and_then(RlpInteger::to_be_bytes32)
        .map_err(|source| err(field, source))
}
