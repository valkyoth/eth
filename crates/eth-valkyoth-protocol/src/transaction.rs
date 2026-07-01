//! Transaction envelope and field decoding.

mod access_list;
mod blob;
mod dynamic_fee;
mod encode;
mod envelope;
mod fields;
mod legacy;
mod set_code;
mod signing;

#[cfg(test)]
mod tests;

pub use access_list::{
    ACCESS_LIST_TRANSACTION_FIELD_COUNT, ACCESS_LIST_TRANSACTION_TYPE, AccessList,
    AccessListEntries, AccessListEntry, AccessListStorageKeyItems, AccessListStorageKeys,
    AccessListTransactionDecodeError, AccessListTransactionDecodeErrorCategory,
    AccessListTransactionField, AccessListTransactionTo, InvalidSignatureYParity, SignatureYParity,
    UnvalidatedAccessListTransaction, decode_access_list_transaction,
};
pub use blob::{
    BLOB_TRANSACTION_FIELD_COUNT, BLOB_TRANSACTION_TYPE, BlobTransactionDecodeError,
    BlobTransactionDecodeErrorCategory, BlobTransactionField, BlobVersionedHashItems,
    BlobVersionedHashes, UnvalidatedBlobTransaction, decode_blob_transaction,
};
pub use dynamic_fee::{
    DYNAMIC_FEE_TRANSACTION_FIELD_COUNT, DYNAMIC_FEE_TRANSACTION_TYPE,
    DynamicFeeTransactionDecodeError, DynamicFeeTransactionDecodeErrorCategory,
    DynamicFeeTransactionField, DynamicFeeTransactionTo, UnvalidatedDynamicFeeTransaction,
    decode_dynamic_fee_transaction,
};
pub use encode::{
    TransactionEncodeError, TransactionEncodeErrorCategory, UnvalidatedTransaction,
    encode_access_list_transaction, encode_blob_transaction, encode_dynamic_fee_transaction,
    encode_legacy_transaction, encode_set_code_transaction, encode_transaction,
    encoded_access_list_transaction_len, encoded_blob_transaction_len,
    encoded_dynamic_fee_transaction_len, encoded_legacy_transaction_len,
    encoded_set_code_transaction_len, encoded_transaction_len,
};
pub use envelope::{
    EIP_2718_MAX_TYPED_PREFIX, EIP_2718_RESERVED_PREFIX, EIP_2718_SCALAR_PREFIX_START,
    EIP_2718_TYPED_ZERO_PREFIX, LEGACY_TRANSACTION_PREFIX_START, TransactionEnvelope,
    TransactionEnvelopeError, TransactionEnvelopeErrorCategory, TypedTransactionEnvelope,
    decode_transaction_envelope,
};
pub use legacy::{
    LEGACY_TRANSACTION_FIELD_COUNT, LegacyTransactionDecodeError,
    LegacyTransactionDecodeErrorCategory, LegacyTransactionField, LegacyTransactionTo,
    UnvalidatedLegacyTransaction, decode_legacy_transaction,
};
pub use set_code::{
    SET_CODE_AUTHORIZATION_FIELD_COUNT, SET_CODE_AUTHORIZATION_MAGIC,
    SET_CODE_TRANSACTION_FIELD_COUNT, SET_CODE_TRANSACTION_TYPE, SetCodeAuthorization,
    SetCodeAuthorizationChainId, SetCodeAuthorizationItems, SetCodeAuthorizationList,
    SetCodeTransactionDecodeError, SetCodeTransactionDecodeErrorCategory, SetCodeTransactionField,
    UnvalidatedSetCodeTransaction, decode_set_code_transaction,
};
pub use signing::{
    encode_access_list_signing_preimage, encode_blob_signing_preimage,
    encode_dynamic_fee_signing_preimage, encode_legacy_eip155_signing_preimage,
    encoded_access_list_signing_preimage_len, encoded_blob_signing_preimage_len,
    encoded_dynamic_fee_signing_preimage_len, encoded_legacy_eip155_signing_preimage_len,
};
