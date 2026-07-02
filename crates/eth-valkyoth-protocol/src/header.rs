use eth_valkyoth_codec::{
    DecodeError, DecodeLimits, RlpInteger, RlpItem, RlpScalar, decode_rlp_list,
};
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256, BlockNumber, Gas, UnixTimestamp, Wei};

mod error;
pub use error::{BlockHeaderDecodeError, BlockHeaderDecodeErrorCategory, BlockHeaderField};

const ADDRESS_BYTES: usize = 20;
const B256_BYTES: usize = 32;
const BLOOM_BYTES: usize = 256;
const NONCE_BYTES: usize = 8;

/// Pre-London execution header field count.
pub const LEGACY_HEADER_FIELD_COUNT: usize = 15;
/// London execution header field count, including `base_fee_per_gas`.
pub const LONDON_HEADER_FIELD_COUNT: usize = 16;
/// Shanghai execution header field count, including `withdrawals_root`.
pub const SHANGHAI_HEADER_FIELD_COUNT: usize = 17;
/// Cancun execution header field count, including blob gas fields and parent
/// beacon block root.
pub const CANCUN_HEADER_FIELD_COUNT: usize = 20;
/// Prague execution header field count, including `requests_hash`.
pub const PRAGUE_HEADER_FIELD_COUNT: usize = 21;

/// Fork-specific execution header field set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeaderFieldSet {
    /// Header layout before EIP-1559.
    Legacy,
    /// Header layout with `base_fee_per_gas`.
    London,
    /// Header layout with `withdrawals_root`.
    Shanghai,
    /// Header layout with blob gas fields and `parent_beacon_block_root`.
    Cancun,
    /// Header layout with `requests_hash`.
    Prague,
}

impl HeaderFieldSet {
    /// Returns the exact RLP field count expected for this field set.
    #[must_use]
    pub const fn field_count(self) -> usize {
        match self {
            Self::Legacy => LEGACY_HEADER_FIELD_COUNT,
            Self::London => LONDON_HEADER_FIELD_COUNT,
            Self::Shanghai => SHANGHAI_HEADER_FIELD_COUNT,
            Self::Cancun => CANCUN_HEADER_FIELD_COUNT,
            Self::Prague => PRAGUE_HEADER_FIELD_COUNT,
        }
    }

    const fn has_base_fee(self) -> bool {
        !matches!(self, Self::Legacy)
    }

    const fn has_withdrawals_root(self) -> bool {
        matches!(self, Self::Shanghai | Self::Cancun | Self::Prague)
    }

    const fn has_cancun_fields(self) -> bool {
        matches!(self, Self::Cancun | Self::Prague)
    }

    const fn has_requests_hash(self) -> bool {
        matches!(self, Self::Prague)
    }
}

/// Domain-separated execution block hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockHash(B256);

impl BlockHash {
    /// Creates a block-hash domain value from raw bytes.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the raw 32-byte hash.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

impl From<BlockHash> for B256 {
    fn from(value: BlockHash) -> Self {
        value.to_b256()
    }
}

/// Borrowed execution-layer header decoded only into field domains.
///
/// This type is intentionally unvalidated. It does not prove ancestry, ommers
/// hash, state root, transaction root, receipt root, bloom correctness, gas
/// accounting, base-fee calculation, withdrawals root, blob-gas accounting,
/// parent beacon root, requests hash, proof roots, or fork activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedBlockHeader<'a> {
    encoded_rlp: &'a [u8],
    /// Field layout used to decode the header.
    pub field_set: HeaderFieldSet,
    /// Parent block hash.
    pub parent_hash: B256,
    /// Ommers/uncles hash.
    pub ommers_hash: B256,
    /// Block beneficiary/coinbase.
    pub beneficiary: Address,
    /// Post-state root.
    pub state_root: B256,
    /// Transactions trie root.
    pub transactions_root: B256,
    /// Receipts trie root.
    pub receipts_root: B256,
    /// Logs bloom filter.
    pub logs_bloom: LogsBloom,
    /// Raw canonical U256 difficulty field.
    pub difficulty: [u8; 32],
    /// Block number.
    pub number: BlockNumber,
    /// Gas limit.
    pub gas_limit: Gas,
    /// Gas used.
    pub gas_used: Gas,
    /// Header timestamp.
    pub timestamp: UnixTimestamp,
    /// Borrowed extra data.
    pub extra_data: &'a [u8],
    /// Mix hash or post-merge prev_randao.
    pub mix_hash: B256,
    /// Header nonce bytes.
    pub nonce: [u8; 8],
    /// London base fee per gas.
    pub base_fee_per_gas: Option<Wei>,
    /// Shanghai withdrawals root.
    pub withdrawals_root: Option<B256>,
    /// Cancun blob gas used.
    pub blob_gas_used: Option<Gas>,
    /// Cancun excess blob gas.
    pub excess_blob_gas: Option<Gas>,
    /// Cancun parent beacon block root.
    pub parent_beacon_block_root: Option<B256>,
    /// Prague requests hash.
    pub requests_hash: Option<B256>,
}

impl UnvalidatedBlockHeader<'_> {
    /// Returns the exact canonical RLP bytes that were decoded.
    #[must_use]
    pub const fn encoded_rlp(&self) -> &[u8] {
        self.encoded_rlp
    }

    /// Hashes the exact canonical header RLP with the caller-provided Keccak
    /// implementation and returns a block-hash domain value.
    #[must_use]
    pub fn hash_with<H>(&self, hasher: H) -> BlockHash
    where
        H: Keccak256,
    {
        BlockHash::from_b256(hash_one(hasher, self.encoded_rlp))
    }
}

/// Ethereum logs bloom filter bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogsBloom([u8; BLOOM_BYTES]);

impl LogsBloom {
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

/// Decodes one canonical execution block header under explicit limits.
///
/// The selected [`HeaderFieldSet`] controls which optional fork fields must be
/// present. This function is syntactic: successfully decoding a header does not
/// prove block validity, fork activation, root correctness, or consensus-layer
/// commitments.
pub fn decode_block_header<'a>(
    input: &'a [u8],
    field_set: HeaderFieldSet,
    limits: DecodeLimits,
) -> Result<UnvalidatedBlockHeader<'a>, BlockHeaderDecodeError> {
    let list = decode_rlp_list(input, limits).map_err(BlockHeaderDecodeError::Decode)?;
    let expected = field_set.field_count();
    let found = list.item_count();
    if found != expected {
        return Err(BlockHeaderDecodeError::WrongFieldCount { expected, found });
    }

    let mut fields = list.items();
    let header = UnvalidatedBlockHeader {
        encoded_rlp: input,
        field_set,
        parent_hash: decode_b256(&mut fields, BlockHeaderField::ParentHash)?,
        ommers_hash: decode_b256(&mut fields, BlockHeaderField::OmmersHash)?,
        beneficiary: decode_address(&mut fields, BlockHeaderField::Beneficiary)?,
        state_root: decode_b256(&mut fields, BlockHeaderField::StateRoot)?,
        transactions_root: decode_b256(&mut fields, BlockHeaderField::TransactionsRoot)?,
        receipts_root: decode_b256(&mut fields, BlockHeaderField::ReceiptsRoot)?,
        logs_bloom: decode_logs_bloom(&mut fields)?,
        difficulty: decode_u256(&mut fields, BlockHeaderField::Difficulty)?,
        number: BlockNumber::new(decode_u64(&mut fields, BlockHeaderField::Number)?),
        gas_limit: Gas::new(decode_u64(&mut fields, BlockHeaderField::GasLimit)?),
        gas_used: Gas::new(decode_u64(&mut fields, BlockHeaderField::GasUsed)?),
        timestamp: UnixTimestamp::new(decode_u64(&mut fields, BlockHeaderField::Timestamp)?),
        extra_data: next_scalar(&mut fields, BlockHeaderField::ExtraData)?.payload(),
        mix_hash: decode_b256(&mut fields, BlockHeaderField::MixHash)?,
        nonce: decode_fixed::<NONCE_BYTES>(&mut fields, BlockHeaderField::Nonce)?,
        base_fee_per_gas: decode_optional_wei(
            &mut fields,
            field_set.has_base_fee(),
            BlockHeaderField::BaseFeePerGas,
        )?,
        withdrawals_root: decode_optional_b256(
            &mut fields,
            field_set.has_withdrawals_root(),
            BlockHeaderField::WithdrawalsRoot,
        )?,
        blob_gas_used: decode_optional_gas(
            &mut fields,
            field_set.has_cancun_fields(),
            BlockHeaderField::BlobGasUsed,
        )?,
        excess_blob_gas: decode_optional_gas(
            &mut fields,
            field_set.has_cancun_fields(),
            BlockHeaderField::ExcessBlobGas,
        )?,
        parent_beacon_block_root: decode_optional_b256(
            &mut fields,
            field_set.has_cancun_fields(),
            BlockHeaderField::ParentBeaconBlockRoot,
        )?,
        requests_hash: decode_optional_b256(
            &mut fields,
            field_set.has_requests_hash(),
            BlockHeaderField::RequestsHash,
        )?,
    };
    Ok(header)
}

fn next_scalar<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: BlockHeaderField,
) -> Result<RlpScalar<'a>, BlockHeaderDecodeError> {
    let item = fields
        .next()
        .ok_or(BlockHeaderDecodeError::FieldDecode {
            field,
            source: DecodeError::Malformed,
        })?
        .map_err(|source| BlockHeaderDecodeError::FieldDecode { field, source })?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(BlockHeaderDecodeError::FieldDecode {
            field,
            source: DecodeError::UnexpectedList,
        }),
    }
}

fn decode_u64<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: BlockHeaderField,
) -> Result<u64, BlockHeaderDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_u64)
        .map_err(|source| BlockHeaderDecodeError::FieldDecode { field, source })
}

fn decode_u256<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: BlockHeaderField,
) -> Result<[u8; 32], BlockHeaderDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_be_bytes32)
        .map_err(|source| BlockHeaderDecodeError::FieldDecode { field, source })
}

fn decode_wei<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: BlockHeaderField,
) -> Result<Wei, BlockHeaderDecodeError> {
    decode_u256(fields, field).map(Wei::from_be_bytes)
}

fn decode_fixed<'a, const N: usize>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: BlockHeaderField,
) -> Result<[u8; N], BlockHeaderDecodeError> {
    let scalar = next_scalar(fields, field)?;
    scalar
        .payload()
        .try_into()
        .map_err(|_| BlockHeaderDecodeError::InvalidFieldLength {
            field,
            expected: N,
            found: scalar.payload().len(),
        })
}

fn decode_b256<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: BlockHeaderField,
) -> Result<B256, BlockHeaderDecodeError> {
    decode_fixed::<B256_BYTES>(fields, field).map(B256::from_bytes)
}

fn decode_address<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: BlockHeaderField,
) -> Result<Address, BlockHeaderDecodeError> {
    decode_fixed::<ADDRESS_BYTES>(fields, field).map(Address::from_bytes)
}

fn decode_logs_bloom<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
) -> Result<LogsBloom, BlockHeaderDecodeError> {
    decode_fixed::<BLOOM_BYTES>(fields, BlockHeaderField::LogsBloom).map(LogsBloom::from_bytes)
}

fn decode_optional_b256<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    present: bool,
    field: BlockHeaderField,
) -> Result<Option<B256>, BlockHeaderDecodeError> {
    if present {
        decode_b256(fields, field).map(Some)
    } else {
        Ok(None)
    }
}

fn decode_optional_gas<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    present: bool,
    field: BlockHeaderField,
) -> Result<Option<Gas>, BlockHeaderDecodeError> {
    if present {
        decode_u64(fields, field).map(Gas::new).map(Some)
    } else {
        Ok(None)
    }
}

fn decode_optional_wei<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    present: bool,
    field: BlockHeaderField,
) -> Result<Option<Wei>, BlockHeaderDecodeError> {
    if present {
        decode_wei(fields, field).map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[path = "header_tests.rs"]
mod tests;
