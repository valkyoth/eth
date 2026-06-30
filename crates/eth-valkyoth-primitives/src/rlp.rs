use core::fmt;

use eth_valkyoth_codec::{
    DecodeError, DecodeLimits, decode_rlp_scalar, decode_rlp_u64, decode_rlp_u256_bytes,
    encode_rlp_integer, encode_rlp_scalar, encoded_rlp_integer_len, encoded_rlp_scalar_len,
};

use crate::{Address, B256, BlockNumber, ChainId, Gas, Nonce, PrimitiveError, UnixTimestamp, Wei};

/// Primitive RLP bridge failures.
///
/// This enum is `#[non_exhaustive]`. New variants may be added in minor
/// releases. Downstream wildcard `match` arms can silently handle new failure
/// categories, so security monitors should re-audit wildcard arms when this
/// type changes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveRlpError {
    /// The underlying bounded RLP codec rejected the input or output buffer.
    Decode(DecodeError),
    /// The decoded payload did not fit the target primitive domain.
    Primitive(PrimitiveError),
    /// A fixed-width primitive was encoded with the wrong scalar byte length.
    FixedWidthScalar {
        /// Required scalar payload width for the target primitive.
        expected: usize,
        /// Actual decoded scalar payload width.
        found: usize,
    },
}

impl From<DecodeError> for PrimitiveRlpError {
    fn from(error: DecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<PrimitiveError> for PrimitiveRlpError {
    fn from(error: PrimitiveError) -> Self {
        Self::Primitive(error)
    }
}

impl fmt::Display for PrimitiveRlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => write!(f, "RLP decode error: {error}"),
            Self::Primitive(error) => write!(f, "primitive domain error: {error}"),
            Self::FixedWidthScalar { expected, found } => write!(
                f,
                "RLP scalar payload has wrong byte width for this primitive: expected {expected}, found {found}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PrimitiveRlpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::Primitive(error) => Some(error),
            Self::FixedWidthScalar { .. } => None,
        }
    }
}

macro_rules! rlp_u64_bridge {
    ($name:ident) => {
        impl $name {
            /// Returns the canonical RLP encoded length.
            pub fn encoded_rlp_len(self) -> Result<usize, PrimitiveRlpError> {
                encoded_u64_len(self.get())
            }

            /// Canonically encodes this value into `output`.
            ///
            /// Returns the number of bytes written. `output` is not modified
            /// unless this function returns `Ok`.
            pub fn encode_rlp(self, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
                encode_u64(self.get(), output)
            }

            /// Decodes exactly one canonical RLP integer into this domain.
            pub fn try_from_rlp(
                input: &[u8],
                limits: DecodeLimits,
            ) -> Result<Self, PrimitiveRlpError> {
                decode_rlp_u64(input, limits)
                    .map(Self::new)
                    .map_err(Into::into)
            }
        }
    };
}

impl ChainId {
    /// Returns the canonical RLP encoded length.
    pub fn encoded_rlp_len(self) -> Result<usize, PrimitiveRlpError> {
        encoded_u64_len(self.get())
    }

    /// Canonically encodes this value into `output`.
    ///
    /// Returns the number of bytes written. `output` is not modified unless
    /// this function returns `Ok`.
    pub fn encode_rlp(self, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
        encode_u64(self.get(), output)
    }

    /// Decodes exactly one canonical RLP integer into this domain.
    ///
    /// **EIP-155 note**: This function accepts `ChainId(0)`, which is reserved
    /// for unsigned legacy transactions. Callers that validate signed
    /// transaction chain IDs must reject `ChainId(0)` independently; this
    /// decoder does not enforce that constraint.
    pub fn try_from_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, PrimitiveRlpError> {
        decode_rlp_u64(input, limits)
            .map(Self::new)
            .map_err(Into::into)
    }

    /// Decodes a signed EIP-155 transaction chain ID.
    ///
    /// Rejects `0`, which is reserved for unsigned legacy transactions and
    /// must not be accepted as a signed transaction replay domain.
    pub fn try_from_rlp_signed(
        input: &[u8],
        limits: DecodeLimits,
    ) -> Result<Self, PrimitiveRlpError> {
        let chain_id = Self::try_from_rlp(input, limits)?;
        if chain_id.get() == 0 {
            return Err(PrimitiveRlpError::Primitive(
                PrimitiveError::ReservedLegacyType,
            ));
        }
        Ok(chain_id)
    }
}

rlp_u64_bridge!(BlockNumber);
rlp_u64_bridge!(Gas);
rlp_u64_bridge!(Nonce);
rlp_u64_bridge!(UnixTimestamp);

impl Wei {
    /// Returns the canonical RLP encoded length.
    pub fn encoded_rlp_len(self) -> Result<usize, PrimitiveRlpError> {
        let bytes = self.to_be_bytes();
        encoded_rlp_integer_len(trim_u256_payload(&bytes)).map_err(Into::into)
    }

    /// Canonically encodes this value into `output`.
    ///
    /// Returns the number of bytes written. `output` is not modified unless
    /// this function returns `Ok`.
    pub fn encode_rlp(self, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
        let bytes = self.to_be_bytes();
        encode_rlp_integer(trim_u256_payload(&bytes), output).map_err(Into::into)
    }

    /// Decodes exactly one canonical RLP U256 integer into `Wei`.
    pub fn try_from_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, PrimitiveRlpError> {
        decode_rlp_u256_bytes(input, limits)
            .map(Self::from_be_bytes)
            .map_err(Into::into)
    }
}

impl Address {
    /// Returns the canonical RLP encoded length.
    ///
    /// This is currently infallible for the fixed 20-byte address payload but
    /// keeps the same `Result` shape as the other primitive bridge helpers.
    pub fn encoded_rlp_len(self) -> Result<usize, PrimitiveRlpError> {
        encoded_rlp_scalar_len(&self.to_bytes()).map_err(Into::into)
    }

    /// Canonically encodes this address as a fixed-width scalar.
    ///
    /// Returns the number of bytes written. `output` is not modified unless
    /// this function returns `Ok`.
    pub fn encode_rlp(self, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
        encode_rlp_scalar(&self.to_bytes(), output).map_err(Into::into)
    }

    /// Decodes exactly one fixed-width address scalar.
    pub fn try_from_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, PrimitiveRlpError> {
        let scalar = decode_rlp_scalar(input, limits)?;
        let found = scalar.payload().len();
        let bytes: [u8; 20] =
            scalar
                .payload()
                .try_into()
                .map_err(|_| PrimitiveRlpError::FixedWidthScalar {
                    expected: 20,
                    found,
                })?;
        Ok(Self::from_bytes(bytes))
    }
}

impl B256 {
    /// Returns the canonical RLP encoded length.
    ///
    /// This is currently infallible for the fixed 32-byte hash payload but
    /// keeps the same `Result` shape as the other primitive bridge helpers.
    pub fn encoded_rlp_len(self) -> Result<usize, PrimitiveRlpError> {
        encoded_rlp_scalar_len(&self.to_bytes()).map_err(Into::into)
    }

    /// Canonically encodes this hash as a fixed-width scalar.
    ///
    /// Returns the number of bytes written. `output` is not modified unless
    /// this function returns `Ok`.
    pub fn encode_rlp(self, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
        encode_rlp_scalar(&self.to_bytes(), output).map_err(Into::into)
    }

    /// Decodes exactly one fixed-width hash scalar.
    pub fn try_from_rlp(input: &[u8], limits: DecodeLimits) -> Result<Self, PrimitiveRlpError> {
        let scalar = decode_rlp_scalar(input, limits)?;
        let found = scalar.payload().len();
        let bytes: [u8; 32] =
            scalar
                .payload()
                .try_into()
                .map_err(|_| PrimitiveRlpError::FixedWidthScalar {
                    expected: 32,
                    found,
                })?;
        Ok(Self::from_bytes(bytes))
    }
}

fn encoded_u64_len(value: u64) -> Result<usize, PrimitiveRlpError> {
    let bytes = value.to_be_bytes();
    encoded_rlp_integer_len(trim_u64_payload(&bytes)).map_err(Into::into)
}

fn encode_u64(value: u64, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
    let bytes = value.to_be_bytes();
    encode_rlp_integer(trim_u64_payload(&bytes), output).map_err(Into::into)
}

fn trim_u64_payload(bytes: &[u8; 8]) -> &[u8] {
    // Variable-time scan: execution time leaks the bit width of the value.
    // This path is for public Ethereum protocol fields such as chain IDs,
    // nonces, gas values, timestamps, and block numbers. Do not reuse it for
    // secret or pre-disclosure values.
    let start = bytes.iter().position(|byte| *byte != 0).unwrap_or(8);
    bytes.get(start..).unwrap_or(&[])
}

fn trim_u256_payload(bytes: &[u8; 32]) -> &[u8] {
    // Variable-time scan: execution time leaks the bit width of the value.
    // Wei values are public in normal Ethereum transactions. Do not reuse this
    // helper for secret or pre-disclosure amounts without re-auditing timing.
    let start = bytes.iter().position(|byte| *byte != 0).unwrap_or(32);
    bytes.get(start..).unwrap_or(&[])
}
