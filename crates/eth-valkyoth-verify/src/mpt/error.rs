use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

/// MPT node field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptNodeField {
    /// Whole MPT node.
    Node,
    /// Compact hex-prefix path.
    CompactPath,
    /// Branch child reference.
    BranchChild,
    /// Branch terminal value slot.
    BranchValue,
    /// Extension child reference.
    ExtensionChild,
    /// Leaf value.
    LeafValue,
    /// Proof-node sequence accounting.
    ProofNode,
}

/// MPT node decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptNodeDecodeError {
    /// A node did not contain a valid MPT field count.
    WrongFieldCount {
        /// Actual field count.
        found: usize,
    },
    /// A field failed bounded RLP decoding.
    FieldDecode {
        /// Field being decoded.
        field: MptNodeField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// Compact hex-prefix path was empty.
    EmptyCompactPath,
    /// Compact hex-prefix flag used reserved high bits.
    InvalidCompactPathFlag {
        /// Decoded high-nibble flag.
        flag: u8,
    },
    /// Even-length compact path did not use zero padding.
    InvalidCompactPathPadding {
        /// Decoded low-nibble padding value.
        found: u8,
    },
    /// Required child reference was empty.
    EmptyNodeReference {
        /// Field being decoded.
        field: MptNodeField,
    },
    /// Scalar child reference was neither empty nor 32 bytes.
    InvalidNodeReferenceLength {
        /// Field being decoded.
        field: MptNodeField,
        /// Actual decoded scalar length.
        found: usize,
    },
    /// Inline child node was encoded at 32 bytes or more.
    InlineNodeTooLarge {
        /// Field being decoded.
        field: MptNodeField,
        /// Actual encoded child-list length.
        found: usize,
    },
    /// Length arithmetic overflowed.
    LengthOverflow,
    /// Inline child nodes exceeded the explicit decoder traversal limit.
    InlineNodeTooDeep,
}

impl MptNodeDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::WrongFieldCount { .. } => "ETH_MPT_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::EmptyCompactPath => "ETH_MPT_EMPTY_COMPACT_PATH",
            Self::InvalidCompactPathFlag { .. } => "ETH_MPT_INVALID_COMPACT_PATH_FLAG",
            Self::InvalidCompactPathPadding { .. } => "ETH_MPT_INVALID_COMPACT_PATH_PADDING",
            Self::EmptyNodeReference { .. } => "ETH_MPT_EMPTY_NODE_REFERENCE",
            Self::InvalidNodeReferenceLength { .. } => "ETH_MPT_INVALID_NODE_REFERENCE_LENGTH",
            Self::InlineNodeTooLarge { .. } => "ETH_MPT_INLINE_NODE_TOO_LARGE",
            Self::LengthOverflow => "ETH_MPT_LENGTH_OVERFLOW",
            Self::InlineNodeTooDeep => "ETH_MPT_INLINE_NODE_TOO_DEEP",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::WrongFieldCount { .. } => "MPT node must have two or seventeen fields",
            Self::FieldDecode { .. } => "MPT node field failed bounded decoding",
            Self::EmptyCompactPath => "MPT compact path must include a flag byte",
            Self::InvalidCompactPathFlag { .. } => {
                "MPT compact path uses a reserved hex-prefix flag"
            }
            Self::InvalidCompactPathPadding { .. } => {
                "MPT compact path has nonzero even-path padding"
            }
            Self::EmptyNodeReference { .. } => "MPT child reference must not be empty",
            Self::InvalidNodeReferenceLength { .. } => {
                "MPT scalar child reference must be empty or 32 bytes"
            }
            Self::InlineNodeTooLarge { .. } => {
                "MPT inline child reference must be shorter than 32 encoded bytes"
            }
            Self::LengthOverflow => "MPT length arithmetic overflowed",
            Self::InlineNodeTooDeep => "MPT inline child-node depth limit exceeded",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> MptNodeDecodeErrorCategory {
        match self {
            Self::FieldDecode { source, .. } => match source.category() {
                DecodeErrorCategory::ResourceExhaustion => {
                    MptNodeDecodeErrorCategory::ResourceExhaustion
                }
                DecodeErrorCategory::MalformedInput => MptNodeDecodeErrorCategory::MalformedInput,
                _ => MptNodeDecodeErrorCategory::MalformedInput,
            },
            Self::WrongFieldCount { .. }
            | Self::EmptyCompactPath
            | Self::InvalidCompactPathFlag { .. }
            | Self::InvalidCompactPathPadding { .. }
            | Self::EmptyNodeReference { .. }
            | Self::InvalidNodeReferenceLength { .. }
            | Self::InlineNodeTooLarge { .. }
            | Self::LengthOverflow => MptNodeDecodeErrorCategory::MalformedInput,
            Self::InlineNodeTooDeep => MptNodeDecodeErrorCategory::ResourceExhaustion,
        }
    }
}

impl fmt::Display for MptNodeDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MptNodeDecodeError {}

/// Stable high-level MPT node decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptNodeDecodeErrorCategory {
    /// Input is malformed for an MPT node or child reference.
    MalformedInput,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
