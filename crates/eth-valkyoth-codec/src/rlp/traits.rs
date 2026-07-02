use core::fmt;

use crate::{DecodeError, DecodeLimits, checked_len_add};

use super::{
    RlpInteger, RlpItem, decode_rlp_u64, decode_rlp_u128, encode_rlp_integer, encode_rlp_scalar,
    encoded_rlp_integer_len, encoded_rlp_scalar_len,
};

/// RLP encoding contract for derive-generated and hand-written domains.
///
/// Implementations must return the exact encoded length and must not modify the
/// output buffer when returning an error.
pub trait RlpEncode {
    /// Error returned by this encoder.
    type Error;

    /// Returns the exact canonical RLP encoded length.
    fn encoded_rlp_len(&self) -> Result<usize, Self::Error>;

    /// Canonically encodes `self` into `output`.
    ///
    /// Returns the number of bytes written.
    fn encode_rlp(&self, output: &mut [u8]) -> Result<usize, Self::Error>;
}

/// RLP decoding contract for derive-generated and hand-written domains.
pub trait RlpDecode: Sized {
    /// Error returned by this decoder.
    type Error;

    /// Decodes exactly one canonical RLP item from `input`.
    fn decode_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, Self::Error>;

    /// Decodes one already-bounded child item from an outer RLP list.
    fn decode_rlp_item(item: RlpItem<'_>) -> Result<Self, Self::Error>;
}

/// Error used by derive-generated RLP struct encoders and decoders.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlpDeriveError {
    /// The bounded codec rejected the input or output buffer.
    Decode(DecodeError),
    /// A field-specific codec rejected one generated field operation.
    Field,
    /// The decoded list field count did not match the generated struct shape.
    WrongFieldCount {
        /// Expected encoded field count.
        expected: usize,
        /// Decoded encoded field count.
        found: usize,
    },
}

impl From<DecodeError> for RlpDeriveError {
    fn from(error: DecodeError) -> Self {
        Self::Decode(error)
    }
}

impl fmt::Display for RlpDeriveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => write!(formatter, "RLP codec error: {error}"),
            Self::Field => formatter.write_str("RLP field codec error"),
            Self::WrongFieldCount { expected, found } => write!(
                formatter,
                "RLP list field count mismatch: expected {expected}, found {found}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RlpDeriveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::Field | Self::WrongFieldCount { .. } => None,
        }
    }
}

impl From<core::convert::Infallible> for RlpDeriveError {
    fn from(error: core::convert::Infallible) -> Self {
        match error {}
    }
}

impl RlpEncode for u64 {
    type Error = DecodeError;

    fn encoded_rlp_len(&self) -> Result<usize, Self::Error> {
        let bytes = self.to_be_bytes();
        encoded_rlp_integer_len(trim_payload(&bytes))
    }

    fn encode_rlp(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
        let bytes = self.to_be_bytes();
        encode_rlp_integer(trim_payload(&bytes), output)
    }
}

impl RlpDecode for u64 {
    type Error = DecodeError;

    fn decode_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, Self::Error> {
        decode_rlp_u64(input, limits)
    }

    fn decode_rlp_item(item: RlpItem<'_>) -> Result<Self, Self::Error> {
        let scalar = item.as_scalar().ok_or(DecodeError::UnexpectedList)?;
        RlpInteger::try_from_scalar(scalar)?.to_u64()
    }
}

impl RlpEncode for u128 {
    type Error = DecodeError;

    fn encoded_rlp_len(&self) -> Result<usize, Self::Error> {
        let bytes = self.to_be_bytes();
        encoded_rlp_integer_len(trim_payload(&bytes))
    }

    fn encode_rlp(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
        let bytes = self.to_be_bytes();
        encode_rlp_integer(trim_payload(&bytes), output)
    }
}

impl RlpDecode for u128 {
    type Error = DecodeError;

    fn decode_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, Self::Error> {
        decode_rlp_u128(input, limits)
    }

    fn decode_rlp_item(item: RlpItem<'_>) -> Result<Self, Self::Error> {
        let scalar = item.as_scalar().ok_or(DecodeError::UnexpectedList)?;
        RlpInteger::try_from_scalar(scalar)?.to_u128()
    }
}

impl<const N: usize> RlpEncode for [u8; N] {
    type Error = DecodeError;

    fn encoded_rlp_len(&self) -> Result<usize, Self::Error> {
        encoded_rlp_scalar_len(self)
    }

    fn encode_rlp(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
        encode_rlp_scalar(self, output)
    }
}

impl<const N: usize> RlpDecode for [u8; N] {
    type Error = DecodeError;

    fn decode_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, Self::Error> {
        let scalar = super::decode_rlp_scalar(input, limits)?;
        scalar
            .payload()
            .try_into()
            .map_err(|_| DecodeError::Malformed)
    }

    fn decode_rlp_item(item: RlpItem<'_>) -> Result<Self, Self::Error> {
        let scalar = item.as_scalar().ok_or(DecodeError::UnexpectedList)?;
        scalar
            .payload()
            .try_into()
            .map_err(|_| DecodeError::Malformed)
    }
}

fn trim_payload<const N: usize>(bytes: &[u8; N]) -> &[u8] {
    let start = bytes.iter().position(|byte| *byte != 0).unwrap_or(N);
    bytes.get(start..).unwrap_or(&[])
}

/// Adds two encoded lengths with overflow checking.
pub fn checked_encoded_len_add(left: usize, right: usize) -> Result<usize, RlpDeriveError> {
    checked_len_add(left, right).map_err(Into::into)
}
