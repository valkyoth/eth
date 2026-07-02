use eth_valkyoth_codec::{DecodeError, DecodeLimits, RlpInteger, RlpItem, RlpList, RlpScalar};
use eth_valkyoth_primitives::{Address, B256, Gas, TransactionType};

use crate::eip2718::{
    EIP_2718_MAX_TYPED_PREFIX as SHARED_EIP_2718_MAX_TYPED_PREFIX,
    EIP_2718_RESERVED_PREFIX as SHARED_EIP_2718_RESERVED_PREFIX,
    EIP_2718_SCALAR_PREFIX_START as SHARED_EIP_2718_SCALAR_PREFIX_START,
    EIP_2718_TYPED_ZERO_PREFIX as SHARED_EIP_2718_TYPED_ZERO_PREFIX, Eip2718Prefix,
    LEGACY_PREFIX_START as SHARED_LEGACY_PREFIX_START, classify_eip2718_prefix,
};

mod error;
pub use error::{ReceiptDecodeError, ReceiptDecodeErrorCategory, ReceiptField};

const ADDRESS_BYTES: usize = 20;
const B256_BYTES: usize = 32;
const BLOOM_BYTES: usize = 256;
const LOG_FIELD_COUNT: usize = 3;

/// Number of fields in a legacy or typed receipt payload.
pub const RECEIPT_FIELD_COUNT: usize = 4;
/// EIP-2718 byte value reserved by this crate for the legacy receipt domain.
pub const EIP_2718_TYPED_ZERO_RECEIPT_PREFIX: u8 = SHARED_EIP_2718_TYPED_ZERO_PREFIX;
/// Largest single-byte EIP-2718 typed receipt prefix.
pub const EIP_2718_MAX_TYPED_RECEIPT_PREFIX: u8 = SHARED_EIP_2718_MAX_TYPED_PREFIX;
/// First RLP scalar prefix that cannot be a typed receipt or legacy list.
pub const EIP_2718_RECEIPT_SCALAR_PREFIX_START: u8 = SHARED_EIP_2718_SCALAR_PREFIX_START;
/// First byte used by canonical RLP short-list legacy receipts.
pub const LEGACY_RECEIPT_PREFIX_START: u8 = SHARED_LEGACY_PREFIX_START;
/// Prefix reserved by EIP-2718 as a future extension sentinel.
pub const EIP_2718_RESERVED_RECEIPT_PREFIX: u8 = SHARED_EIP_2718_RESERVED_PREFIX;

/// Borrowed receipt envelope shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptEnvelope<'a> {
    /// A legacy receipt represented as one exact RLP list.
    Legacy(RlpList<'a>),
    /// An EIP-2718 typed receipt with an opaque payload.
    Typed(TypedReceiptEnvelope<'a>),
}

/// Borrowed EIP-2718 typed receipt shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypedReceiptEnvelope<'a> {
    /// EIP-2718 receipt type byte.
    pub receipt_type: TransactionType,
    /// Opaque receipt payload.
    pub payload: &'a [u8],
}

/// Borrowed receipt decoded only into field domains.
///
/// This type is intentionally unvalidated. It does not prove transaction
/// execution, receipt-trie inclusion, block receipt-root membership, log
/// semantics, cumulative-gas monotonicity, or that a typed receipt matches the
/// transaction type at the same block index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedReceipt<'a> {
    encoded_payload: &'a [u8],
    /// Receipt domain.
    pub envelope: ReceiptKind,
    /// Status code or pre-Byzantium state root.
    pub status_or_state_root: ReceiptStatusOrStateRoot,
    /// Cumulative gas used after this transaction in the containing block.
    pub cumulative_gas_used: Gas,
    /// Logs bloom filter.
    pub logs_bloom: ReceiptLogsBloom,
    /// Borrowed logs list.
    pub logs: ReceiptLogs<'a>,
}

impl UnvalidatedReceipt<'_> {
    /// Returns the exact canonical RLP payload bytes that were decoded.
    ///
    /// For legacy receipts this is the whole receipt. For typed receipts this
    /// excludes the EIP-2718 type byte.
    #[must_use]
    pub const fn encoded_payload(&self) -> &[u8] {
        self.encoded_payload
    }
}

/// Receipt domain after envelope classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptKind {
    /// Legacy RLP receipt.
    Legacy,
    /// EIP-2718 typed receipt.
    Typed(TransactionType),
}

/// Receipt status code or pre-Byzantium state root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptStatusOrStateRoot {
    /// Transaction failed.
    Failure,
    /// Transaction succeeded.
    Success,
    /// Pre-Byzantium intermediate state root.
    StateRoot(B256),
}

/// Ethereum receipt logs bloom filter bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptLogsBloom([u8; BLOOM_BYTES]);

impl ReceiptLogsBloom {
    /// Creates a logs bloom from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; BLOOM_BYTES]) -> Self {
        Self(bytes)
    }

    /// Returns the raw logs-bloom bytes.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; BLOOM_BYTES] {
        self.0
    }
}

/// Borrowed receipt logs list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptLogs<'a> {
    list: RlpList<'a>,
}

impl<'a> ReceiptLogs<'a> {
    /// Returns the number of logs.
    #[must_use]
    pub const fn len(self) -> usize {
        self.list.item_count()
    }

    /// Returns true when there are no logs.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.list.is_empty()
    }

    /// Returns an iterator over decoded log entries.
    #[must_use]
    pub const fn entries(self) -> ReceiptLogEntries<'a> {
        ReceiptLogEntries {
            items: self.list.items(),
        }
    }
}

/// Borrowed receipt log entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptLog<'a> {
    /// Contract address that emitted the log.
    pub address: Address,
    /// Borrowed topics list.
    pub topics: ReceiptLogTopics<'a>,
    /// Borrowed log data bytes.
    pub data: &'a [u8],
}

/// Borrowed log topics list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptLogTopics<'a> {
    list: RlpList<'a>,
}

impl<'a> ReceiptLogTopics<'a> {
    /// Returns the number of topics.
    #[must_use]
    pub const fn len(self) -> usize {
        self.list.item_count()
    }

    /// Returns true when the log has no topics.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.list.is_empty()
    }

    /// Returns an iterator over decoded topics.
    #[must_use]
    pub const fn topics(self) -> ReceiptLogTopicItems<'a> {
        ReceiptLogTopicItems {
            items: self.list.items(),
        }
    }
}

/// Iterator over receipt logs.
#[derive(Clone, Debug)]
pub struct ReceiptLogEntries<'a> {
    items: eth_valkyoth_codec::RlpListItems<'a>,
}

impl<'a> Iterator for ReceiptLogEntries<'a> {
    type Item = Result<ReceiptLog<'a>, ReceiptDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(decode_log_item)
    }
}

impl core::iter::FusedIterator for ReceiptLogEntries<'_> {}

/// Iterator over log topics.
#[derive(Clone, Debug)]
pub struct ReceiptLogTopicItems<'a> {
    items: eth_valkyoth_codec::RlpListItems<'a>,
}

impl Iterator for ReceiptLogTopicItems<'_> {
    type Item = Result<B256, ReceiptDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(decode_topic_item)
    }
}

impl core::iter::FusedIterator for ReceiptLogTopicItems<'_> {}

/// Classifies a borrowed receipt envelope under explicit decode limits.
pub fn decode_receipt_envelope<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<ReceiptEnvelope<'a>, ReceiptDecodeError> {
    limits
        .check_input_len(input.len())
        .map_err(|source| field_error(ReceiptField::Payload, source))?;

    let Some(prefix) = classify_eip2718_prefix(input) else {
        return Err(ReceiptDecodeError::EmptyInput);
    };

    match prefix {
        Eip2718Prefix::TypedZero => Err(ReceiptDecodeError::UnsupportedReceiptType {
            type_byte: EIP_2718_TYPED_ZERO_RECEIPT_PREFIX,
        }),
        Eip2718Prefix::Typed { type_byte, payload } => {
            let receipt_type = TransactionType::try_new_typed(type_byte)
                .map_err(|_| ReceiptDecodeError::UnsupportedReceiptType { type_byte })?;
            Ok(ReceiptEnvelope::Typed(TypedReceiptEnvelope {
                receipt_type,
                payload,
            }))
        }
        Eip2718Prefix::ScalarPrefix { prefix } => Err(ReceiptDecodeError::ScalarPrefix { prefix }),
        Eip2718Prefix::Legacy => {
            let list = eth_valkyoth_codec::decode_rlp_list(input, limits)
                .map_err(|source| field_error(ReceiptField::Payload, source))?;
            Ok(ReceiptEnvelope::Legacy(list))
        }
        Eip2718Prefix::Reserved => Err(ReceiptDecodeError::ReservedPrefix),
    }
}

/// Decodes a legacy or typed receipt into unvalidated field domains.
pub fn decode_receipt<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedReceipt<'a>, ReceiptDecodeError> {
    match decode_receipt_envelope(input, limits)? {
        ReceiptEnvelope::Legacy(list) => decode_receipt_list(list, input, ReceiptKind::Legacy),
        ReceiptEnvelope::Typed(typed) => {
            let list = eth_valkyoth_codec::decode_rlp_list(typed.payload, limits)
                .map_err(|source| field_error(ReceiptField::Payload, source))?;
            decode_receipt_list(list, typed.payload, ReceiptKind::Typed(typed.receipt_type))
        }
    }
}

fn decode_receipt_list<'a>(
    list: RlpList<'a>,
    encoded_payload: &'a [u8],
    envelope: ReceiptKind,
) -> Result<UnvalidatedReceipt<'a>, ReceiptDecodeError> {
    if list.item_count() != RECEIPT_FIELD_COUNT {
        return Err(ReceiptDecodeError::WrongFieldCount {
            expected: RECEIPT_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
    let status_or_state_root =
        decode_status_or_root(next_scalar(&mut fields, ReceiptField::StatusOrStateRoot)?)?;
    let cumulative_gas_used = Gas::new(decode_u64(&mut fields, ReceiptField::CumulativeGasUsed)?);
    let logs_bloom = ReceiptLogsBloom::from_bytes(decode_fixed::<BLOOM_BYTES>(
        &mut fields,
        ReceiptField::LogsBloom,
    )?);
    let logs = decode_logs(next_list(&mut fields, ReceiptField::Logs)?)?;

    Ok(UnvalidatedReceipt {
        encoded_payload,
        envelope,
        status_or_state_root,
        cumulative_gas_used,
        logs_bloom,
        logs,
    })
}

fn decode_status_or_root(
    scalar: RlpScalar<'_>,
) -> Result<ReceiptStatusOrStateRoot, ReceiptDecodeError> {
    let payload = scalar.payload();
    match payload {
        [] => Ok(ReceiptStatusOrStateRoot::Failure),
        [1] => Ok(ReceiptStatusOrStateRoot::Success),
        bytes if bytes.len() == B256_BYTES => Ok(ReceiptStatusOrStateRoot::StateRoot(
            B256::from_bytes(bytes.try_into().map_err(|_| {
                ReceiptDecodeError::InvalidStatusOrStateRoot { found: bytes.len() }
            })?),
        )),
        _ => Err(ReceiptDecodeError::InvalidStatusOrStateRoot {
            found: payload.len(),
        }),
    }
}

fn decode_logs(list: RlpList<'_>) -> Result<ReceiptLogs<'_>, ReceiptDecodeError> {
    for item in list.items() {
        let _ = decode_log_item(item)?;
    }
    Ok(ReceiptLogs { list })
}

fn decode_log_item(
    item: Result<RlpItem<'_>, DecodeError>,
) -> Result<ReceiptLog<'_>, ReceiptDecodeError> {
    let item = item.map_err(|source| field_error(ReceiptField::Logs, source))?;
    let RlpItem::List(list) = item else {
        return Err(field_error(
            ReceiptField::Logs,
            DecodeError::UnexpectedScalar,
        ));
    };
    if list.item_count() != LOG_FIELD_COUNT {
        return Err(ReceiptDecodeError::InvalidLogFieldCount {
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
    let address = Address::from_bytes(decode_fixed::<ADDRESS_BYTES>(
        &mut fields,
        ReceiptField::LogAddress,
    )?);
    let topics = decode_topics(next_list(&mut fields, ReceiptField::LogTopics)?)?;
    let data = next_scalar(&mut fields, ReceiptField::LogData)?.payload();
    Ok(ReceiptLog {
        address,
        topics,
        data,
    })
}

fn decode_topics(list: RlpList<'_>) -> Result<ReceiptLogTopics<'_>, ReceiptDecodeError> {
    for item in list.items() {
        let _ = decode_topic_item(item)?;
    }
    Ok(ReceiptLogTopics { list })
}

fn decode_topic_item(item: Result<RlpItem<'_>, DecodeError>) -> Result<B256, ReceiptDecodeError> {
    let item = item.map_err(|source| field_error(ReceiptField::LogTopics, source))?;
    let RlpItem::Scalar(scalar) = item else {
        return Err(field_error(
            ReceiptField::LogTopics,
            DecodeError::UnexpectedList,
        ));
    };
    let found = scalar.payload().len();
    let bytes: [u8; B256_BYTES] = scalar
        .payload()
        .try_into()
        .map_err(|_| ReceiptDecodeError::InvalidLogTopicLength { found })?;
    Ok(B256::from_bytes(bytes))
}

fn decode_u64<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: ReceiptField,
) -> Result<u64, ReceiptDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_u64)
        .map_err(|source| field_error(field, source))
}

fn decode_fixed<'a, const N: usize>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: ReceiptField,
) -> Result<[u8; N], ReceiptDecodeError> {
    let scalar = next_scalar(fields, field)?;
    let found = scalar.payload().len();
    scalar.payload().try_into().map_err(|_| match field {
        ReceiptField::LogsBloom => ReceiptDecodeError::InvalidLogsBloomLength { found },
        ReceiptField::LogAddress => ReceiptDecodeError::InvalidLogAddressLength { found },
        _ => field_error(field, DecodeError::Malformed),
    })
}

fn next_scalar<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: ReceiptField,
) -> Result<RlpScalar<'a>, ReceiptDecodeError> {
    let item = fields
        .next()
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(field_error(field, DecodeError::UnexpectedList)),
    }
}

fn next_list<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: ReceiptField,
) -> Result<RlpList<'a>, ReceiptDecodeError> {
    let item = fields
        .next()
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::List(list) => Ok(list),
        RlpItem::Scalar(_) => Err(field_error(field, DecodeError::UnexpectedScalar)),
    }
}

const fn field_error(field: ReceiptField, source: DecodeError) -> ReceiptDecodeError {
    ReceiptDecodeError::FieldDecode { field, source }
}

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod tests;
