use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeSession, DecodeSessionPolicy};
use eth_valkyoth_primitives::{ChainId, TransactionType};
use eth_valkyoth_protocol::{
    ACCESS_LIST_TRANSACTION_TYPE, BLOB_TRANSACTION_TYPE, DYNAMIC_FEE_TRANSACTION_TYPE, Hardfork,
    SET_CODE_TRANSACTION_TYPE, TransactionEnvelope, TransactionEnvelopeError,
    UnvalidatedAccessListTransaction, UnvalidatedBlobTransaction, UnvalidatedDynamicFeeTransaction,
    UnvalidatedLegacyTransaction, UnvalidatedSetCodeTransaction,
    decode_access_list_transaction_in_session, decode_blob_transaction_in_session,
    decode_dynamic_fee_transaction_in_session, decode_legacy_transaction_in_session,
    decode_set_code_transaction_in_session, decode_transaction_envelope,
};

use crate::ExecutionEnvironment;

/// EIP-2718 or legacy envelope admitted for type-specific execution decoding.
///
/// This token cannot be constructed from fields. Classification rejects empty
/// typed payloads and typed domains without a first-party canonical decoder.
#[derive(Debug)]
pub struct ClassifiedEnvelope<'a> {
    raw: &'a [u8],
    envelope: TransactionEnvelope<'a>,
    session: DecodeSession,
}

impl<'a> ClassifiedEnvelope<'a> {
    /// Classifies raw transaction bytes under one complete decode policy.
    pub fn decode(
        raw: &'a [u8],
        policy: DecodeSessionPolicy,
    ) -> Result<Self, ExecutionAdmissionError> {
        let session = DecodeSession::new(policy).map_err(ExecutionAdmissionError::DecodePolicy)?;
        let envelope = decode_transaction_envelope(raw, policy.limits())
            .map_err(ExecutionAdmissionError::Envelope)?;
        if let TransactionEnvelope::Typed(typed) = envelope {
            let type_byte = typed.transaction_type.get();
            if typed.payload.is_empty() {
                return Err(ExecutionAdmissionError::EmptyTypedPayload { type_byte });
            }
            if !is_supported_type(type_byte) {
                return Err(ExecutionAdmissionError::UnsupportedTransactionType { type_byte });
            }
        }
        Ok(Self {
            raw,
            envelope,
            session,
        })
    }

    /// Exact raw transaction bytes represented by this token.
    #[must_use]
    pub const fn raw(&self) -> &'a [u8] {
        self.raw
    }

    /// Classified outer envelope.
    #[must_use]
    pub const fn envelope(&self) -> TransactionEnvelope<'a> {
        self.envelope
    }

    /// Runs the type-specific canonical decoder under the conserved session.
    #[allow(clippy::result_large_err)] // Returning the non-copyable token avoids alloc and lost work evidence.
    pub fn try_into_canonical(
        self,
    ) -> Result<CanonicallyDecodedTransaction<'a>, ExecutionAdmissionFailure<Self>> {
        let Self {
            raw,
            envelope,
            mut session,
        } = self;
        let transaction_type = envelope_type(envelope);
        let decoded = decode_canonical(raw, transaction_type, &mut session);
        match decoded {
            Ok(transaction) => Ok(CanonicallyDecodedTransaction {
                raw,
                transaction_type,
                transaction,
                session,
            }),
            Err(error) => Err(ExecutionAdmissionFailure {
                candidate: Self {
                    raw,
                    envelope,
                    session,
                },
                error,
            }),
        }
    }
}

/// Canonically decoded transaction fields admitted by the first-party codec.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalTransaction<'a> {
    /// Legacy transaction fields.
    Legacy(UnvalidatedLegacyTransaction<'a>),
    /// EIP-2930 access-list transaction fields.
    AccessList(UnvalidatedAccessListTransaction<'a>),
    /// EIP-1559 dynamic-fee transaction fields.
    DynamicFee(UnvalidatedDynamicFeeTransaction<'a>),
    /// EIP-4844 blob transaction fields.
    Blob(UnvalidatedBlobTransaction<'a>),
    /// EIP-7702 set-code transaction fields.
    SetCode(UnvalidatedSetCodeTransaction<'a>),
}

impl CanonicalTransaction<'_> {
    fn signed_chain_id(&self) -> Option<ChainId> {
        match self {
            Self::Legacy(transaction) => transaction.eip155_chain_id(),
            Self::AccessList(transaction) => Some(transaction.chain_id),
            Self::DynamicFee(transaction) => Some(transaction.chain_id),
            Self::Blob(transaction) => Some(transaction.chain_id),
            Self::SetCode(transaction) => Some(transaction.chain_id),
        }
    }
}

/// Transaction whose complete type-specific RLP field structure is canonical.
#[derive(Debug)]
pub struct CanonicallyDecodedTransaction<'a> {
    raw: &'a [u8],
    transaction_type: TransactionType,
    transaction: CanonicalTransaction<'a>,
    session: DecodeSession,
}

impl<'a> CanonicallyDecodedTransaction<'a> {
    /// Exact raw bytes accepted by the canonical decoder.
    #[must_use]
    pub const fn raw(&self) -> &'a [u8] {
        self.raw
    }

    /// Canonically decoded borrowed transaction fields.
    #[must_use]
    pub const fn transaction(&self) -> &CanonicalTransaction<'a> {
        &self.transaction
    }

    /// Legacy or typed transaction domain.
    #[must_use]
    pub const fn transaction_type(&self) -> TransactionType {
        self.transaction_type
    }

    /// Conserved decode session containing all canonical decode work.
    #[must_use]
    pub const fn decode_session(&self) -> &DecodeSession {
        &self.session
    }

    /// Checks transaction-type activation and signed chain binding.
    ///
    /// This stage does not claim sender recovery, intrinsic gas, account
    /// state, fee, blob/KZG, or authorization validity.
    #[allow(clippy::result_large_err)] // The fail-closed no_std API returns the original typestate token.
    pub fn try_into_fork_validated(
        self,
        environment: ExecutionEnvironment,
    ) -> Result<ForkValidatedTransaction<'a>, ExecutionAdmissionFailure<Self>> {
        let active = environment.fork().fork.hardfork;
        let required = minimum_fork(&self.transaction);
        if active < required {
            let type_byte = self.transaction_type.get();
            return Err(ExecutionAdmissionFailure {
                candidate: self,
                error: ExecutionAdmissionError::TransactionTypeNotActive {
                    type_byte,
                    required,
                    active,
                },
            });
        }
        let expected_chain_id = environment.block().chain_id;
        if let Some(found) = self.transaction.signed_chain_id()
            && found != expected_chain_id
        {
            return Err(ExecutionAdmissionFailure {
                candidate: self,
                error: ExecutionAdmissionError::ChainMismatch {
                    expected: expected_chain_id,
                    found,
                },
            });
        }
        Ok(ForkValidatedTransaction {
            canonical: self,
            environment,
        })
    }
}

/// Canonical transaction admitted for its active fork and signed chain.
#[derive(Debug)]
pub struct ForkValidatedTransaction<'a> {
    canonical: CanonicallyDecodedTransaction<'a>,
    environment: ExecutionEnvironment,
}

impl<'a> ForkValidatedTransaction<'a> {
    /// Promotes the fork-bound token to the only request-admissible state.
    #[must_use]
    pub fn into_execution_ready(self) -> ExecutionReadyTransaction<'a> {
        ExecutionReadyTransaction {
            canonical: self.canonical,
            environment: self.environment,
        }
    }
}

/// Non-forgeable transaction token accepted by [`crate::ExecutionRequest`].
#[derive(Debug)]
pub struct ExecutionReadyTransaction<'a> {
    canonical: CanonicallyDecodedTransaction<'a>,
    environment: ExecutionEnvironment,
}

impl<'a> ExecutionReadyTransaction<'a> {
    /// Exact canonical raw transaction bytes.
    #[must_use]
    pub const fn raw(&self) -> &'a [u8] {
        self.canonical.raw()
    }

    /// Legacy or typed transaction domain.
    #[must_use]
    pub const fn transaction_type(&self) -> TransactionType {
        self.canonical.transaction_type()
    }

    /// Fork and block environment used during admission.
    #[must_use]
    pub const fn environment(&self) -> ExecutionEnvironment {
        self.environment
    }

    /// Canonically decoded borrowed transaction fields.
    #[must_use]
    pub const fn transaction(&self) -> &CanonicalTransaction<'a> {
        self.canonical.transaction()
    }

    /// Conserved work evidence from canonical decoding.
    #[must_use]
    pub const fn decode_session(&self) -> &DecodeSession {
        self.canonical.decode_session()
    }
}

/// Failed promotion that returns the non-copyable candidate token.
#[derive(Debug)]
pub struct ExecutionAdmissionFailure<T> {
    candidate: T,
    error: ExecutionAdmissionError,
}

impl<T> ExecutionAdmissionFailure<T> {
    /// Admission error that prevented promotion.
    #[must_use]
    pub const fn error(&self) -> ExecutionAdmissionError {
        self.error
    }

    /// Splits the failure into its original token and error.
    #[must_use]
    pub fn into_parts(self) -> (T, ExecutionAdmissionError) {
        (self.candidate, self.error)
    }
}

/// Execution transaction admission failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionAdmissionError {
    /// Decode-session policy is inconsistent.
    DecodePolicy(DecodeError),
    /// Outer envelope classification failed.
    Envelope(TransactionEnvelopeError),
    /// A typed envelope contains no payload bytes.
    EmptyTypedPayload {
        /// Observed typed transaction byte.
        type_byte: u8,
    },
    /// No first-party canonical decoder is admitted for this typed domain.
    UnsupportedTransactionType {
        /// Observed typed transaction byte.
        type_byte: u8,
    },
    /// Type-specific canonical field decoding failed.
    CanonicalDecode {
        /// Legacy zero or typed transaction byte.
        type_byte: u8,
        /// Stable source error code.
        source_code: &'static str,
    },
    /// Transaction type is not active under the selected fork.
    TransactionTypeNotActive {
        /// Legacy zero or typed transaction byte.
        type_byte: u8,
        /// First modeled fork that admits this type.
        required: Hardfork,
        /// Selected active fork.
        active: Hardfork,
    },
    /// Signed transaction chain differs from the execution environment.
    ChainMismatch {
        /// Environment chain.
        expected: ChainId,
        /// Transaction chain.
        found: ChainId,
    },
}

impl ExecutionAdmissionError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DecodePolicy(error) => error.code(),
            Self::Envelope(error) => error.code(),
            Self::EmptyTypedPayload { .. } => "ETH_EVM_ADMISSION_EMPTY_TYPED_PAYLOAD",
            Self::UnsupportedTransactionType { .. } => "ETH_EVM_ADMISSION_UNSUPPORTED_TYPE",
            Self::CanonicalDecode { source_code, .. } => source_code,
            Self::TransactionTypeNotActive { .. } => "ETH_EVM_ADMISSION_TYPE_NOT_ACTIVE",
            Self::ChainMismatch { .. } => "ETH_EVM_ADMISSION_CHAIN_MISMATCH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::DecodePolicy(_) => "execution admission decode policy is invalid",
            Self::Envelope(error) => error.message(),
            Self::EmptyTypedPayload { .. } => "typed execution envelope payload is empty",
            Self::UnsupportedTransactionType { .. } => {
                "typed transaction has no admitted execution decoder"
            }
            Self::CanonicalDecode { .. } => {
                "transaction failed canonical type-specific field decoding"
            }
            Self::TransactionTypeNotActive { .. } => {
                "transaction type is not active for the selected fork"
            }
            Self::ChainMismatch { .. } => {
                "transaction signed chain does not match execution environment"
            }
        }
    }
}

impl fmt::Display for ExecutionAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ExecutionAdmissionError {}

const fn is_supported_type(type_byte: u8) -> bool {
    matches!(
        type_byte,
        ACCESS_LIST_TRANSACTION_TYPE
            | DYNAMIC_FEE_TRANSACTION_TYPE
            | BLOB_TRANSACTION_TYPE
            | SET_CODE_TRANSACTION_TYPE
    )
}

const fn envelope_type(envelope: TransactionEnvelope<'_>) -> TransactionType {
    match envelope {
        TransactionEnvelope::Legacy(_) => TransactionType::LEGACY,
        TransactionEnvelope::Typed(typed) => typed.transaction_type,
    }
}

fn decode_canonical<'a>(
    raw: &'a [u8],
    transaction_type: TransactionType,
    session: &mut DecodeSession,
) -> Result<CanonicalTransaction<'a>, ExecutionAdmissionError> {
    let type_byte = transaction_type.get();
    let source = match type_byte {
        0 => decode_legacy_transaction_in_session(raw, session)
            .map(CanonicalTransaction::Legacy)
            .map_err(|error| error.code()),
        ACCESS_LIST_TRANSACTION_TYPE => decode_access_list_transaction_in_session(raw, session)
            .map(CanonicalTransaction::AccessList)
            .map_err(|error| error.code()),
        DYNAMIC_FEE_TRANSACTION_TYPE => decode_dynamic_fee_transaction_in_session(raw, session)
            .map(CanonicalTransaction::DynamicFee)
            .map_err(|error| error.code()),
        BLOB_TRANSACTION_TYPE => decode_blob_transaction_in_session(raw, session)
            .map(CanonicalTransaction::Blob)
            .map_err(|error| error.code()),
        SET_CODE_TRANSACTION_TYPE => decode_set_code_transaction_in_session(raw, session)
            .map(CanonicalTransaction::SetCode)
            .map_err(|error| error.code()),
        _ => return Err(ExecutionAdmissionError::UnsupportedTransactionType { type_byte }),
    };
    source.map_err(|source_code| ExecutionAdmissionError::CanonicalDecode {
        type_byte,
        source_code,
    })
}

const fn minimum_fork(transaction: &CanonicalTransaction<'_>) -> Hardfork {
    match transaction {
        CanonicalTransaction::Legacy(_) => Hardfork::Frontier,
        // The current Hardfork domain does not yet model Berlin separately.
        CanonicalTransaction::AccessList(_) | CanonicalTransaction::DynamicFee(_) => {
            Hardfork::London
        }
        CanonicalTransaction::Blob(_) => Hardfork::Cancun,
        CanonicalTransaction::SetCode(_) => Hardfork::Prague,
    }
}
