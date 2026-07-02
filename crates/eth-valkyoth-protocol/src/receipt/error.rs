use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

/// Receipt field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptField {
    /// Whole typed-receipt payload.
    Payload,
    /// Status code or pre-Byzantium state root.
    StatusOrStateRoot,
    /// Cumulative gas used.
    CumulativeGasUsed,
    /// Logs bloom.
    LogsBloom,
    /// Logs list.
    Logs,
    /// Log address.
    LogAddress,
    /// Log topics.
    LogTopics,
    /// Log data.
    LogData,
}

/// Receipt decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptDecodeError {
    /// No bytes were supplied.
    EmptyInput,
    /// The first byte is the legacy-zero typed prefix.
    UnsupportedReceiptType {
        /// Unsupported typed receipt prefix byte.
        type_byte: u8,
    },
    /// The first byte is an RLP scalar prefix, not a receipt envelope.
    ScalarPrefix {
        /// Observed scalar prefix byte.
        prefix: u8,
    },
    /// The first byte is the EIP-2718 reserved extension sentinel.
    ReservedPrefix,
    /// Receipt list did not contain exactly four fields.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A log entry did not contain exactly three fields.
    InvalidLogFieldCount {
        /// Actual field count.
        found: usize,
    },
    /// A field failed bounded RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: ReceiptField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// Status/root field was neither status `0`/`1` nor a 32-byte root.
    InvalidStatusOrStateRoot {
        /// Actual decoded scalar byte length.
        found: usize,
    },
    /// Logs bloom was not exactly 256 bytes.
    InvalidLogsBloomLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
    /// Log address was not exactly 20 bytes.
    InvalidLogAddressLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
    /// Log topic was not exactly 32 bytes.
    InvalidLogTopicLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
}

impl ReceiptDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyInput => "ETH_RECEIPT_EMPTY_INPUT",
            Self::UnsupportedReceiptType { .. } => "ETH_RECEIPT_UNSUPPORTED_TYPE",
            Self::ScalarPrefix { .. } => "ETH_RECEIPT_SCALAR_PREFIX",
            Self::ReservedPrefix => "ETH_RECEIPT_RESERVED_PREFIX",
            Self::WrongFieldCount { .. } => "ETH_RECEIPT_WRONG_FIELD_COUNT",
            Self::InvalidLogFieldCount { .. } => "ETH_RECEIPT_INVALID_LOG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::InvalidStatusOrStateRoot { .. } => "ETH_RECEIPT_INVALID_STATUS_OR_ROOT",
            Self::InvalidLogsBloomLength { .. } => "ETH_RECEIPT_INVALID_BLOOM_LENGTH",
            Self::InvalidLogAddressLength { .. } => "ETH_RECEIPT_INVALID_LOG_ADDRESS_LENGTH",
            Self::InvalidLogTopicLength { .. } => "ETH_RECEIPT_INVALID_LOG_TOPIC_LENGTH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::EmptyInput => "receipt input is empty",
            Self::UnsupportedReceiptType { .. } => {
                "receipt type is not supported by this receipt decoder"
            }
            Self::ScalarPrefix { .. } => "receipt envelope starts with an RLP scalar prefix",
            Self::ReservedPrefix => "receipt envelope starts with the reserved 0xff prefix",
            Self::WrongFieldCount { .. } => "receipt must contain exactly four RLP fields",
            Self::InvalidLogFieldCount { .. } => "receipt log must contain exactly three fields",
            Self::FieldDecode { .. } => "receipt field failed bounded decoding",
            Self::InvalidStatusOrStateRoot { .. } => {
                "receipt status/root field must be status 0 or 1, or a 32-byte root"
            }
            Self::InvalidLogsBloomLength { .. } => "receipt logs bloom must be exactly 256 bytes",
            Self::InvalidLogAddressLength { .. } => "receipt log address must be exactly 20 bytes",
            Self::InvalidLogTopicLength { .. } => "receipt log topic must be exactly 32 bytes",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> ReceiptDecodeErrorCategory {
        match self {
            Self::UnsupportedReceiptType { .. } => ReceiptDecodeErrorCategory::Unsupported,
            Self::EmptyInput
            | Self::ScalarPrefix { .. }
            | Self::ReservedPrefix
            | Self::WrongFieldCount { .. }
            | Self::InvalidLogFieldCount { .. }
            | Self::InvalidStatusOrStateRoot { .. }
            | Self::InvalidLogsBloomLength { .. }
            | Self::InvalidLogAddressLength { .. }
            | Self::InvalidLogTopicLength { .. } => ReceiptDecodeErrorCategory::MalformedInput,
            Self::FieldDecode { source, .. } => match source.category() {
                DecodeErrorCategory::MalformedInput => ReceiptDecodeErrorCategory::MalformedInput,
                DecodeErrorCategory::ResourceExhaustion => {
                    ReceiptDecodeErrorCategory::ResourceExhaustion
                }
                _ => ReceiptDecodeErrorCategory::MalformedInput,
            },
        }
    }
}

impl fmt::Display for ReceiptDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ReceiptDecodeError {}

/// Stable high-level receipt decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptDecodeErrorCategory {
    /// Input is malformed for a receipt envelope or payload.
    MalformedInput,
    /// A future or unsupported receipt domain was encountered.
    Unsupported,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
