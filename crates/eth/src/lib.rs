#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use eth_valkyoth_codec as codec;
#[cfg(feature = "evm")]
pub use eth_valkyoth_evm as evm;
#[cfg(feature = "evm-core")]
pub use eth_valkyoth_evm_core as evm_core;
pub use eth_valkyoth_hash as hash;
pub use eth_valkyoth_primitives as primitives;
pub use eth_valkyoth_protocol as protocol;
#[cfg(feature = "reth")]
pub use eth_valkyoth_reth as reth;
#[cfg(feature = "rpc")]
pub use eth_valkyoth_rpc as rpc;
#[cfg(feature = "sanitization")]
pub use eth_valkyoth_sanitization as sanitization;
#[cfg(feature = "signer")]
pub use eth_valkyoth_signer as signer;
#[cfg(feature = "testkit")]
pub use eth_valkyoth_testkit as testkit;
pub use eth_valkyoth_verify as verify;

/// Stable error types re-exported by the facade crate.
pub mod error {
    pub use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory, ResourceError, RlpDeriveError};
    #[cfg(feature = "evm")]
    pub use eth_valkyoth_evm::{
        ExecutionEnvironmentError, ExecutionError, GasEstimationError, SnapshotError,
    };
    #[cfg(feature = "evm-core")]
    pub use eth_valkyoth_evm_core::EvmCoreError;
    pub use eth_valkyoth_protocol::{
        AccessListTransactionDecodeError, AccessListTransactionDecodeErrorCategory,
        BlobTransactionDecodeError, BlobTransactionDecodeErrorCategory, BlockHeaderDecodeError,
        BlockHeaderDecodeErrorCategory, DynamicFeeTransactionDecodeError,
        DynamicFeeTransactionDecodeErrorCategory, FeatureError, ForkError,
        LegacyTransactionDecodeError, LegacyTransactionDecodeErrorCategory, ProtocolError,
        ProtocolErrorCategory, ReceiptDecodeError, ReceiptDecodeErrorCategory,
        SetCodeTransactionDecodeError, SetCodeTransactionDecodeErrorCategory,
        SetCodeTransactionValidityError, SetCodeTransactionValidityErrorCategory,
        StateTransitionError, TransactionEncodeError, TransactionEnvelopeError,
        TransactionEnvelopeErrorCategory, WithdrawalDecodeError, WithdrawalDecodeErrorCategory,
    };
    #[cfg(feature = "eip712-json")]
    pub use eth_valkyoth_verify::Eip712JsonError;
    pub use eth_valkyoth_verify::{
        Eip712EncodeError, MptNodeDecodeError, MptNodeDecodeErrorCategory,
        MptProofVerificationError, MptProofVerificationErrorCategory,
        SetCodeAuthorizationValidationError, SetCodeAuthorizationValidationErrorCategory,
        TransactionSignatureValidationError, TransactionSignatureValidationErrorCategory,
        TransactionSigningHashError, VerifyError, VerifyErrorCategory,
    };
}
