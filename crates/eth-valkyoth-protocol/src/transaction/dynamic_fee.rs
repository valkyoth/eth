use eth_valkyoth_codec::{DecodeError, DecodeLimits, RlpInteger, RlpItem, RlpScalar};
use eth_valkyoth_primitives::{Address, ChainId, Gas, Nonce, Wei};

use super::access_list::decode_access_list;
use super::{
    AccessList, AccessListTransactionDecodeError, AccessListTransactionTo, SignatureYParity,
    TransactionEnvelope, decode_transaction_envelope,
};

mod error;

pub use error::{DynamicFeeTransactionDecodeError, DynamicFeeTransactionDecodeErrorCategory};

/// EIP-1559 dynamic-fee transaction type byte.
pub const DYNAMIC_FEE_TRANSACTION_TYPE: u8 = 0x02;
/// Number of fields in an EIP-1559 dynamic-fee transaction payload.
pub const DYNAMIC_FEE_TRANSACTION_FIELD_COUNT: usize = 12;

const ADDRESS_BYTES: usize = 20;

/// EIP-1559 transaction call/create target.
pub type DynamicFeeTransactionTo = AccessListTransactionTo;

/// Borrowed EIP-1559 transaction decoded only into field domains.
///
/// This type is intentionally unvalidated: no sender recovery, signature
/// validity, gas accounting, account-state check, fee-order check, duplicate
/// access-list policy, or fork validity is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedDynamicFeeTransaction<'a> {
    /// Chain ID encoded in the signed transaction domain.
    pub chain_id: ChainId,
    /// Account nonce.
    pub nonce: Nonce,
    /// Maximum priority fee per gas in wei.
    pub max_priority_fee_per_gas: Wei,
    /// Maximum total fee per gas in wei.
    pub max_fee_per_gas: Wei,
    /// Gas limit.
    pub gas_limit: Gas,
    /// Call or contract-creation target.
    pub to: DynamicFeeTransactionTo,
    /// Transferred value in wei.
    pub value: Wei,
    /// Borrowed transaction input data.
    pub input: &'a [u8],
    /// Borrowed access list.
    pub access_list: AccessList<'a>,
    /// Signature y parity.
    pub y_parity: SignatureYParity,
    /// Raw canonical U256 signature `r` value.
    ///
    /// This is not checked for secp256k1 scalar validity.
    pub r: [u8; 32],
    /// Raw canonical U256 signature `s` value.
    ///
    /// This is not checked against the EIP-2 low-s bound or secp256k1 scalar
    /// validity.
    pub s: [u8; 32],
}

/// EIP-1559 transaction field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynamicFeeTransactionField {
    /// `chain_id`.
    ChainId,
    /// `nonce`.
    Nonce,
    /// `max_priority_fee_per_gas`.
    MaxPriorityFeePerGas,
    /// `max_fee_per_gas`.
    MaxFeePerGas,
    /// `gas_limit`.
    GasLimit,
    /// `destination`.
    To,
    /// `amount`.
    Value,
    /// `data`.
    Data,
    /// `access_list`.
    AccessList,
    /// `signature_y_parity`.
    SignatureYParity,
    /// `signature_r`.
    SignatureR,
    /// `signature_s`.
    SignatureS,
}

/// Decodes an EIP-1559 dynamic-fee transaction into unvalidated field domains.
pub fn decode_dynamic_fee_transaction<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedDynamicFeeTransaction<'a>, DynamicFeeTransactionDecodeError> {
    match decode_transaction_envelope(input, limits)
        .map_err(DynamicFeeTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == DYNAMIC_FEE_TRANSACTION_TYPE =>
        {
            decode_dynamic_fee_payload(typed.payload, limits)
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

fn decode_dynamic_fee_payload<'a>(
    payload: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedDynamicFeeTransaction<'a>, DynamicFeeTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list(payload, limits)
        .map_err(|source| field_error(DynamicFeeTransactionField::AccessList, source))?;
    if list.item_count() != DYNAMIC_FEE_TRANSACTION_FIELD_COUNT {
        return Err(DynamicFeeTransactionDecodeError::WrongFieldCount {
            expected: DYNAMIC_FEE_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
    let chain_id = decode_chain_id(&mut fields)?;
    let nonce = Nonce::new(decode_u64_field(
        &mut fields,
        DynamicFeeTransactionField::Nonce,
    )?);
    let max_priority_fee_per_gas = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        DynamicFeeTransactionField::MaxPriorityFeePerGas,
    )?);
    let max_fee_per_gas = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        DynamicFeeTransactionField::MaxFeePerGas,
    )?);
    let gas_limit = Gas::new(decode_u64_field(
        &mut fields,
        DynamicFeeTransactionField::GasLimit,
    )?);
    let to = decode_to(next_scalar(&mut fields, DynamicFeeTransactionField::To)?)?;
    let value = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        DynamicFeeTransactionField::Value,
    )?);
    let input = next_scalar(&mut fields, DynamicFeeTransactionField::Data)?.payload();
    limits
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(DynamicFeeTransactionField::Data, source))?;
    let access_list = decode_access_list(next_list(
        &mut fields,
        DynamicFeeTransactionField::AccessList,
    )?)
    .map_err(map_access_list_error)?;
    let y_parity = SignatureYParity::try_new(decode_u64_field(
        &mut fields,
        DynamicFeeTransactionField::SignatureYParity,
    )?)
    .map_err(|error| match error {
        AccessListTransactionDecodeError::InvalidYParity { value } => {
            DynamicFeeTransactionDecodeError::InvalidYParity { value }
        }
        _ => field_error(
            DynamicFeeTransactionField::SignatureYParity,
            DecodeError::Malformed,
        ),
    })?;
    let r = decode_u256_field(&mut fields, DynamicFeeTransactionField::SignatureR)?;
    let s = decode_u256_field(&mut fields, DynamicFeeTransactionField::SignatureS)?;

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

fn decode_chain_id<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
) -> Result<ChainId, DynamicFeeTransactionDecodeError> {
    let integer =
        RlpInteger::try_from_scalar(next_scalar(fields, DynamicFeeTransactionField::ChainId)?)
            .map_err(|source| field_error(DynamicFeeTransactionField::ChainId, source))?;
    ChainId::try_from_signed_canonical_be_slice(integer.payload())
        .map_err(|_| field_error(DynamicFeeTransactionField::ChainId, DecodeError::Malformed))
}

fn decode_to(
    scalar: RlpScalar<'_>,
) -> Result<DynamicFeeTransactionTo, DynamicFeeTransactionDecodeError> {
    let payload = scalar.payload();
    if payload.is_empty() {
        return Ok(DynamicFeeTransactionTo::Create);
    }
    let found = payload.len();
    let bytes: [u8; ADDRESS_BYTES] = payload
        .try_into()
        .map_err(|_| DynamicFeeTransactionDecodeError::InvalidToLength { found })?;
    Ok(DynamicFeeTransactionTo::Call(Address::from_bytes(bytes)))
}

fn next_scalar<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: DynamicFeeTransactionField,
) -> Result<RlpScalar<'a>, DynamicFeeTransactionDecodeError> {
    let item = fields
        .next()
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(field_error(field, DecodeError::UnexpectedList)),
    }
}

fn next_list<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: DynamicFeeTransactionField,
) -> Result<eth_valkyoth_codec::RlpList<'a>, DynamicFeeTransactionDecodeError> {
    let item = fields
        .next()
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::List(list) => Ok(list),
        RlpItem::Scalar(_) => Err(field_error(field, DecodeError::UnexpectedScalar)),
    }
}

fn decode_u64_field<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: DynamicFeeTransactionField,
) -> Result<u64, DynamicFeeTransactionDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_u64)
        .map_err(|source| field_error(field, source))
}

fn decode_u256_field<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: DynamicFeeTransactionField,
) -> Result<[u8; 32], DynamicFeeTransactionDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_be_bytes32)
        .map_err(|source| field_error(field, source))
}

const fn field_error(
    field: DynamicFeeTransactionField,
    source: DecodeError,
) -> DynamicFeeTransactionDecodeError {
    DynamicFeeTransactionDecodeError::FieldDecode { field, source }
}

const fn map_access_list_error(
    error: AccessListTransactionDecodeError,
) -> DynamicFeeTransactionDecodeError {
    match error {
        AccessListTransactionDecodeError::FieldDecode { source, .. } => {
            field_error(DynamicFeeTransactionField::AccessList, source)
        }
        AccessListTransactionDecodeError::InvalidAccessListEntryFieldCount { found } => {
            DynamicFeeTransactionDecodeError::InvalidAccessListEntryFieldCount { found }
        }
        AccessListTransactionDecodeError::InvalidAccessListAddressLength { found } => {
            DynamicFeeTransactionDecodeError::InvalidAccessListAddressLength { found }
        }
        AccessListTransactionDecodeError::InvalidStorageKeyLength { found } => {
            DynamicFeeTransactionDecodeError::InvalidStorageKeyLength { found }
        }
        _ => field_error(
            DynamicFeeTransactionField::AccessList,
            DecodeError::Malformed,
        ),
    }
}
