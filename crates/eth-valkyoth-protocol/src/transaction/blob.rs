use eth_valkyoth_codec::{DecodeError, DecodeLimits, RlpItem, RlpList};
use eth_valkyoth_primitives::{Address, B256, ChainId, Gas, Nonce, Wei};

use super::access_list::{AccessListDecodeError, decode_access_list};
use super::{AccessList, SignatureYParity, TransactionEnvelope, decode_transaction_envelope};
use crate::transaction::fields::{
    decode_chain_id as decode_shared_chain_id, decode_required_address,
    decode_u64_field as decode_shared_u64_field, decode_u256_field as decode_shared_u256_field,
    next_list as next_shared_list, next_scalar as next_shared_scalar,
};

mod error;

pub use error::{BlobTransactionDecodeError, BlobTransactionDecodeErrorCategory};

/// EIP-4844 blob transaction type byte.
pub const BLOB_TRANSACTION_TYPE: u8 = 0x03;
/// Number of fields in an EIP-4844 blob transaction payload.
pub const BLOB_TRANSACTION_FIELD_COUNT: usize = 14;

const B256_BYTES: usize = 32;

/// Borrowed EIP-4844 transaction decoded only into field domains.
///
/// This type is intentionally unvalidated: no sender recovery, signature
/// validity, fee-order check, blob fee adequacy, KZG proof validation,
/// data-availability validation, blob-hash version policy, account-state check,
/// block blob-gas accounting, or fork validity is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedBlobTransaction<'a> {
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
    /// Call target. EIP-4844 blob transactions cannot be contract creation.
    pub to: Address,
    /// Transferred value in wei.
    pub value: Wei,
    /// Borrowed transaction input data.
    pub input: &'a [u8],
    /// Borrowed access list.
    pub access_list: AccessList<'a>,
    /// Maximum fee per blob gas in wei.
    pub max_fee_per_blob_gas: Wei,
    /// Borrowed list of blob versioned hashes.
    pub blob_versioned_hashes: BlobVersionedHashes<'a>,
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

/// Borrowed EIP-4844 blob versioned hash list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlobVersionedHashes<'a> {
    list: RlpList<'a>,
}

impl<'a> BlobVersionedHashes<'a> {
    /// Returns the number of blob versioned hashes.
    #[must_use]
    pub const fn len(self) -> usize {
        self.list.item_count()
    }

    /// Returns true when the list is empty.
    ///
    /// EIP-4844 execution validation requires at least one blob hash. This
    /// syntactic decoder intentionally leaves that fork-validity check to a
    /// later validation state.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.list.is_empty()
    }

    /// Returns an iterator over blob versioned hashes.
    ///
    /// The transaction decoder validates every hash length before returning
    /// this borrowed model. Iterating re-parses the same bounded RLP bytes so
    /// callers can use zero-copy access without storing decoded hashes.
    #[must_use]
    pub const fn hashes(self) -> BlobVersionedHashItems<'a> {
        BlobVersionedHashItems {
            items: self.list.items(),
        }
    }

    /// Returns the encoded RLP list length of the borrowed hash list.
    pub(crate) const fn encoded_rlp_len(self) -> usize {
        self.list.encoded_len()
    }

    /// Re-encodes the already validated borrowed hash list.
    pub(crate) fn encode_rlp(self, output: &mut [u8]) -> Result<usize, DecodeError> {
        eth_valkyoth_codec::encode_decoded_list(self.list, output)
    }
}

/// Iterator over borrowed blob versioned hashes.
#[derive(Clone, Debug)]
pub struct BlobVersionedHashItems<'a> {
    items: eth_valkyoth_codec::RlpListItems<'a>,
}

impl Iterator for BlobVersionedHashItems<'_> {
    type Item = Result<B256, BlobTransactionDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(decode_blob_versioned_hash_item)
    }
}

impl core::iter::FusedIterator for BlobVersionedHashItems<'_> {}

/// EIP-4844 transaction field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobTransactionField {
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
    /// `to`.
    To,
    /// `value`.
    Value,
    /// `data`.
    Data,
    /// `access_list`.
    AccessList,
    /// `max_fee_per_blob_gas`.
    MaxFeePerBlobGas,
    /// `blob_versioned_hashes`.
    BlobVersionedHashes,
    /// `y_parity`.
    SignatureYParity,
    /// `r`.
    SignatureR,
    /// `s`.
    SignatureS,
}

/// Decodes an EIP-4844 blob transaction into unvalidated field domains.
pub fn decode_blob_transaction<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedBlobTransaction<'a>, BlobTransactionDecodeError> {
    match decode_transaction_envelope(input, limits)
        .map_err(BlobTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Typed(typed)
            if typed.transaction_type.get() == BLOB_TRANSACTION_TYPE =>
        {
            decode_blob_payload(typed.payload, limits)
        }
        TransactionEnvelope::Typed(typed) => {
            Err(BlobTransactionDecodeError::WrongTransactionType {
                type_byte: typed.transaction_type.get(),
            })
        }
        TransactionEnvelope::Legacy(_) => {
            Err(BlobTransactionDecodeError::WrongTransactionType { type_byte: 0 })
        }
    }
}

fn decode_blob_payload<'a>(
    payload: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedBlobTransaction<'a>, BlobTransactionDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list(payload, limits)
        .map_err(|source| field_error(BlobTransactionField::Payload, source))?;
    if list.item_count() != BLOB_TRANSACTION_FIELD_COUNT {
        return Err(BlobTransactionDecodeError::WrongFieldCount {
            expected: BLOB_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
    let chain_id = decode_shared_chain_id(&mut fields, BlobTransactionField::ChainId, field_error)?;
    let nonce = Nonce::new(decode_shared_u64_field(
        &mut fields,
        BlobTransactionField::Nonce,
        field_error,
    )?);
    let max_priority_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::MaxPriorityFeePerGas,
        field_error,
    )?);
    let max_fee_per_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::MaxFeePerGas,
        field_error,
    )?);
    let gas_limit = Gas::new(decode_shared_u64_field(
        &mut fields,
        BlobTransactionField::GasLimit,
        field_error,
    )?);
    let to = decode_required_address(
        next_shared_scalar(&mut fields, BlobTransactionField::To, field_error)?,
        |found| BlobTransactionDecodeError::InvalidToLength { found },
    )?;
    let value = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::Value,
        field_error,
    )?);
    let input = next_shared_scalar(&mut fields, BlobTransactionField::Data, field_error)?.payload();
    limits
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(BlobTransactionField::Data, source))?;
    let access_list = decode_access_list(next_shared_list(
        &mut fields,
        BlobTransactionField::AccessList,
        field_error,
    )?)
    .map_err(map_access_list_error)?;
    let max_fee_per_blob_gas = Wei::from_be_bytes(decode_shared_u256_field(
        &mut fields,
        BlobTransactionField::MaxFeePerBlobGas,
        field_error,
    )?);
    let blob_versioned_hashes = decode_blob_versioned_hashes(next_shared_list(
        &mut fields,
        BlobTransactionField::BlobVersionedHashes,
        field_error,
    )?)?;
    let y_parity = SignatureYParity::try_new(decode_shared_u64_field(
        &mut fields,
        BlobTransactionField::SignatureYParity,
        field_error,
    )?)
    .map_err(|error| BlobTransactionDecodeError::InvalidYParity {
        value: error.value(),
    })?;
    let r = decode_shared_u256_field(&mut fields, BlobTransactionField::SignatureR, field_error)?;
    let s = decode_shared_u256_field(&mut fields, BlobTransactionField::SignatureS, field_error)?;

    Ok(UnvalidatedBlobTransaction {
        chain_id,
        nonce,
        max_priority_fee_per_gas,
        max_fee_per_gas,
        gas_limit,
        to,
        value,
        input,
        access_list,
        max_fee_per_blob_gas,
        blob_versioned_hashes,
        y_parity,
        r,
        s,
    })
}

fn decode_blob_versioned_hashes(
    list: RlpList<'_>,
) -> Result<BlobVersionedHashes<'_>, BlobTransactionDecodeError> {
    for item in list.items() {
        let _ = decode_blob_versioned_hash_item(item)?;
    }
    Ok(BlobVersionedHashes { list })
}

fn decode_blob_versioned_hash_item(
    item: Result<RlpItem<'_>, DecodeError>,
) -> Result<B256, BlobTransactionDecodeError> {
    let item =
        item.map_err(|source| field_error(BlobTransactionField::BlobVersionedHashes, source))?;
    let RlpItem::Scalar(scalar) = item else {
        return Err(field_error(
            BlobTransactionField::BlobVersionedHashes,
            DecodeError::UnexpectedList,
        ));
    };
    let found = scalar.payload().len();
    let bytes: [u8; B256_BYTES] = scalar
        .payload()
        .try_into()
        .map_err(|_| BlobTransactionDecodeError::InvalidBlobVersionedHashLength { found })?;
    Ok(B256::from_bytes(bytes))
}

const fn field_error(
    field: BlobTransactionField,
    source: DecodeError,
) -> BlobTransactionDecodeError {
    BlobTransactionDecodeError::FieldDecode { field, source }
}

const fn map_access_list_error(error: AccessListDecodeError) -> BlobTransactionDecodeError {
    match error {
        AccessListDecodeError::FieldDecode(source) => {
            field_error(BlobTransactionField::AccessList, source)
        }
        AccessListDecodeError::InvalidAccessListEntryFieldCount { found } => {
            BlobTransactionDecodeError::InvalidAccessListEntryFieldCount { found }
        }
        AccessListDecodeError::InvalidAccessListAddressLength { found } => {
            BlobTransactionDecodeError::InvalidAccessListAddressLength { found }
        }
        AccessListDecodeError::InvalidStorageKeyLength { found } => {
            BlobTransactionDecodeError::InvalidStorageKeyLength { found }
        }
    }
}
