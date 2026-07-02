use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

/// Header field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockHeaderField {
    /// `parent_hash`.
    ParentHash,
    /// `ommers_hash`.
    OmmersHash,
    /// `beneficiary`.
    Beneficiary,
    /// `state_root`.
    StateRoot,
    /// `transactions_root`.
    TransactionsRoot,
    /// `receipts_root`.
    ReceiptsRoot,
    /// `logs_bloom`.
    LogsBloom,
    /// `difficulty`.
    Difficulty,
    /// `number`.
    Number,
    /// `gas_limit`.
    GasLimit,
    /// `gas_used`.
    GasUsed,
    /// `timestamp`.
    Timestamp,
    /// `extra_data`.
    ExtraData,
    /// `mix_hash` / `prev_randao`.
    MixHash,
    /// `nonce`.
    Nonce,
    /// `base_fee_per_gas`.
    BaseFeePerGas,
    /// `withdrawals_root`.
    WithdrawalsRoot,
    /// `blob_gas_used`.
    BlobGasUsed,
    /// `excess_blob_gas`.
    ExcessBlobGas,
    /// `parent_beacon_block_root`.
    ParentBeaconBlockRoot,
    /// `requests_hash`.
    RequestsHash,
}

/// Execution header decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockHeaderDecodeError {
    /// Header RLP list decode failed.
    Decode(DecodeError),
    /// Header field count did not match the selected fork field set.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A field failed RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: BlockHeaderField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// A fixed-width field had the wrong byte length.
    InvalidFieldLength {
        /// Field being decoded.
        field: BlockHeaderField,
        /// Expected byte length.
        expected: usize,
        /// Actual decoded scalar byte length.
        found: usize,
    },
}

impl BlockHeaderDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Decode(error) => error.code(),
            Self::WrongFieldCount { .. } => "ETH_HEADER_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::InvalidFieldLength { .. } => "ETH_HEADER_INVALID_FIELD_LENGTH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Decode(error) => error.message(),
            Self::WrongFieldCount { .. } => {
                "block header field count does not match the selected field set"
            }
            Self::FieldDecode { .. } => "block header field failed bounded decoding",
            Self::InvalidFieldLength { .. } => {
                "block header fixed-width field has an invalid byte length"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> BlockHeaderDecodeErrorCategory {
        match self {
            Self::WrongFieldCount { .. } | Self::InvalidFieldLength { .. } => {
                BlockHeaderDecodeErrorCategory::MalformedInput
            }
            Self::Decode(error) | Self::FieldDecode { source: error, .. } => {
                match error.category() {
                    DecodeErrorCategory::ResourceExhaustion => {
                        BlockHeaderDecodeErrorCategory::ResourceExhaustion
                    }
                    _ => BlockHeaderDecodeErrorCategory::MalformedInput,
                }
            }
        }
    }
}

impl fmt::Display for BlockHeaderDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BlockHeaderDecodeError {}

/// Stable high-level header decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockHeaderDecodeErrorCategory {
    /// Input is malformed for the selected header field set.
    MalformedInput,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
