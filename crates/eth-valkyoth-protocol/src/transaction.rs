//! Transaction envelope and field decoding.

mod access_list;
mod dynamic_fee;
mod envelope;
mod fields;
mod legacy;

#[cfg(test)]
mod tests;

pub use access_list::{
    ACCESS_LIST_TRANSACTION_FIELD_COUNT, AccessList, AccessListEntries, AccessListEntry,
    AccessListStorageKeyItems, AccessListStorageKeys, AccessListTransactionDecodeError,
    AccessListTransactionDecodeErrorCategory, AccessListTransactionField, AccessListTransactionTo,
    SignatureYParity, UnvalidatedAccessListTransaction, decode_access_list_transaction,
};
pub use dynamic_fee::{
    DYNAMIC_FEE_TRANSACTION_FIELD_COUNT, DynamicFeeTransactionDecodeError,
    DynamicFeeTransactionDecodeErrorCategory, DynamicFeeTransactionField, DynamicFeeTransactionTo,
    UnvalidatedDynamicFeeTransaction, decode_dynamic_fee_transaction,
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
