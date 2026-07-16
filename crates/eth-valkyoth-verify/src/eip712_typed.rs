use core::fmt;

use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256, ChainId};

use crate::eip712_signing_digest;

#[cfg(feature = "json")]
#[path = "eip712_json.rs"]
mod eip712_json;
#[path = "eip712_schema.rs"]
mod eip712_schema;
#[path = "eip712_type_graph.rs"]
mod type_graph;
#[path = "eip712_typed_helpers.rs"]
mod typed_helpers;
#[path = "eip712_value_encode.rs"]
mod value_encode;
#[cfg(feature = "json")]
pub use eip712_json::{Eip712JsonError, Eip712JsonLimits, eip712_json_typed_data_signing_digest};

use type_graph::{collect_reachable_types, next_dependency};
use typed_helpers::{
    SliceWriter, encode_domain_type, find_struct, find_value, update_domain_fields,
    validate_values, write_struct_type,
};
use value_encode::encode_value_word;

use self::eip712_schema::ValidatedSchema;

const WORD_BYTES: usize = 32;
const ADDRESS_PADDING_BYTES: usize = 12;
const MAX_TYPE_DEPTH: usize = 32;
/// Maximum number of EIP-712 struct types admitted by the bounded encoder.
pub const EIP712_MAX_TYPES: usize = 64;
/// Maximum number of fields admitted in one borrowed EIP-712 struct type.
pub const EIP712_MAX_FIELDS_PER_TYPE: usize = 64;
/// Maximum number of named values admitted in one borrowed EIP-712 struct.
pub const EIP712_MAX_VALUES_PER_STRUCT: usize = 64;
/// Maximum elements admitted at any borrowed EIP-712 array dimension.
pub const EIP712_MAX_ARRAY_ITEMS: usize = 256;

/// One EIP-712 struct type definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eip712StructType<'a> {
    /// Struct name.
    pub name: &'a str,
    /// Struct fields in declaration order.
    pub fields: &'a [Eip712Field<'a>],
}

/// One EIP-712 struct field definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eip712Field<'a> {
    /// Field name.
    pub name: &'a str,
    /// Canonical EIP-712 type string, for example `uint256`, `Person`, or
    /// `Person[]`.
    pub type_name: &'a str,
}

/// One named EIP-712 value inside a struct instance.
///
/// Debug formatting is redacted and the value is intentionally not `Copy` or
/// `Clone` because unpublished signing preimages can be secret-bearing.
#[derive(Eq, PartialEq)]
pub struct Eip712Value<'a> {
    /// Field name.
    pub name: &'a str,
    /// Field value.
    pub value: Eip712ValueKind<'a>,
}

/// Borrowed EIP-712 value representation.
///
/// Debug formatting reveals only the variant and never the contained value.
#[derive(Eq, PartialEq)]
pub enum Eip712ValueKind<'a> {
    /// Boolean value.
    Bool(bool),
    /// Ethereum address value.
    Address(Address),
    /// Unsigned integer supplied as a `u64`.
    Uint64(u64),
    /// Unsigned 256-bit integer, already encoded as big-endian bytes.
    Uint256([u8; 32]),
    /// Signed 256-bit integer, already sign-extended as big-endian bytes.
    Int256([u8; 32]),
    /// Fixed-size `bytes1` through `bytes32`.
    FixedBytes(&'a [u8]),
    /// Dynamic `bytes`.
    Bytes(&'a [u8]),
    /// UTF-8 `string`.
    String(&'a str),
    /// Nested struct values.
    Struct(&'a [Eip712Value<'a>]),
    /// Array element values.
    Array(&'a [Eip712ValueKind<'a>]),
}

impl fmt::Debug for Eip712Value<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Eip712Value(<redacted>)")
    }
}

impl fmt::Debug for Eip712ValueKind<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = match self {
            Self::Bool(_) => "Bool",
            Self::Address(_) => "Address",
            Self::Uint64(_) => "Uint64",
            Self::Uint256(_) => "Uint256",
            Self::Int256(_) => "Int256",
            Self::FixedBytes(_) => "FixedBytes",
            Self::Bytes(_) => "Bytes",
            Self::String(_) => "String",
            Self::Struct(_) => "Struct",
            Self::Array(_) => "Array",
        };
        write!(formatter, "Eip712ValueKind::{variant}(<redacted>)")
    }
}

/// EIP-712 domain data admitted by the first-party encoder.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Eip712DomainData<'a> {
    /// Optional user-readable signing domain name.
    pub name: Option<&'a str>,
    /// Optional signing domain version.
    pub version: Option<&'a str>,
    /// Optional EIP-155 chain ID.
    ///
    /// [`ChainId`] can represent zero for compatibility with legacy replay
    /// domains. Callers that require EIP-712 replay protection should reject
    /// `ChainId(0)` before computing a domain separator.
    pub chain_id: Option<ChainId>,
    /// Optional verifying contract address.
    pub verifying_contract: Option<Address>,
    /// Optional domain salt.
    pub salt: Option<B256>,
}

impl<'a> Eip712DomainData<'a> {
    /// Creates an empty domain.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            name: None,
            version: None,
            chain_id: None,
            verifying_contract: None,
            salt: None,
        }
    }
}

/// EIP-712 typed-data encoder failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Eip712EncodeError {
    /// Output scratch buffer is too short.
    OutputTooShort,
    /// A named struct type was not present in the schema.
    UnknownStruct,
    /// A named value was missing from a struct instance.
    MissingValue,
    /// A type string is malformed or unsupported by this release.
    InvalidType,
    /// A value does not match the declared field type.
    TypeMismatch,
    /// Type graph or value graph exceeded the recursion limit.
    RecursionLimit,
    /// The schema exceeds the bounded type-count limit.
    SchemaTooLarge,
    /// A field, value, or array collection exceeds the release hard limit.
    ResourceLimit,
    /// Multiple struct definitions use the same name.
    DuplicateType,
    /// Multiple fields in one struct use the same name.
    DuplicateField,
    /// Multiple supplied values use the same name.
    DuplicateValue,
}

impl fmt::Display for Eip712EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::OutputTooShort => "EIP-712 output buffer is too short",
            Self::UnknownStruct => "EIP-712 struct type is not in the schema",
            Self::MissingValue => "EIP-712 struct value is missing a field",
            Self::InvalidType => "EIP-712 type string is invalid or unsupported",
            Self::TypeMismatch => "EIP-712 value does not match its declared type",
            Self::RecursionLimit => "EIP-712 recursion limit was exceeded",
            Self::SchemaTooLarge => "EIP-712 schema exceeds the type-count limit",
            Self::ResourceLimit => "EIP-712 collection exceeds the release limit",
            Self::DuplicateType => "EIP-712 schema contains a duplicate struct type",
            Self::DuplicateField => "EIP-712 struct contains a duplicate field name",
            Self::DuplicateValue => "EIP-712 struct contains a duplicate value name",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Eip712EncodeError {}

/// Encodes `encodeType(primaryType)` into `output`.
///
/// Schemas larger than [`EIP712_MAX_TYPES`] are rejected before dependency
/// traversal. Struct and field identifiers must follow EIP-712 identifier
/// syntax, and duplicate type or field names are rejected before output
/// mutation. Each reachable type is visited once before lexical emission.
pub fn encode_eip712_type(
    types: &[Eip712StructType<'_>],
    primary_type: &str,
    output: &mut [u8],
) -> Result<usize, Eip712EncodeError> {
    let schema = ValidatedSchema::try_new(types)?;
    encode_eip712_type_validated(schema.types(), primary_type, output)
}

fn encode_eip712_type_validated(
    types: &[Eip712StructType<'_>],
    primary_type: &str,
    output: &mut [u8],
) -> Result<usize, Eip712EncodeError> {
    let mut writer = SliceWriter::new(output);
    let primary = find_struct(types, primary_type)?;
    let reachable = collect_reachable_types(types, primary_type)?;
    write_struct_type(primary, &mut writer)?;
    let mut previous = None::<&str>;
    loop {
        let next = next_dependency(types, primary_type, previous, &reachable)?;
        let Some(dependency) = next else {
            break;
        };
        write_struct_type(dependency, &mut writer)?;
        previous = Some(dependency.name);
    }
    Ok(writer.len())
}

/// Computes `keccak256(encodeType(primaryType))`.
pub fn eip712_type_hash<H>(
    types: &[Eip712StructType<'_>],
    primary_type: &str,
    scratch: &mut [u8],
) -> Result<B256, Eip712EncodeError>
where
    H: Default + Keccak256,
{
    let mut schema = ValidatedSchema::try_new(types)?;
    schema.type_hash::<H>(primary_type, scratch)
}

/// Encodes EIP-712 member data for one struct instance.
///
/// Schema and value names are validated once before encoding. Every borrowed
/// array dimension is capped at [`EIP712_MAX_ARRAY_ITEMS`]. If a field fails
/// after encoding starts, the selected output region is cleared before the
/// error is returned.
pub fn encode_eip712_data<H>(
    types: &[Eip712StructType<'_>],
    primary_type: &str,
    values: &[Eip712Value<'_>],
    output: &mut [u8],
    type_scratch: &mut [u8],
) -> Result<usize, Eip712EncodeError>
where
    H: Default + Keccak256,
{
    let mut schema = ValidatedSchema::try_new(types)?;
    validate_values(values)?;
    let ty = find_struct(schema.types(), primary_type)?;
    let len = ty
        .fields
        .len()
        .checked_mul(WORD_BYTES)
        .ok_or(Eip712EncodeError::OutputTooShort)?;
    let target = output
        .get_mut(..len)
        .ok_or(Eip712EncodeError::OutputTooShort)?;
    for (field, slot) in ty.fields.iter().zip(target.chunks_exact_mut(WORD_BYTES)) {
        let result = (|| {
            let value = find_value(values, field.name)?;
            let mut word = [0_u8; WORD_BYTES];
            encode_value_word::<H>(
                &mut schema,
                field.type_name,
                value,
                &mut word,
                type_scratch,
                0,
            )?;
            slot.copy_from_slice(&word);
            Ok(())
        })();
        if let Err(error) = result {
            target.fill(0);
            return Err(error);
        }
    }
    Ok(len)
}

/// Computes EIP-712 `hashStruct(value)`.
///
/// Schema validation runs once at this public boundary, and type hashes are
/// cached in a fixed-size context throughout recursive struct and array
/// hashing.
pub fn eip712_hash_struct<H>(
    types: &[Eip712StructType<'_>],
    primary_type: &str,
    values: &[Eip712Value<'_>],
    type_scratch: &mut [u8],
) -> Result<B256, Eip712EncodeError>
where
    H: Default + Keccak256,
{
    let mut schema = ValidatedSchema::try_new(types)?;
    hash_struct_inner::<H>(&mut schema, primary_type, values, type_scratch, 0)
}

/// Computes the EIP-712 domain separator for admitted domain fields.
pub fn eip712_domain_separator<H>(
    domain: Eip712DomainData<'_>,
    type_scratch: &mut [u8],
) -> Result<B256, Eip712EncodeError>
where
    H: Default + Keccak256,
{
    let type_len = encode_domain_type(domain, type_scratch)?;
    let type_bytes = type_scratch
        .get(..type_len)
        .ok_or(Eip712EncodeError::OutputTooShort)?;
    let type_hash = hash_one(H::default(), type_bytes);
    let mut hasher = H::default();
    hasher.update(&type_hash.to_bytes());
    update_domain_fields::<H>(domain, &mut hasher);
    Ok(hasher.finalize())
}

/// Computes `keccak256("\x19\x01" || domainSeparator || hashStruct(message))`.
pub fn eip712_typed_data_signing_digest<H>(
    domain: Eip712DomainData<'_>,
    types: &[Eip712StructType<'_>],
    primary_type: &str,
    values: &[Eip712Value<'_>],
    type_scratch: &mut [u8],
) -> Result<B256, Eip712EncodeError>
where
    H: Default + Keccak256,
{
    let domain_separator = eip712_domain_separator::<H>(domain, type_scratch)?;
    let message_hash = eip712_hash_struct::<H>(types, primary_type, values, type_scratch)?;
    Ok(eip712_signing_digest(
        domain_separator,
        message_hash,
        H::default(),
    ))
}

fn hash_struct_inner<H>(
    schema: &mut ValidatedSchema<'_>,
    primary_type: &str,
    values: &[Eip712Value<'_>],
    type_scratch: &mut [u8],
    depth: usize,
) -> Result<B256, Eip712EncodeError>
where
    H: Default + Keccak256,
{
    if depth >= MAX_TYPE_DEPTH {
        return Err(Eip712EncodeError::RecursionLimit);
    }
    validate_values(values)?;
    let ty = find_struct(schema.types(), primary_type)?;
    let type_hash = schema.type_hash::<H>(primary_type, type_scratch)?;
    let mut hasher = H::default();
    hasher.update(&type_hash.to_bytes());
    for field in ty.fields {
        let value = find_value(values, field.name)?;
        let mut word = [0_u8; WORD_BYTES];
        encode_value_word::<H>(
            schema,
            field.type_name,
            value,
            &mut word,
            type_scratch,
            depth.saturating_add(1),
        )?;
        hasher.update(&word);
    }
    Ok(hasher.finalize())
}

#[cfg(test)]
#[path = "eip712_typed_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "eip712_security_tests.rs"]
mod security_tests;
