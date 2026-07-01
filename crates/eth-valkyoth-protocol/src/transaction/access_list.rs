use eth_valkyoth_codec::{DecodeError, DecodeLimits, RlpInteger, RlpItem, RlpList, RlpScalar};
use eth_valkyoth_primitives::{Address, B256, ChainId, Gas, Nonce, Wei};

use super::{TransactionEnvelope, decode_transaction_envelope};

mod error;

pub use error::{AccessListTransactionDecodeError, AccessListTransactionDecodeErrorCategory};

/// EIP-2930 transaction type byte.
pub const ACCESS_LIST_TRANSACTION_TYPE: u8 = 0x01;
/// Number of fields in an EIP-2930 access-list transaction payload.
pub const ACCESS_LIST_TRANSACTION_FIELD_COUNT: usize = 11;

const ACCESS_LIST_ENTRY_FIELD_COUNT: usize = 2;
const ADDRESS_BYTES: usize = 20;
const B256_BYTES: usize = 32;

/// Borrowed EIP-2930 transaction decoded only into field domains.
///
/// This type is intentionally unvalidated: no sender recovery, signature
/// validity, gas accounting, account-state check, duplicate access-list policy,
/// or fork validity is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedAccessListTransaction<'a> {
    /// Chain ID encoded in the signed transaction domain.
    pub chain_id: ChainId,
    /// Account nonce.
    pub nonce: Nonce,
    /// Gas price in wei.
    pub gas_price: Wei,
    /// Gas limit.
    pub gas_limit: Gas,
    /// Call or contract-creation target.
    pub to: AccessListTransactionTo,
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

/// EIP-2930 transaction call/create target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessListTransactionTo {
    /// Contract creation transaction with an empty `to` field.
    Create,
    /// Message call to an address.
    Call(Address),
}

/// EIP-2930 signature y parity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureYParity {
    /// Even y coordinate.
    Even,
    /// Odd y coordinate.
    Odd,
}

impl SignatureYParity {
    /// Creates y parity from its wire integer.
    pub const fn try_new(value: u64) -> Result<Self, AccessListTransactionDecodeError> {
        match value {
            0 => Ok(Self::Even),
            1 => Ok(Self::Odd),
            _ => Err(AccessListTransactionDecodeError::InvalidYParity { value }),
        }
    }

    /// Returns the raw y-parity bit.
    #[must_use]
    pub const fn get(self) -> u8 {
        match self {
            Self::Even => 0,
            Self::Odd => 1,
        }
    }
}

/// Borrowed EIP-2930 access list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessList<'a> {
    list: RlpList<'a>,
    storage_key_count: usize,
}

impl<'a> AccessList<'a> {
    /// Returns the number of address entries.
    #[must_use]
    pub const fn address_count(self) -> usize {
        self.list.item_count()
    }

    /// Returns the total number of storage keys across all entries.
    #[must_use]
    pub const fn storage_key_count(self) -> usize {
        self.storage_key_count
    }

    /// Returns an iterator over access-list entries.
    ///
    /// The transaction decoder validates every access-list entry before
    /// returning this borrowed model. Iterating re-parses the same bounded RLP
    /// bytes so callers can use zero-copy access without storing decoded
    /// entries.
    #[must_use]
    pub const fn entries(self) -> AccessListEntries<'a> {
        AccessListEntries {
            items: self.list.items(),
        }
    }
}

/// Borrowed EIP-2930 access-list entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessListEntry<'a> {
    /// Accessed address.
    pub address: Address,
    /// Storage keys for this address.
    pub storage_keys: AccessListStorageKeys<'a>,
}

/// Borrowed storage-key list for one access-list address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessListStorageKeys<'a> {
    list: RlpList<'a>,
}

impl<'a> AccessListStorageKeys<'a> {
    /// Returns the number of storage keys in this entry.
    #[must_use]
    pub const fn len(self) -> usize {
        self.list.item_count()
    }

    /// Returns true when this entry has no storage keys.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.list.is_empty()
    }

    /// Returns an iterator over storage keys.
    ///
    /// The parent access-list decoder validates every storage key before
    /// returning this borrowed model. Iterating re-parses the same bounded RLP
    /// bytes so callers can use zero-copy access without storing decoded keys.
    #[must_use]
    pub const fn keys(self) -> AccessListStorageKeyItems<'a> {
        AccessListStorageKeyItems {
            items: self.list.items(),
        }
    }
}

/// Iterator over borrowed access-list entries.
#[derive(Clone, Debug)]
pub struct AccessListEntries<'a> {
    items: eth_valkyoth_codec::RlpListItems<'a>,
}

impl<'a> Iterator for AccessListEntries<'a> {
    type Item = Result<AccessListEntry<'a>, AccessListTransactionDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(decode_access_list_entry_item)
    }
}

impl core::iter::FusedIterator for AccessListEntries<'_> {}

/// Iterator over borrowed storage keys.
#[derive(Clone, Debug)]
pub struct AccessListStorageKeyItems<'a> {
    items: eth_valkyoth_codec::RlpListItems<'a>,
}

impl Iterator for AccessListStorageKeyItems<'_> {
    type Item = Result<B256, AccessListTransactionDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(decode_storage_key_item)
    }
}

impl core::iter::FusedIterator for AccessListStorageKeyItems<'_> {}

/// EIP-2930 transaction field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessListTransactionField {
    /// `chainId`.
    ChainId,
    /// `nonce`.
    Nonce,
    /// `gasPrice`.
    GasPrice,
    /// `gasLimit`.
    GasLimit,
    /// `to`.
    To,
    /// `value`.
    Value,
    /// `data`.
    Data,
    /// `accessList`.
    AccessList,
    /// `signatureYParity`.
    SignatureYParity,
    /// `signatureR`.
    SignatureR,
    /// `signatureS`.
    SignatureS,
}

/// Decodes an EIP-2930 access-list transaction into unvalidated field domains.
pub fn decode_access_list_transaction<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedAccessListTransaction<'a>, AccessListTransactionDecodeError> {
    match decode_transaction_envelope(input, limits)
        .map_err(AccessListTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == ACCESS_LIST_TRANSACTION_TYPE =>
        {
            decode_access_list_payload(typed.payload, limits)
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

fn decode_access_list_payload<'a>(
    payload: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedAccessListTransaction<'a>, AccessListTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list(payload, limits)
        .map_err(|source| field_error(AccessListTransactionField::AccessList, source))?;
    if list.item_count() != ACCESS_LIST_TRANSACTION_FIELD_COUNT {
        return Err(AccessListTransactionDecodeError::WrongFieldCount {
            expected: ACCESS_LIST_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
    let chain_id = decode_chain_id(&mut fields)?;
    let nonce = Nonce::new(decode_u64_field(
        &mut fields,
        AccessListTransactionField::Nonce,
    )?);
    let gas_price = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        AccessListTransactionField::GasPrice,
    )?);
    let gas_limit = Gas::new(decode_u64_field(
        &mut fields,
        AccessListTransactionField::GasLimit,
    )?);
    let to = decode_to(next_scalar(&mut fields, AccessListTransactionField::To)?)?;
    let value = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        AccessListTransactionField::Value,
    )?);
    let input = next_scalar(&mut fields, AccessListTransactionField::Data)?.payload();
    limits
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(AccessListTransactionField::Data, source))?;
    let access_list = decode_access_list(next_list(
        &mut fields,
        AccessListTransactionField::AccessList,
    )?)?;
    let y_parity = SignatureYParity::try_new(decode_u64_field(
        &mut fields,
        AccessListTransactionField::SignatureYParity,
    )?)?;
    let r = decode_u256_field(&mut fields, AccessListTransactionField::SignatureR)?;
    let s = decode_u256_field(&mut fields, AccessListTransactionField::SignatureS)?;

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

fn decode_chain_id<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
) -> Result<ChainId, AccessListTransactionDecodeError> {
    let integer =
        RlpInteger::try_from_scalar(next_scalar(fields, AccessListTransactionField::ChainId)?)
            .map_err(|source| field_error(AccessListTransactionField::ChainId, source))?;
    ChainId::try_from_signed_canonical_be_slice(integer.payload())
        .map_err(|_| field_error(AccessListTransactionField::ChainId, DecodeError::Malformed))
}

pub(crate) fn decode_access_list(
    list: RlpList<'_>,
) -> Result<AccessList<'_>, AccessListTransactionDecodeError> {
    // Eager validation proves the borrowed model is well-formed. Later
    // iteration intentionally re-parses these bounded bytes instead of storing
    // decoded entries in an allocation-backed structure.
    let mut storage_key_count = 0usize;
    for item in list.items() {
        let entry = decode_access_list_entry_item(item)?;
        storage_key_count =
            eth_valkyoth_codec::checked_len_add(storage_key_count, entry.storage_keys.len())
                .map_err(|source| field_error(AccessListTransactionField::AccessList, source))?;
    }
    Ok(AccessList {
        list,
        storage_key_count,
    })
}

fn decode_access_list_entry_item(
    item: Result<RlpItem<'_>, DecodeError>,
) -> Result<AccessListEntry<'_>, AccessListTransactionDecodeError> {
    let item =
        item.map_err(|source| field_error(AccessListTransactionField::AccessList, source))?;
    let RlpItem::List(list) = item else {
        return Err(field_error(
            AccessListTransactionField::AccessList,
            DecodeError::UnexpectedScalar,
        ));
    };
    if list.item_count() != ACCESS_LIST_ENTRY_FIELD_COUNT {
        return Err(
            AccessListTransactionDecodeError::InvalidAccessListEntryFieldCount {
                found: list.item_count(),
            },
        );
    }

    let mut fields = list.items();
    let address = decode_access_list_address(next_scalar(
        &mut fields,
        AccessListTransactionField::AccessList,
    )?)?;
    let storage_keys = AccessListStorageKeys {
        list: next_list(&mut fields, AccessListTransactionField::AccessList)?,
    };

    for key in storage_keys.keys() {
        let _ = key?;
    }
    Ok(AccessListEntry {
        address,
        storage_keys,
    })
}

fn decode_storage_key_item(
    item: Result<RlpItem<'_>, DecodeError>,
) -> Result<B256, AccessListTransactionDecodeError> {
    let item =
        item.map_err(|source| field_error(AccessListTransactionField::AccessList, source))?;
    let RlpItem::Scalar(scalar) = item else {
        return Err(field_error(
            AccessListTransactionField::AccessList,
            DecodeError::UnexpectedList,
        ));
    };
    let found = scalar.payload().len();
    let bytes: [u8; B256_BYTES] = scalar
        .payload()
        .try_into()
        .map_err(|_| AccessListTransactionDecodeError::InvalidStorageKeyLength { found })?;
    Ok(B256::from_bytes(bytes))
}

fn decode_to(
    scalar: RlpScalar<'_>,
) -> Result<AccessListTransactionTo, AccessListTransactionDecodeError> {
    let payload = scalar.payload();
    if payload.is_empty() {
        return Ok(AccessListTransactionTo::Create);
    }
    let found = payload.len();
    let bytes: [u8; ADDRESS_BYTES] = payload
        .try_into()
        .map_err(|_| AccessListTransactionDecodeError::InvalidToLength { found })?;
    Ok(AccessListTransactionTo::Call(Address::from_bytes(bytes)))
}

fn decode_access_list_address(
    scalar: RlpScalar<'_>,
) -> Result<Address, AccessListTransactionDecodeError> {
    let found = scalar.payload().len();
    let bytes: [u8; ADDRESS_BYTES] = scalar
        .payload()
        .try_into()
        .map_err(|_| AccessListTransactionDecodeError::InvalidAccessListAddressLength { found })?;
    Ok(Address::from_bytes(bytes))
}

fn next_scalar<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: AccessListTransactionField,
) -> Result<RlpScalar<'a>, AccessListTransactionDecodeError> {
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
    field: AccessListTransactionField,
) -> Result<RlpList<'a>, AccessListTransactionDecodeError> {
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
    field: AccessListTransactionField,
) -> Result<u64, AccessListTransactionDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_u64)
        .map_err(|source| field_error(field, source))
}

fn decode_u256_field<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: AccessListTransactionField,
) -> Result<[u8; 32], AccessListTransactionDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_be_bytes32)
        .map_err(|source| field_error(field, source))
}

const fn field_error(
    field: AccessListTransactionField,
    source: DecodeError,
) -> AccessListTransactionDecodeError {
    AccessListTransactionDecodeError::FieldDecode { field, source }
}
