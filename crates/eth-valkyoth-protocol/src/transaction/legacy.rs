use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeLimits, RlpInteger, RlpItem, RlpList, RlpScalar};
use eth_valkyoth_primitives::{Address, ChainId, Gas, Nonce, Wei};

use super::{TransactionEnvelope, TransactionEnvelopeError, decode_transaction_envelope};

/// Number of fields in a canonical legacy transaction RLP list.
pub const LEGACY_TRANSACTION_FIELD_COUNT: usize = 9;
const ADDRESS_BYTES: usize = 20;

/// Borrowed legacy transaction decoded only into field domains.
///
/// This type is intentionally unvalidated: no sender recovery, EIP-155 chain
/// binding, intrinsic-gas check, nonce-state check, fork check, or signature
/// validity is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedLegacyTransaction<'a> {
    /// Account nonce.
    pub nonce: Nonce,
    /// Gas price in wei.
    pub gas_price: Wei,
    /// Gas limit.
    pub gas_limit: Gas,
    /// Call or contract-creation target.
    pub to: LegacyTransactionTo,
    /// Transferred value in wei.
    pub value: Wei,
    /// Borrowed transaction input data.
    pub input: &'a [u8],
    /// Raw canonical U256 signature recovery value.
    ///
    /// This is not checked for EIP-155, chain, or fork validity. Use
    /// [`Self::eip155_chain_id`] instead of subtracting from this value
    /// directly.
    pub v: [u8; 32],
    /// Raw canonical U256 signature `r` value.
    ///
    /// This is not checked for secp256k1 scalar validity.
    pub r: [u8; 32],
    /// Raw canonical U256 signature `s` value.
    ///
    /// This is not checked against the EIP-2 low-s bound or secp256k1 scalar
    /// validity.
    pub s: [u8; 32],
}

impl UnvalidatedLegacyTransaction<'_> {
    /// Returns the EIP-155 chain ID encoded by `v`, if it fits this crate's
    /// chain-domain width.
    ///
    /// Returns `None` for pre-EIP-155 `v` values such as `27` and `28`, and
    /// for oversized `v` values that cannot fit in `u64`. This helper is
    /// intentionally syntactic: it does not prove that the chain ID is nonzero,
    /// configured, active, or valid for any fork.
    #[must_use]
    pub fn eip155_chain_id(&self) -> Option<ChainId> {
        const U64_TAIL_START: usize = 24;

        let high_bytes = self.v.get(..U64_TAIL_START)?;
        if high_bytes.iter().any(|byte| *byte != 0) {
            return None;
        }

        let low_bytes = self.v.get(U64_TAIL_START..)?.try_into().ok()?;
        let v = u64::from_be_bytes(low_bytes);
        v.checked_sub(35).map(|delta| ChainId::new(delta / 2))
    }
}

/// Legacy transaction call/create target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyTransactionTo {
    /// Contract creation transaction with an empty `to` field.
    Create,
    /// Message call to an address.
    Call(Address),
}

/// Legacy transaction field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyTransactionField {
    /// `nonce`.
    Nonce,
    /// `gas_price`.
    GasPrice,
    /// `gas_limit`.
    GasLimit,
    /// `to`.
    To,
    /// `value`.
    Value,
    /// `input`.
    Input,
    /// `v`.
    V,
    /// `r`.
    R,
    /// `s`.
    S,
}

/// Legacy transaction decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyTransactionDecodeError {
    /// Envelope classification failed before legacy fields could be decoded.
    Envelope(TransactionEnvelopeError),
    /// A typed envelope was supplied to the legacy decoder.
    TypedEnvelope {
        /// Observed typed transaction prefix.
        type_byte: u8,
    },
    /// Legacy transaction list did not contain exactly nine fields.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A field failed RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: LegacyTransactionField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// The `to` field was neither empty nor a 20-byte address.
    InvalidToLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
}

impl LegacyTransactionDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.code(),
            Self::TypedEnvelope { .. } => "ETH_LEGACY_TX_TYPED_ENVELOPE",
            Self::WrongFieldCount { .. } => "ETH_LEGACY_TX_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::InvalidToLength { .. } => "ETH_LEGACY_TX_INVALID_TO_LENGTH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.message(),
            Self::TypedEnvelope { .. } => "legacy transaction decoder received a typed envelope",
            Self::WrongFieldCount { .. } => {
                "legacy transaction must contain exactly nine RLP fields"
            }
            Self::FieldDecode { .. } => "legacy transaction field failed bounded decoding",
            Self::InvalidToLength { .. } => {
                "legacy transaction to field must be empty or a 20-byte address"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> LegacyTransactionDecodeErrorCategory {
        match self {
            Self::Envelope(error) => match error.category() {
                super::TransactionEnvelopeErrorCategory::MalformedInput => {
                    LegacyTransactionDecodeErrorCategory::MalformedInput
                }
                super::TransactionEnvelopeErrorCategory::Unsupported => {
                    LegacyTransactionDecodeErrorCategory::Unsupported
                }
                super::TransactionEnvelopeErrorCategory::ResourceExhaustion => {
                    LegacyTransactionDecodeErrorCategory::ResourceExhaustion
                }
            },
            Self::TypedEnvelope { .. } => LegacyTransactionDecodeErrorCategory::WrongType,
            Self::WrongFieldCount { .. } | Self::InvalidToLength { .. } => {
                LegacyTransactionDecodeErrorCategory::MalformedInput
            }
            Self::FieldDecode { source, .. } => match source.category() {
                eth_valkyoth_codec::DecodeErrorCategory::MalformedInput => {
                    LegacyTransactionDecodeErrorCategory::MalformedInput
                }
                eth_valkyoth_codec::DecodeErrorCategory::ResourceExhaustion => {
                    LegacyTransactionDecodeErrorCategory::ResourceExhaustion
                }
                _ => LegacyTransactionDecodeErrorCategory::MalformedInput,
            },
        }
    }
}

impl fmt::Display for LegacyTransactionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LegacyTransactionDecodeError {}

/// Stable high-level legacy transaction decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyTransactionDecodeErrorCategory {
    /// Input is malformed for a legacy transaction.
    MalformedInput,
    /// A typed transaction envelope was supplied to a legacy-only decoder.
    WrongType,
    /// A future or unsupported transaction domain was encountered.
    Unsupported,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}

/// Decodes a canonical legacy transaction into unvalidated field domains.
pub fn decode_legacy_transaction<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedLegacyTransaction<'a>, LegacyTransactionDecodeError> {
    match decode_transaction_envelope(input, limits)
        .map_err(LegacyTransactionDecodeError::Envelope)?
    {
        TransactionEnvelope::Legacy(list) => decode_legacy_list(list, limits),
        TransactionEnvelope::Typed(typed) => Err(LegacyTransactionDecodeError::TypedEnvelope {
            type_byte: typed.transaction_type.get(),
        }),
    }
}

fn decode_legacy_list<'a>(
    list: RlpList<'a>,
    limits: DecodeLimits,
) -> Result<UnvalidatedLegacyTransaction<'a>, LegacyTransactionDecodeError> {
    if list.item_count() != LEGACY_TRANSACTION_FIELD_COUNT {
        return Err(LegacyTransactionDecodeError::WrongFieldCount {
            expected: LEGACY_TRANSACTION_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
    let nonce = Nonce::new(decode_u64_field(
        &mut fields,
        LegacyTransactionField::Nonce,
    )?);
    let gas_price = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        LegacyTransactionField::GasPrice,
    )?);
    let gas_limit = Gas::new(decode_u64_field(
        &mut fields,
        LegacyTransactionField::GasLimit,
    )?);
    let to = decode_to(next_scalar(&mut fields, LegacyTransactionField::To)?)?;
    let value = Wei::from_be_bytes(decode_u256_field(
        &mut fields,
        LegacyTransactionField::Value,
    )?);
    let input = next_scalar(&mut fields, LegacyTransactionField::Input)?.payload();
    limits
        .check_single_allocation_limit(input.len())
        .map_err(|source| field_error(LegacyTransactionField::Input, source))?;
    let v = decode_u256_field(&mut fields, LegacyTransactionField::V)?;
    let r = decode_u256_field(&mut fields, LegacyTransactionField::R)?;
    let s = decode_u256_field(&mut fields, LegacyTransactionField::S)?;

    Ok(UnvalidatedLegacyTransaction {
        nonce,
        gas_price,
        gas_limit,
        to,
        value,
        input,
        v,
        r,
        s,
    })
}

fn next_scalar<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: LegacyTransactionField,
) -> Result<RlpScalar<'a>, LegacyTransactionDecodeError> {
    let item = fields
        .next()
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(field_error(field, DecodeError::UnexpectedList)),
    }
}

fn decode_to(scalar: RlpScalar<'_>) -> Result<LegacyTransactionTo, LegacyTransactionDecodeError> {
    let payload = scalar.payload();
    if payload.is_empty() {
        return Ok(LegacyTransactionTo::Create);
    }
    let found = payload.len();
    let bytes: [u8; ADDRESS_BYTES] = payload
        .try_into()
        .map_err(|_| LegacyTransactionDecodeError::InvalidToLength { found })?;
    Ok(LegacyTransactionTo::Call(Address::from_bytes(bytes)))
}

fn decode_u64_field<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: LegacyTransactionField,
) -> Result<u64, LegacyTransactionDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_u64)
        .map_err(|source| field_error(field, source))
}

fn decode_u256_field<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: LegacyTransactionField,
) -> Result<[u8; 32], LegacyTransactionDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_be_bytes32)
        .map_err(|source| field_error(field, source))
}

const fn field_error(
    field: LegacyTransactionField,
    source: DecodeError,
) -> LegacyTransactionDecodeError {
    LegacyTransactionDecodeError::FieldDecode { field, source }
}
