use core::fmt;

/// Shared decode failure categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The byte input is larger than the active decode budget.
    InputTooLarge,
    /// The input contains trailing bytes after a decoded value.
    TrailingBytes,
    /// A decoder reported consuming more bytes than the input contains.
    DecoderOverread,
    /// The input is malformed for the selected wire format.
    Malformed,
    /// An RLP list was encountered where a scalar byte string was required.
    UnexpectedList,
    /// An RLP scalar was encountered where a list was required.
    UnexpectedScalar,
    /// A decoded list contains more items than the active list budget.
    ListTooLong,
    /// Decoding exceeded the active nesting-depth budget.
    NestingTooDeep,
    /// A decoder requested allocation beyond the active allocation budget.
    AllocationExceeded,
    /// A proof contains more nodes than the active proof-node budget.
    ProofTooLarge,
    /// A decoder visited more items than the active cumulative item budget.
    ItemCountExceeded,
    /// A decode session scanned more encoded bytes than its work policy permits.
    EncodedBytesExceeded,
    /// A decode session visited more RLP headers than its work policy permits.
    RlpHeaderCountExceeded,
    /// A decode session requested more hashes than its work policy permits.
    HashCountExceeded,
    /// A decode session hashed more bytes than its work policy permits.
    HashBytesExceeded,
    /// A decode session inspected more trie path nibbles than permitted.
    NibbleCountExceeded,
    /// A decode session exposed or compared more trie value bytes than permitted.
    ValueBytesExceeded,
    /// A decode session exceeded its aggregate work budget.
    WorkExceeded,
    /// Decode-session limits have inconsistent cross-limit relationships.
    InvalidSessionPolicy,
    /// A deployment used an unchanged starter budget policy.
    UnreviewedDeploymentPolicy,
    /// Length or offset arithmetic overflowed.
    LengthOverflow,
    /// An offset or range points outside the input.
    OffsetOutOfBounds,
}

impl DecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "ETH_CODEC_INPUT_TOO_LARGE",
            Self::TrailingBytes => "ETH_CODEC_TRAILING_BYTES",
            Self::DecoderOverread => "ETH_CODEC_DECODER_OVERREAD",
            Self::Malformed => "ETH_CODEC_MALFORMED",
            Self::UnexpectedList => "ETH_CODEC_UNEXPECTED_LIST",
            Self::UnexpectedScalar => "ETH_CODEC_UNEXPECTED_SCALAR",
            Self::ListTooLong => "ETH_CODEC_LIST_TOO_LONG",
            Self::NestingTooDeep => "ETH_CODEC_NESTING_TOO_DEEP",
            Self::AllocationExceeded => "ETH_CODEC_ALLOCATION_EXCEEDED",
            Self::ProofTooLarge => "ETH_CODEC_PROOF_TOO_LARGE",
            Self::ItemCountExceeded => "ETH_CODEC_ITEM_COUNT_EXCEEDED",
            Self::EncodedBytesExceeded => "ETH_CODEC_ENCODED_BYTES_EXCEEDED",
            Self::RlpHeaderCountExceeded => "ETH_CODEC_RLP_HEADER_COUNT_EXCEEDED",
            Self::HashCountExceeded => "ETH_CODEC_HASH_COUNT_EXCEEDED",
            Self::HashBytesExceeded => "ETH_CODEC_HASH_BYTES_EXCEEDED",
            Self::NibbleCountExceeded => "ETH_CODEC_NIBBLE_COUNT_EXCEEDED",
            Self::ValueBytesExceeded => "ETH_CODEC_VALUE_BYTES_EXCEEDED",
            Self::WorkExceeded => "ETH_CODEC_WORK_EXCEEDED",
            Self::InvalidSessionPolicy => "ETH_CODEC_INVALID_SESSION_POLICY",
            Self::UnreviewedDeploymentPolicy => "ETH_CODEC_UNREVIEWED_DEPLOYMENT_POLICY",
            Self::LengthOverflow => "ETH_CODEC_LENGTH_OVERFLOW",
            Self::OffsetOutOfBounds => "ETH_CODEC_OFFSET_OUT_OF_BOUNDS",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InputTooLarge => "input exceeds the active decode byte limit",
            Self::TrailingBytes => "decoded value did not consume the full input",
            Self::DecoderOverread => "decoder consumed more bytes than were available",
            Self::Malformed => "input is malformed for the selected codec",
            Self::UnexpectedList => "RLP list encountered where scalar was required",
            Self::UnexpectedScalar => "RLP scalar encountered where list was required",
            Self::ListTooLong => "decoded list exceeds the active item limit",
            Self::NestingTooDeep => "decoded structure exceeds the active nesting limit",
            Self::AllocationExceeded => "decoder exceeded the active allocation limit",
            Self::ProofTooLarge => "proof exceeds the active proof-node limit",
            Self::ItemCountExceeded => "decoder exceeded the active cumulative item limit",
            Self::EncodedBytesExceeded => "decoder exceeded the encoded-byte work limit",
            Self::RlpHeaderCountExceeded => "decoder exceeded the RLP-header work limit",
            Self::HashCountExceeded => "decoder exceeded the hash-count work limit",
            Self::HashBytesExceeded => "decoder exceeded the hashed-byte work limit",
            Self::NibbleCountExceeded => "decoder exceeded the trie-nibble work limit",
            Self::ValueBytesExceeded => "decoder exceeded the trie-value byte limit",
            Self::WorkExceeded => "decoder exceeded the aggregate work limit",
            Self::InvalidSessionPolicy => "decode-session policy relationships are invalid",
            Self::UnreviewedDeploymentPolicy => {
                "deployment decode policy must be reviewed and tightened"
            }
            Self::LengthOverflow => "length or offset arithmetic overflowed",
            Self::OffsetOutOfBounds => "offset or range is outside the input",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> DecodeErrorCategory {
        match self {
            Self::InputTooLarge
            | Self::ListTooLong
            | Self::NestingTooDeep
            | Self::AllocationExceeded
            | Self::ProofTooLarge
            | Self::ItemCountExceeded
            | Self::EncodedBytesExceeded
            | Self::RlpHeaderCountExceeded
            | Self::HashCountExceeded
            | Self::HashBytesExceeded
            | Self::NibbleCountExceeded
            | Self::ValueBytesExceeded
            | Self::WorkExceeded
            | Self::InvalidSessionPolicy
            | Self::UnreviewedDeploymentPolicy => DecodeErrorCategory::ResourceExhaustion,
            Self::TrailingBytes
            | Self::DecoderOverread
            | Self::Malformed
            | Self::UnexpectedList
            | Self::UnexpectedScalar
            | Self::LengthOverflow
            | Self::OffsetOutOfBounds => DecodeErrorCategory::MalformedInput,
        }
    }

    /// Returns the resource budget that was exceeded, if this is a resource
    /// exhaustion error.
    #[must_use]
    pub const fn resource(self) -> Option<ResourceError> {
        match self {
            Self::InputTooLarge => Some(ResourceError::InputBytes),
            Self::ListTooLong => Some(ResourceError::ListItems),
            Self::NestingTooDeep => Some(ResourceError::NestingDepth),
            Self::AllocationExceeded => Some(ResourceError::AllocationBytes),
            Self::ProofTooLarge => Some(ResourceError::ProofNodes),
            Self::ItemCountExceeded => Some(ResourceError::TotalItems),
            Self::EncodedBytesExceeded => Some(ResourceError::EncodedBytes),
            Self::RlpHeaderCountExceeded => Some(ResourceError::RlpHeaders),
            Self::HashCountExceeded => Some(ResourceError::Hashes),
            Self::HashBytesExceeded => Some(ResourceError::HashBytes),
            Self::NibbleCountExceeded => Some(ResourceError::Nibbles),
            Self::ValueBytesExceeded => Some(ResourceError::ValueBytes),
            Self::WorkExceeded => Some(ResourceError::TotalWork),
            Self::InvalidSessionPolicy => Some(ResourceError::SessionPolicy),
            Self::UnreviewedDeploymentPolicy => Some(ResourceError::DeploymentPolicy),
            Self::TrailingBytes
            | Self::DecoderOverread
            | Self::Malformed
            | Self::UnexpectedList
            | Self::UnexpectedScalar
            | Self::LengthOverflow
            | Self::OffsetOutOfBounds => None,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

/// Stable high-level decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeErrorCategory {
    /// The input is malformed or internally inconsistent.
    MalformedInput,
    /// The input exceeded an explicit resource budget.
    ResourceExhaustion,
}

/// Stable resource budget categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceError {
    /// Input byte limit was exceeded.
    InputBytes,
    /// Per-list item limit was exceeded.
    ListItems,
    /// Nesting depth limit was exceeded.
    NestingDepth,
    /// Cumulative allocation limit was exceeded.
    AllocationBytes,
    /// Proof-node limit was exceeded.
    ProofNodes,
    /// Cumulative decoded item limit was exceeded.
    TotalItems,
    /// Cumulative encoded-byte scan limit was exceeded.
    EncodedBytes,
    /// Cumulative RLP-header visit limit was exceeded.
    RlpHeaders,
    /// Cumulative hash-count limit was exceeded.
    Hashes,
    /// Cumulative hashed-byte limit was exceeded.
    HashBytes,
    /// Cumulative trie-path nibble limit was exceeded.
    Nibbles,
    /// Cumulative trie-value byte limit was exceeded.
    ValueBytes,
    /// Aggregate decode-work limit was exceeded.
    TotalWork,
    /// Decode-session policy relationships were invalid.
    SessionPolicy,
    /// Deployment policy was not reviewed.
    DeploymentPolicy,
}

impl ResourceError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputBytes => "ETH_RESOURCE_INPUT_BYTES",
            Self::ListItems => "ETH_RESOURCE_LIST_ITEMS",
            Self::NestingDepth => "ETH_RESOURCE_NESTING_DEPTH",
            Self::AllocationBytes => "ETH_RESOURCE_ALLOCATION_BYTES",
            Self::ProofNodes => "ETH_RESOURCE_PROOF_NODES",
            Self::TotalItems => "ETH_RESOURCE_TOTAL_ITEMS",
            Self::EncodedBytes => "ETH_RESOURCE_ENCODED_BYTES",
            Self::RlpHeaders => "ETH_RESOURCE_RLP_HEADERS",
            Self::Hashes => "ETH_RESOURCE_HASHES",
            Self::HashBytes => "ETH_RESOURCE_HASH_BYTES",
            Self::Nibbles => "ETH_RESOURCE_NIBBLES",
            Self::ValueBytes => "ETH_RESOURCE_VALUE_BYTES",
            Self::TotalWork => "ETH_RESOURCE_TOTAL_WORK",
            Self::SessionPolicy => "ETH_RESOURCE_SESSION_POLICY",
            Self::DeploymentPolicy => "ETH_RESOURCE_DEPLOYMENT_POLICY",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InputBytes => "input byte budget exceeded",
            Self::ListItems => "list item budget exceeded",
            Self::NestingDepth => "nesting depth budget exceeded",
            Self::AllocationBytes => "allocation byte budget exceeded",
            Self::ProofNodes => "proof-node budget exceeded",
            Self::TotalItems => "total item budget exceeded",
            Self::EncodedBytes => "encoded-byte scan budget exceeded",
            Self::RlpHeaders => "RLP-header visit budget exceeded",
            Self::Hashes => "hash-count budget exceeded",
            Self::HashBytes => "hashed-byte budget exceeded",
            Self::Nibbles => "trie-path nibble budget exceeded",
            Self::ValueBytes => "trie-value byte budget exceeded",
            Self::TotalWork => "aggregate decode-work budget exceeded",
            Self::SessionPolicy => "decode-session policy is inconsistent",
            Self::DeploymentPolicy => "deployment policy review required",
        }
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ResourceError {}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::string::ToString;

    #[test]
    fn decode_errors_have_stable_codes_and_messages() {
        assert_eq!(DecodeError::Malformed.code(), "ETH_CODEC_MALFORMED");
        assert_eq!(
            DecodeError::Malformed.message(),
            "input is malformed for the selected codec"
        );
        assert_eq!(
            DecodeError::Malformed.category(),
            DecodeErrorCategory::MalformedInput
        );
        assert_eq!(
            DecodeError::Malformed.to_string(),
            "input is malformed for the selected codec"
        );
    }

    #[test]
    fn resource_errors_are_classified_without_payloads() {
        let error = DecodeError::AllocationExceeded;

        assert_eq!(error.category(), DecodeErrorCategory::ResourceExhaustion);
        assert_eq!(error.resource(), Some(ResourceError::AllocationBytes));
        assert_eq!(
            ResourceError::AllocationBytes.code(),
            "ETH_RESOURCE_ALLOCATION_BYTES"
        );
        assert_eq!(
            ResourceError::AllocationBytes.to_string(),
            "allocation byte budget exceeded"
        );
    }

    #[test]
    fn new_resource_errors_are_classified() {
        assert_eq!(
            DecodeError::ProofTooLarge.resource(),
            Some(ResourceError::ProofNodes)
        );
        assert_eq!(
            DecodeError::ItemCountExceeded.resource(),
            Some(ResourceError::TotalItems)
        );
        assert_eq!(
            DecodeError::UnreviewedDeploymentPolicy.resource(),
            Some(ResourceError::DeploymentPolicy)
        );
    }

    #[test]
    fn arithmetic_errors_are_malformed_input() {
        assert_eq!(
            DecodeError::LengthOverflow.category(),
            DecodeErrorCategory::MalformedInput
        );
        assert_eq!(DecodeError::OffsetOutOfBounds.resource(), None);
    }

    #[test]
    fn unexpected_list_is_malformed_input() {
        assert_eq!(
            DecodeError::UnexpectedList.code(),
            "ETH_CODEC_UNEXPECTED_LIST"
        );
        assert_eq!(
            DecodeError::UnexpectedList.category(),
            DecodeErrorCategory::MalformedInput
        );
        assert_eq!(DecodeError::UnexpectedList.resource(), None);
    }

    #[test]
    fn unexpected_scalar_is_malformed_input() {
        assert_eq!(
            DecodeError::UnexpectedScalar.code(),
            "ETH_CODEC_UNEXPECTED_SCALAR"
        );
        assert_eq!(
            DecodeError::UnexpectedScalar.category(),
            DecodeErrorCategory::MalformedInput
        );
        assert_eq!(DecodeError::UnexpectedScalar.resource(), None);
    }
}
