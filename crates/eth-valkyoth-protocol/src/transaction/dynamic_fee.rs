use eth_valkyoth_codec::{DecodeError, DecodeLimits};
use eth_valkyoth_primitives::{ChainId, Gas, Nonce, Wei};

use super::access_list::{AccessListDecodeError, decode_access_list};
use super::{
    AccessList, AccessListTransactionTo, SignatureYParity, TransactionEnvelope,
    decode_transaction_envelope,
};
use crate::transaction::fields::{
    decode_chain_id as decode_shared_chain_id, decode_to as decode_shared_to,
    decode_u64_field as decode_shared_u64_field, decode_u256_field as decode_shared_u256_field,
    next_list as next_shared_list, next_scalar as next_shared_scalar,
};

mod error;
mod session;

pub use error::{DynamicFeeTransactionDecodeError, DynamicFeeTransactionDecodeErrorCategory};
pub use session::decode_dynamic_fee_transaction_in_session;

/// EIP-1559 dynamic-fee transaction type byte.
pub const DYNAMIC_FEE_TRANSACTION_TYPE: u8 = 0x02;
/// Number of fields in an EIP-1559 dynamic-fee transaction payload.
pub const DYNAMIC_FEE_TRANSACTION_FIELD_COUNT: usize = 12;

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
    /// Whole typed-transaction payload.
    Payload,
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
        .map_err(|source| field_error(DynamicFeeTransactionField::Payload, source))?;
    if list.item_count() != DYNAMIC_FEE_TRANSACTION_FIELD_COUNT {
        return Err(DynamicFeeTransactionDecodeError::WrongFieldCount {
            expected: DYNAMIC_FEE_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
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
    limits
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(DynamicFeeTransactionField::Data, source))?;
    let access_list = decode_access_list(next_shared_list(
        &mut fields,
        DynamicFeeTransactionField::AccessList,
        field_error,
    )?)
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

const fn field_error(
    field: DynamicFeeTransactionField,
    source: DecodeError,
) -> DynamicFeeTransactionDecodeError {
    DynamicFeeTransactionDecodeError::FieldDecode { field, source }
}

const fn map_access_list_error(error: AccessListDecodeError) -> DynamicFeeTransactionDecodeError {
    match error {
        AccessListDecodeError::FieldDecode(source) => {
            field_error(DynamicFeeTransactionField::AccessList, source)
        }
        AccessListDecodeError::InvalidAccessListEntryFieldCount { found } => {
            DynamicFeeTransactionDecodeError::InvalidAccessListEntryFieldCount { found }
        }
        AccessListDecodeError::InvalidAccessListAddressLength { found } => {
            DynamicFeeTransactionDecodeError::InvalidAccessListAddressLength { found }
        }
        AccessListDecodeError::InvalidStorageKeyLength { found } => {
            DynamicFeeTransactionDecodeError::InvalidStorageKeyLength { found }
        }
    }
}
