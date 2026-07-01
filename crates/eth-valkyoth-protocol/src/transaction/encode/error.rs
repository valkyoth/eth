use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

/// Transaction envelope encoding failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionEncodeError {
    /// The underlying bounded RLP codec rejected a field or output buffer.
    Codec(DecodeError),
}

impl TransactionEncodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Codec(error) => error.code(),
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Codec(error) => error.message(),
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> TransactionEncodeErrorCategory {
        match self {
            Self::Codec(error) => match error.category() {
                DecodeErrorCategory::MalformedInput => {
                    TransactionEncodeErrorCategory::MalformedInput
                }
                DecodeErrorCategory::ResourceExhaustion => {
                    TransactionEncodeErrorCategory::ResourceExhaustion
                }
                _ => TransactionEncodeErrorCategory::MalformedInput,
            },
        }
    }
}

impl From<DecodeError> for TransactionEncodeError {
    fn from(error: DecodeError) -> Self {
        Self::Codec(error)
    }
}

impl fmt::Display for TransactionEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionEncodeError {}

/// Stable high-level transaction encode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionEncodeErrorCategory {
    /// A field, length, or output buffer is malformed for canonical encoding.
    MalformedInput,
    /// A bounded codec policy rejected an encoded list payload.
    ResourceExhaustion,
}
