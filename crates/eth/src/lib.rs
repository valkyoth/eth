#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use eth_valkyoth_codec as codec;
#[cfg(feature = "evm")]
pub use eth_valkyoth_evm as evm;
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
    pub use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory, ResourceError};
    pub use eth_valkyoth_protocol::{
        AccessListTransactionDecodeError, AccessListTransactionDecodeErrorCategory,
        DynamicFeeTransactionDecodeError, DynamicFeeTransactionDecodeErrorCategory, FeatureError,
        ForkError, LegacyTransactionDecodeError, LegacyTransactionDecodeErrorCategory,
        ProtocolError, ProtocolErrorCategory, TransactionEnvelopeError,
        TransactionEnvelopeErrorCategory,
    };
    pub use eth_valkyoth_verify::{VerifyError, VerifyErrorCategory};
}
