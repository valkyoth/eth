use core::fmt;

use eth_valkyoth_codec::{
    DecodeError, DecodeErrorCategory, DecodeLimits, DecodeSession, RlpList, decode_rlp_list,
    decode_rlp_list_in_session,
};
use eth_valkyoth_primitives::TransactionType;

use crate::eip2718::{
    EIP_2718_MAX_TYPED_PREFIX as SHARED_EIP_2718_MAX_TYPED_PREFIX,
    EIP_2718_RESERVED_PREFIX as SHARED_EIP_2718_RESERVED_PREFIX,
    EIP_2718_SCALAR_PREFIX_START as SHARED_EIP_2718_SCALAR_PREFIX_START,
    EIP_2718_TYPED_ZERO_PREFIX as SHARED_EIP_2718_TYPED_ZERO_PREFIX, Eip2718Prefix,
    LEGACY_PREFIX_START as SHARED_LEGACY_PREFIX_START, classify_eip2718_prefix,
};

/// EIP-2718 byte value reserved by this crate for the legacy transaction
/// domain.
pub const EIP_2718_TYPED_ZERO_PREFIX: u8 = SHARED_EIP_2718_TYPED_ZERO_PREFIX;

/// Largest single-byte EIP-2718 typed transaction prefix.
pub const EIP_2718_MAX_TYPED_PREFIX: u8 = SHARED_EIP_2718_MAX_TYPED_PREFIX;

/// First RLP scalar prefix that cannot be a typed transaction envelope or a
/// legacy transaction list.
pub const EIP_2718_SCALAR_PREFIX_START: u8 = SHARED_EIP_2718_SCALAR_PREFIX_START;

/// First byte used by canonical RLP short-list legacy transactions.
pub const LEGACY_TRANSACTION_PREFIX_START: u8 = SHARED_LEGACY_PREFIX_START;

/// Prefix reserved by EIP-2718 as a future extension sentinel.
pub const EIP_2718_RESERVED_PREFIX: u8 = SHARED_EIP_2718_RESERVED_PREFIX;

/// Borrowed transaction envelope shell.
///
/// This type classifies the outer transaction envelope only. It does not decode
/// legacy fields, typed transaction payloads, signatures, sender addresses, or
/// fork validity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionEnvelope<'a> {
    /// A legacy transaction represented as one exact RLP list.
    Legacy(RlpList<'a>),
    /// An EIP-2718 typed transaction with an opaque payload.
    Typed(TypedTransactionEnvelope<'a>),
}

/// Borrowed EIP-2718 typed transaction shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypedTransactionEnvelope<'a> {
    /// EIP-2718 transaction type byte.
    pub transaction_type: TransactionType,
    /// Opaque transaction payload. Later milestones decode this by type.
    pub payload: &'a [u8],
}

/// Transaction envelope classification failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionEnvelopeError {
    /// No bytes were supplied.
    EmptyInput,
    /// The first byte is a typed-prefix value this release does not admit.
    ///
    /// The envelope shell intentionally accepts every nonzero typed prefix in
    /// `0x01..=0x7f` as opaque data. This error is reserved for prefix values
    /// outside that admitted shell domain, such as legacy zero if it reaches
    /// typed-prefix handling.
    UnsupportedTransactionType {
        /// Unsupported typed transaction prefix byte.
        type_byte: u8,
    },
    /// The first byte is an RLP scalar prefix, not a transaction envelope.
    ScalarPrefix {
        /// Observed scalar prefix byte.
        prefix: u8,
    },
    /// The first byte is the EIP-2718 reserved extension sentinel.
    ReservedPrefix,
    /// The legacy RLP list shell failed exact bounded decoding.
    Decode(DecodeError),
}

impl TransactionEnvelopeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyInput => "ETH_TX_ENVELOPE_EMPTY_INPUT",
            Self::UnsupportedTransactionType { .. } => "ETH_TX_ENVELOPE_UNSUPPORTED_TYPE",
            Self::ScalarPrefix { .. } => "ETH_TX_ENVELOPE_SCALAR_PREFIX",
            Self::ReservedPrefix => "ETH_TX_ENVELOPE_RESERVED_PREFIX",
            Self::Decode(error) => error.code(),
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::EmptyInput => "transaction envelope input is empty",
            Self::UnsupportedTransactionType { .. } => {
                "transaction type is not supported by this envelope shell"
            }
            Self::ScalarPrefix { .. } => "transaction envelope starts with an RLP scalar prefix",
            Self::ReservedPrefix => "transaction envelope starts with the reserved 0xff prefix",
            Self::Decode(error) => error.message(),
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> TransactionEnvelopeErrorCategory {
        match self {
            Self::EmptyInput
            | Self::ScalarPrefix { .. }
            | Self::ReservedPrefix
            | Self::Decode(
                DecodeError::TrailingBytes
                | DecodeError::DecoderOverread
                | DecodeError::Malformed
                | DecodeError::UnexpectedList
                | DecodeError::UnexpectedScalar
                | DecodeError::LengthOverflow
                | DecodeError::OffsetOutOfBounds,
            ) => TransactionEnvelopeErrorCategory::MalformedInput,
            Self::UnsupportedTransactionType { .. } => {
                TransactionEnvelopeErrorCategory::Unsupported
            }
            Self::Decode(error) => match error.category() {
                DecodeErrorCategory::MalformedInput => {
                    TransactionEnvelopeErrorCategory::MalformedInput
                }
                DecodeErrorCategory::ResourceExhaustion => {
                    TransactionEnvelopeErrorCategory::ResourceExhaustion
                }
                _ => TransactionEnvelopeErrorCategory::MalformedInput,
            },
        }
    }
}

impl fmt::Display for TransactionEnvelopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionEnvelopeError {}

/// Stable high-level transaction envelope error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionEnvelopeErrorCategory {
    /// Input is malformed for an EIP-2718 or legacy transaction envelope.
    MalformedInput,
    /// A typed transaction prefix is intentionally unsupported by this shell.
    Unsupported,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}

/// Classifies a borrowed transaction envelope under explicit decode limits.
///
/// Typed transaction payloads are opaque in this release, but the full input is
/// still checked against `limits.max_input_bytes`. Legacy transactions are
/// required to be exactly one RLP list; field decoding is deferred to later
/// milestones.
pub fn decode_transaction_envelope<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<TransactionEnvelope<'a>, TransactionEnvelopeError> {
    limits
        .check_input_len(input.len())
        .map_err(TransactionEnvelopeError::Decode)?;

    let Some(prefix) = classify_eip2718_prefix(input) else {
        return Err(TransactionEnvelopeError::EmptyInput);
    };

    match prefix {
        Eip2718Prefix::TypedZero => Err(TransactionEnvelopeError::UnsupportedTransactionType {
            type_byte: EIP_2718_TYPED_ZERO_PREFIX,
        }),
        Eip2718Prefix::Typed { type_byte, payload } => {
            let transaction_type = TransactionType::try_new_typed(type_byte)
                .map_err(|_| TransactionEnvelopeError::UnsupportedTransactionType { type_byte })?;
            Ok(TransactionEnvelope::Typed(TypedTransactionEnvelope {
                transaction_type,
                payload,
            }))
        }
        Eip2718Prefix::ScalarPrefix { prefix } => {
            Err(TransactionEnvelopeError::ScalarPrefix { prefix })
        }
        Eip2718Prefix::Legacy => {
            let list = decode_rlp_list(input, limits).map_err(TransactionEnvelopeError::Decode)?;
            Ok(TransactionEnvelope::Legacy(list))
        }
        Eip2718Prefix::Reserved => Err(TransactionEnvelopeError::ReservedPrefix),
    }
}

/// Classifies a transaction envelope under one shared decode session.
///
/// Typed-envelope classification charges its one-byte type prefix. The typed
/// payload remains borrowed for the transaction-specific decoder, which must
/// charge that payload through the same session.
pub fn decode_transaction_envelope_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<TransactionEnvelope<'a>, TransactionEnvelopeError> {
    session
        .check_input_len(input.len())
        .map_err(TransactionEnvelopeError::Decode)?;

    let Some(prefix) = classify_eip2718_prefix(input) else {
        return Err(TransactionEnvelopeError::EmptyInput);
    };
    if !matches!(prefix, Eip2718Prefix::Legacy) {
        session
            .account_encoded_bytes(1)
            .map_err(TransactionEnvelopeError::Decode)?;
    }

    match prefix {
        Eip2718Prefix::TypedZero => Err(TransactionEnvelopeError::UnsupportedTransactionType {
            type_byte: EIP_2718_TYPED_ZERO_PREFIX,
        }),
        Eip2718Prefix::Typed { type_byte, payload } => {
            let transaction_type = TransactionType::try_new_typed(type_byte)
                .map_err(|_| TransactionEnvelopeError::UnsupportedTransactionType { type_byte })?;
            Ok(TransactionEnvelope::Typed(TypedTransactionEnvelope {
                transaction_type,
                payload,
            }))
        }
        Eip2718Prefix::ScalarPrefix { prefix } => {
            Err(TransactionEnvelopeError::ScalarPrefix { prefix })
        }
        Eip2718Prefix::Legacy => {
            let list = decode_rlp_list_in_session(input, session)
                .map_err(TransactionEnvelopeError::Decode)?;
            Ok(TransactionEnvelope::Legacy(list))
        }
        Eip2718Prefix::Reserved => Err(TransactionEnvelopeError::ReservedPrefix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::string::ToString;

    const TEST_LIMITS: DecodeLimits = DecodeLimits {
        max_input_bytes: 64,
        max_list_items: 16,
        max_nesting_depth: 8,
        max_total_allocation: 64,
        max_proof_nodes: 4,
        max_total_items: 32,
    };

    #[test]
    fn classifies_typed_transaction_without_decoding_payload() {
        let envelope = decode_transaction_envelope(&[0x02, 0xc0, 0x01], TEST_LIMITS);

        assert!(matches!(envelope, Ok(TransactionEnvelope::Typed(_))));
        if let Ok(TransactionEnvelope::Typed(typed)) = envelope {
            assert_eq!(u8::from(typed.transaction_type), 0x02);
            assert_eq!(typed.payload, &[0xc0, 0x01]);
        }
    }

    #[test]
    fn classifies_legacy_transaction_as_exact_rlp_list() {
        let envelope = decode_transaction_envelope(&[0xc0], TEST_LIMITS);

        assert!(matches!(envelope, Ok(TransactionEnvelope::Legacy(_))));
        if let Ok(TransactionEnvelope::Legacy(list)) = envelope {
            assert_eq!(list.item_count(), 0);
        }
    }

    #[test]
    fn rejects_legacy_transaction_with_trailing_bytes() {
        let envelope = decode_transaction_envelope(&[0xc0, 0x80], TEST_LIMITS);

        assert_eq!(
            envelope,
            Err(TransactionEnvelopeError::Decode(DecodeError::TrailingBytes))
        );
    }

    #[test]
    fn rejects_empty_input_before_prefix_classification() {
        assert_eq!(
            decode_transaction_envelope(&[], TEST_LIMITS),
            Err(TransactionEnvelopeError::EmptyInput)
        );
    }

    #[test]
    fn rejects_typed_zero_prefix_as_unsupported() {
        assert_eq!(
            decode_transaction_envelope(&[0x00], TEST_LIMITS),
            Err(TransactionEnvelopeError::UnsupportedTransactionType { type_byte: 0 })
        );
    }

    #[test]
    fn rejects_rlp_scalar_prefixes() {
        assert_eq!(
            decode_transaction_envelope(&[0x80], TEST_LIMITS),
            Err(TransactionEnvelopeError::ScalarPrefix { prefix: 0x80 })
        );
    }

    #[test]
    fn rejects_reserved_extension_prefix() {
        assert_eq!(
            decode_transaction_envelope(&[0xff], TEST_LIMITS),
            Err(TransactionEnvelopeError::ReservedPrefix)
        );
    }

    #[test]
    fn enforces_input_budget_for_typed_payloads() {
        let limits = DecodeLimits {
            max_input_bytes: 1,
            ..TEST_LIMITS
        };

        assert_eq!(
            decode_transaction_envelope(&[0x02, 0x01], limits),
            Err(TransactionEnvelopeError::Decode(DecodeError::InputTooLarge))
        );
    }

    #[test]
    fn envelope_errors_have_stable_codes_and_messages() {
        let error = TransactionEnvelopeError::ReservedPrefix;

        assert_eq!(error.code(), "ETH_TX_ENVELOPE_RESERVED_PREFIX");
        assert_eq!(
            error.message(),
            "transaction envelope starts with the reserved 0xff prefix"
        );
        assert_eq!(
            error.category(),
            TransactionEnvelopeErrorCategory::MalformedInput
        );
        assert_eq!(
            error.to_string(),
            "transaction envelope starts with the reserved 0xff prefix"
        );
    }
}
