use eth_valkyoth_codec::{
    DecodeError, DecodeLimits, decode_rlp_scalar, decode_rlp_u64, decode_rlp_u256_bytes,
    encode_rlp_integer, encode_rlp_scalar, encoded_rlp_integer_len, encoded_rlp_scalar_len,
};

use crate::{Address, B256, BlockNumber, ChainId, Gas, Nonce, PrimitiveError, UnixTimestamp, Wei};

/// Primitive RLP bridge failures.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveRlpError {
    /// The underlying bounded RLP codec rejected the input or output buffer.
    Decode(DecodeError),
    /// The decoded payload did not fit the target primitive domain.
    Primitive(PrimitiveError),
    /// A fixed-width primitive was encoded with the wrong scalar byte length.
    FixedWidthScalar,
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

rlp_u64_bridge!(ChainId);
rlp_u64_bridge!(BlockNumber);
rlp_u64_bridge!(Gas);
rlp_u64_bridge!(Nonce);
rlp_u64_bridge!(UnixTimestamp);

impl Wei {
    /// Returns the canonical RLP encoded length.
    pub fn encoded_rlp_len(self) -> Result<usize, PrimitiveRlpError> {
        let bytes = self.to_be_bytes();
        encoded_rlp_integer_len(trim_u256_payload(&bytes)?).map_err(Into::into)
    }

    /// Canonically encodes this value into `output`.
    ///
    /// Returns the number of bytes written. `output` is not modified unless
    /// this function returns `Ok`.
    pub fn encode_rlp(self, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
        let bytes = self.to_be_bytes();
        encode_rlp_integer(trim_u256_payload(&bytes)?, output).map_err(Into::into)
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
        let bytes: [u8; 20] = scalar
            .payload()
            .try_into()
            .map_err(|_| PrimitiveRlpError::FixedWidthScalar)?;
        Ok(Self::from_bytes(bytes))
    }
}

impl B256 {
    /// Returns the canonical RLP encoded length.
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
        let bytes: [u8; 32] = scalar
            .payload()
            .try_into()
            .map_err(|_| PrimitiveRlpError::FixedWidthScalar)?;
        Ok(Self::from_bytes(bytes))
    }
}

fn encoded_u64_len(value: u64) -> Result<usize, PrimitiveRlpError> {
    let bytes = value.to_be_bytes();
    encoded_rlp_integer_len(trim_u64_payload(value, &bytes)?).map_err(Into::into)
}

fn encode_u64(value: u64, output: &mut [u8]) -> Result<usize, PrimitiveRlpError> {
    let bytes = value.to_be_bytes();
    encode_rlp_integer(trim_u64_payload(value, &bytes)?, output).map_err(Into::into)
}

fn trim_u64_payload(value: u64, bytes: &[u8; 8]) -> Result<&[u8], PrimitiveRlpError> {
    if value == 0 {
        return Ok(&[]);
    }
    let start = bytes
        .iter()
        .position(|byte| *byte != 0)
        .ok_or(PrimitiveRlpError::Decode(DecodeError::Malformed))?;
    bytes
        .get(start..)
        .ok_or(PrimitiveRlpError::Decode(DecodeError::OffsetOutOfBounds))
}

fn trim_u256_payload(bytes: &[u8; 32]) -> Result<&[u8], PrimitiveRlpError> {
    let Some(start) = bytes.iter().position(|byte| *byte != 0) else {
        return Ok(&[]);
    };
    bytes
        .get(start..)
        .ok_or(PrimitiveRlpError::Decode(DecodeError::OffsetOutOfBounds))
}
