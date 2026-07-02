use eth_valkyoth_codec::{DecodeError, DecodeLimits, RlpInteger, RlpItem, RlpList, RlpScalar};
use eth_valkyoth_primitives::{Address, ChainId, Gas, Nonce, Wei};

use super::access_list::{AccessListDecodeError, decode_access_list};
use super::{AccessList, SignatureYParity, TransactionEnvelope, decode_transaction_envelope};
use crate::transaction::fields::{
    ADDRESS_BYTES, decode_chain_id as decode_shared_chain_id, decode_required_address,
    decode_u64_field as decode_shared_u64_field, decode_u256_field as decode_shared_u256_field,
    next_list as next_shared_list, next_scalar as next_shared_scalar,
};

mod error;
mod validity;

pub use error::{
    SetCodeAuthorizationField, SetCodeTransactionDecodeError, SetCodeTransactionDecodeErrorCategory,
};
pub use validity::*;

/// EIP-7702 set-code transaction type byte.
pub const SET_CODE_TRANSACTION_TYPE: u8 = 0x04;
/// Number of fields in an EIP-7702 set-code transaction payload.
pub const SET_CODE_TRANSACTION_FIELD_COUNT: usize = 13;
/// Number of fields in an EIP-7702 authorization tuple.
pub const SET_CODE_AUTHORIZATION_FIELD_COUNT: usize = 6;

/// Borrowed EIP-7702 set-code transaction decoded only into field domains.
///
/// This type is intentionally unvalidated: no sender recovery, signature
/// validity, fee-order check, authorization signature validation, non-empty
/// authorization-list policy, account-state check, delegation-indicator policy,
/// or fork validity is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedSetCodeTransaction<'a> {
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
    /// Call target. EIP-7702 set-code transactions cannot be contract creation.
    pub to: Address,
    /// Transferred value in wei.
    pub value: Wei,
    /// Borrowed transaction input data.
    pub input: &'a [u8],
    /// Borrowed access list.
    pub access_list: AccessList<'a>,
    /// Borrowed authorization list.
    pub authorization_list: SetCodeAuthorizationList<'a>,
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

/// EIP-7702 authorization chain ID.
///
/// Authorization chain IDs may be zero to allow a chain-agnostic
/// authorization. The outer transaction chain ID still uses [`ChainId`] and
/// remains nonzero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetCodeAuthorizationChainId {
    bytes: [u8; 32],
}

impl SetCodeAuthorizationChainId {
    /// Creates an authorization chain ID from canonical 256-bit bytes.
    #[must_use]
    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Returns the canonical 256-bit big-endian bytes.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; 32] {
        self.bytes
    }

    /// Returns true when the authorization is chain-agnostic.
    #[must_use]
    pub fn is_universal(self) -> bool {
        self.bytes.iter().all(|byte| *byte == 0)
    }
}

/// Borrowed EIP-7702 authorization list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetCodeAuthorizationList<'a> {
    list: RlpList<'a>,
}

impl<'a> SetCodeAuthorizationList<'a> {
    /// Returns the number of authorization tuples.
    #[must_use]
    pub const fn len(self) -> usize {
        self.list.item_count()
    }

    /// Returns true when the list is empty.
    ///
    /// EIP-7702 execution validation requires at least one authorization
    /// tuple. This syntactic decoder intentionally leaves that fork-validity
    /// check to a later validation state.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.list.is_empty()
    }

    /// Returns an iterator over authorization tuples.
    ///
    /// The transaction decoder validates every tuple before returning this
    /// borrowed model. Iterating re-parses the same bounded RLP bytes so callers
    /// can use zero-copy access without storing decoded authorizations.
    #[must_use]
    pub const fn authorizations(self) -> SetCodeAuthorizationItems<'a> {
        SetCodeAuthorizationItems {
            items: self.list.items(),
        }
    }

    /// Returns the encoded RLP list length of the borrowed authorization list.
    pub(crate) const fn encoded_rlp_len(self) -> usize {
        self.list.encoded_len()
    }

    /// Re-encodes the already validated borrowed authorization list.
    pub(crate) fn encode_rlp(self, output: &mut [u8]) -> Result<usize, DecodeError> {
        eth_valkyoth_codec::encode_decoded_list(self.list, output)
    }
}

/// Borrowed EIP-7702 authorization tuple.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetCodeAuthorization {
    /// Chain ID for this authorization. Zero means chain-agnostic.
    pub chain_id: SetCodeAuthorizationChainId,
    /// Delegation target address.
    pub address: Address,
    /// Authorizing account nonce.
    pub nonce: Nonce,
    /// Authorization signature y parity.
    pub y_parity: SignatureYParity,
    /// Raw canonical U256 authorization signature `r` value.
    pub r: [u8; 32],
    /// Raw canonical U256 authorization signature `s` value.
    pub s: [u8; 32],
}

/// Iterator over borrowed EIP-7702 authorization tuples.
#[derive(Clone, Debug)]
pub struct SetCodeAuthorizationItems<'a> {
    items: eth_valkyoth_codec::RlpListItems<'a>,
}

impl Iterator for SetCodeAuthorizationItems<'_> {
    type Item = Result<SetCodeAuthorization, SetCodeTransactionDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items
            .next()
            .map(|item| decode_authorization_item(item).map_err(map_authorization_error))
    }
}

impl core::iter::FusedIterator for SetCodeAuthorizationItems<'_> {}

/// EIP-7702 transaction field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeTransactionField {
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
    /// `value`.
    Value,
    /// `data`.
    Data,
    /// `access_list`.
    AccessList,
    /// `authorization_list`.
    AuthorizationList,
    /// `signature_y_parity`.
    SignatureYParity,
    /// `signature_r`.
    SignatureR,
    /// `signature_s`.
    SignatureS,
}

/// Decodes an EIP-7702 set-code transaction into unvalidated field domains.
pub fn decode_set_code_transaction<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedSetCodeTransaction<'a>, SetCodeTransactionDecodeError> {
    match decode_transaction_envelope(input, limits)
        .map_err(SetCodeTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == SET_CODE_TRANSACTION_TYPE =>
        {
            decode_set_code_payload(typed.payload, limits)
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

fn decode_set_code_payload<'a>(
    payload: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedSetCodeTransaction<'a>, SetCodeTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list(payload, limits)
        .map_err(|source| field_error(SetCodeTransactionField::Payload, source))?;
    if list.item_count() != SET_CODE_TRANSACTION_FIELD_COUNT {
        return Err(SetCodeTransactionDecodeError::WrongFieldCount {
            expected: SET_CODE_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
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
    limits
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(SetCodeTransactionField::Data, source))?;
    let access_list = decode_access_list(next_shared_list(
        &mut fields,
        SetCodeTransactionField::AccessList,
        field_error,
    )?)
    .map_err(map_access_list_error)?;
    let authorization_list = decode_authorization_list(next_shared_list(
        &mut fields,
        SetCodeTransactionField::AuthorizationList,
        field_error,
    )?)
    .map_err(map_authorization_error)?;
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

fn decode_authorization_list(
    list: RlpList<'_>,
) -> Result<SetCodeAuthorizationList<'_>, SetCodeAuthorizationDecodeError> {
    for item in list.items() {
        let _ = decode_authorization_item(item)?;
    }
    Ok(SetCodeAuthorizationList { list })
}

fn decode_authorization_item(
    item: Result<RlpItem<'_>, DecodeError>,
) -> Result<SetCodeAuthorization, SetCodeAuthorizationDecodeError> {
    let item = item.map_err(SetCodeAuthorizationDecodeError::TupleDecode)?;
    let RlpItem::List(list) = item else {
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

    let mut fields = list.items();
    let chain_id = decode_authorization_chain_id(next_shared_scalar(
        &mut fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::ChainId, source),
    )?)?;
    let address = decode_authorization_address(next_shared_scalar(
        &mut fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::Address, source),
    )?)?;
    let nonce = Nonce::new(decode_shared_u64_field(
        &mut fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::Nonce, source),
    )?);
    let y_parity = SignatureYParity::try_new(decode_shared_u64_field(
        &mut fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::YParity, source),
    )?)
    .map_err(
        |error| SetCodeAuthorizationDecodeError::InvalidAuthorizationYParity {
            value: error.value(),
        },
    )?;
    let r = decode_shared_u256_field(
        &mut fields,
        SetCodeTransactionField::AuthorizationList,
        |_, source| auth_field_error(SetCodeAuthorizationField::R, source),
    )?;
    let s = decode_shared_u256_field(
        &mut fields,
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

fn decode_authorization_chain_id(
    scalar: RlpScalar<'_>,
) -> Result<SetCodeAuthorizationChainId, SetCodeAuthorizationDecodeError> {
    let bytes = RlpInteger::try_from_scalar(scalar)
        .and_then(RlpInteger::to_be_bytes32)
        .map_err(|source| auth_field_error(SetCodeAuthorizationField::ChainId, source))?;
    Ok(SetCodeAuthorizationChainId::from_be_bytes(bytes))
}

fn decode_authorization_address(
    scalar: RlpScalar<'_>,
) -> Result<Address, SetCodeAuthorizationDecodeError> {
    let found = scalar.payload().len();
    let bytes: [u8; ADDRESS_BYTES] = scalar.payload().try_into().map_err(|_| {
        SetCodeAuthorizationDecodeError::InvalidAuthorizationAddressLength { found }
    })?;
    Ok(Address::from_bytes(bytes))
}

const fn auth_field_error(
    field: SetCodeAuthorizationField,
    source: DecodeError,
) -> SetCodeAuthorizationDecodeError {
    SetCodeAuthorizationDecodeError::FieldDecode { field, source }
}

const fn field_error(
    field: SetCodeTransactionField,
    source: DecodeError,
) -> SetCodeTransactionDecodeError {
    SetCodeTransactionDecodeError::FieldDecode { field, source }
}

const fn map_access_list_error(error: AccessListDecodeError) -> SetCodeTransactionDecodeError {
    match error {
        AccessListDecodeError::FieldDecode(source) => {
            field_error(SetCodeTransactionField::AccessList, source)
        }
        AccessListDecodeError::InvalidAccessListEntryFieldCount { found } => {
            SetCodeTransactionDecodeError::InvalidAccessListEntryFieldCount { found }
        }
        AccessListDecodeError::InvalidAccessListAddressLength { found } => {
            SetCodeTransactionDecodeError::InvalidAccessListAddressLength { found }
        }
        AccessListDecodeError::InvalidStorageKeyLength { found } => {
            SetCodeTransactionDecodeError::InvalidStorageKeyLength { found }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SetCodeAuthorizationDecodeError {
    TupleDecode(DecodeError),
    FieldDecode {
        field: SetCodeAuthorizationField,
        source: DecodeError,
    },
    InvalidAuthorizationFieldCount {
        found: usize,
    },
    InvalidAuthorizationAddressLength {
        found: usize,
    },
    InvalidAuthorizationYParity {
        value: u64,
    },
}

const fn map_authorization_error(
    error: SetCodeAuthorizationDecodeError,
) -> SetCodeTransactionDecodeError {
    match error {
        SetCodeAuthorizationDecodeError::TupleDecode(source) => {
            field_error(SetCodeTransactionField::AuthorizationList, source)
        }
        SetCodeAuthorizationDecodeError::FieldDecode { field, source } => {
            SetCodeTransactionDecodeError::AuthorizationFieldDecode { field, source }
        }
        SetCodeAuthorizationDecodeError::InvalidAuthorizationFieldCount { found } => {
            SetCodeTransactionDecodeError::InvalidAuthorizationFieldCount { found }
        }
        SetCodeAuthorizationDecodeError::InvalidAuthorizationAddressLength { found } => {
            SetCodeTransactionDecodeError::InvalidAuthorizationAddressLength { found }
        }
        SetCodeAuthorizationDecodeError::InvalidAuthorizationYParity { value } => {
            SetCodeTransactionDecodeError::InvalidAuthorizationYParity { value }
        }
    }
}
