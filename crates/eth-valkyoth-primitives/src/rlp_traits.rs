use eth_valkyoth_codec::{DecodeError, RlpDecode, RlpDeriveError, RlpEncode, RlpInteger, RlpItem};

use crate::{
    Address, B256, BlockNumber, ChainId, Gas, Nonce, PrimitiveError, PrimitiveRlpError,
    UnixTimestamp, Wei,
};

impl From<PrimitiveRlpError> for RlpDeriveError {
    fn from(_error: PrimitiveRlpError) -> Self {
        Self::Field
    }
}

macro_rules! rlp_u64_traits {
    ($name:ident) => {
        impl RlpEncode for $name {
            type Error = PrimitiveRlpError;

            fn encoded_rlp_len(&self) -> Result<usize, Self::Error> {
                $name::encoded_rlp_len(*self)
            }

            fn encode_rlp(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
                $name::encode_rlp(*self, output)
            }
        }

        impl RlpDecode for $name {
            type Error = PrimitiveRlpError;

            fn decode_rlp(
                input: &[u8],
                limits: eth_valkyoth_codec::DecodeLimits,
            ) -> Result<Self, Self::Error> {
                $name::try_from_rlp(input, limits)
            }

            fn decode_rlp_item(item: RlpItem<'_>) -> Result<Self, Self::Error> {
                decode_u64_item(item).map($name::new)
            }
        }
    };
}

rlp_u64_traits!(ChainId);
rlp_u64_traits!(BlockNumber);
rlp_u64_traits!(Gas);
rlp_u64_traits!(Nonce);
rlp_u64_traits!(UnixTimestamp);

impl RlpEncode for Wei {
    type Error = PrimitiveRlpError;

    fn encoded_rlp_len(&self) -> Result<usize, Self::Error> {
        Self::encoded_rlp_len(*self)
    }

    fn encode_rlp(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
        Self::encode_rlp(*self, output)
    }
}

impl RlpDecode for Wei {
    type Error = PrimitiveRlpError;

    fn decode_rlp(
        input: &[u8],
        limits: eth_valkyoth_codec::DecodeLimits,
    ) -> Result<Self, Self::Error> {
        Self::try_from_rlp(input, limits)
    }

    fn decode_rlp_item(item: RlpItem<'_>) -> Result<Self, Self::Error> {
        let scalar = item.as_scalar().ok_or(DecodeError::UnexpectedList)?;
        RlpInteger::try_from_scalar(scalar)?
            .to_be_bytes32()
            .map(Self::from_be_bytes)
            .map_err(Into::into)
    }
}

macro_rules! rlp_fixed_scalar_traits {
    ($name:ident, $len:expr) => {
        impl RlpEncode for $name {
            type Error = PrimitiveRlpError;

            fn encoded_rlp_len(&self) -> Result<usize, Self::Error> {
                $name::encoded_rlp_len(*self)
            }

            fn encode_rlp(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
                $name::encode_rlp(*self, output)
            }
        }

        impl RlpDecode for $name {
            type Error = PrimitiveRlpError;

            fn decode_rlp(
                input: &[u8],
                limits: eth_valkyoth_codec::DecodeLimits,
            ) -> Result<Self, Self::Error> {
                $name::try_from_rlp(input, limits)
            }

            fn decode_rlp_item(item: RlpItem<'_>) -> Result<Self, Self::Error> {
                let scalar = item.as_scalar().ok_or(DecodeError::UnexpectedList)?;
                let found = scalar.payload().len();
                let bytes = scalar.payload().try_into().map_err(|_| {
                    PrimitiveRlpError::FixedWidthScalar {
                        expected: $len,
                        found,
                    }
                })?;
                Ok($name::from_bytes(bytes))
            }
        }
    };
}

rlp_fixed_scalar_traits!(Address, 20);
rlp_fixed_scalar_traits!(B256, 32);

fn decode_u64_item(item: RlpItem<'_>) -> Result<u64, PrimitiveRlpError> {
    let scalar = item.as_scalar().ok_or(DecodeError::UnexpectedList)?;
    RlpInteger::try_from_scalar(scalar)?
        .to_u64()
        .map_err(map_decode_u64_error)
}

fn map_decode_u64_error(error: DecodeError) -> PrimitiveRlpError {
    match error {
        DecodeError::Malformed => PrimitiveRlpError::Primitive(PrimitiveError::NonCanonicalInteger),
        DecodeError::LengthOverflow | DecodeError::OffsetOutOfBounds => {
            PrimitiveRlpError::Primitive(PrimitiveError::IntegerTooLarge)
        }
        _ => PrimitiveRlpError::Decode(error),
    }
}
