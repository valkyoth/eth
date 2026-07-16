use core::fmt;

use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;
use std::string::{String, ToString};
use std::vec::Vec;

#[path = "eip712_json_dom.rs"]
mod dom;
#[path = "eip712_json_scalar.rs"]
mod scalar;

use dom::{Json, Object};
use scalar::{
    check_depth, check_len, encode_json_numeric_or_bytes, parse_address, parse_b256, parse_chain_id,
};

use super::typed_helpers::{find_struct, parse_array_type, validate_identifier};
use super::{
    ADDRESS_PADDING_BYTES, Eip712DomainData, Eip712EncodeError, Eip712Field, Eip712StructType,
    MAX_TYPE_DEPTH, WORD_BYTES, eip712_domain_separator, eip712_type_hash,
};
use crate::eip712_signing_digest;

/// Limits for optional EIP-712 JSON typed-data parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eip712JsonLimits {
    /// Maximum JSON input length in bytes.
    ///
    /// JSON object width is also independently capped during duplicate-key
    /// detection so raising this limit does not admit unbounded object maps.
    pub max_input_bytes: usize,
    /// Maximum non-domain type definitions.
    pub max_types: usize,
    /// Maximum fields per type definition.
    pub max_fields_per_type: usize,
    /// Maximum elements per JSON array value.
    pub max_array_items: usize,
    /// Maximum UTF-8 string length in bytes.
    pub max_string_bytes: usize,
    /// Maximum decoded dynamic `bytes` length.
    ///
    /// Fixed `bytesN` values remain bounded by the EIP-712 `bytes1` through
    /// `bytes32` type width and are not restricted by this dynamic-byte limit.
    pub max_bytes_value: usize,
    /// Maximum JSON/type recursion depth.
    pub max_depth: usize,
}

impl Eip712JsonLimits {
    /// Conservative application starting point.
    pub const DEFAULT: Self = Self {
        max_input_bytes: 16_384,
        max_types: 64,
        max_fields_per_type: 64,
        max_array_items: 256,
        max_string_bytes: 4096,
        max_bytes_value: 4096,
        max_depth: MAX_TYPE_DEPTH,
    };
}

/// EIP-712 JSON parser failure.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Eip712JsonError {
    /// JSON syntax or duplicate-key validation failed.
    Json,
    /// JSON document does not match the EIP-712 typed-data shape.
    Shape,
    /// A configured parser limit was exceeded.
    Limit,
    /// Hex string decoding failed.
    Hex,
    /// Decimal integer decoding failed.
    Integer,
    /// The borrowed EIP-712 encoder rejected the parsed data.
    Encode(Eip712EncodeError),
}

impl fmt::Display for Eip712JsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Json => "EIP-712 JSON is malformed or has duplicate keys",
            Self::Shape => "EIP-712 JSON document has an unsupported shape",
            Self::Limit => "EIP-712 JSON parser limit was exceeded",
            Self::Hex => "EIP-712 JSON hex string is invalid",
            Self::Integer => "EIP-712 JSON integer is invalid or too large",
            Self::Encode(error) => return error.fmt(formatter),
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Eip712JsonError {}

impl From<Eip712EncodeError> for Eip712JsonError {
    fn from(error: Eip712EncodeError) -> Self {
        Self::Encode(error)
    }
}

/// Parses JSON-RPC EIP-712 typed data and computes the final signing digest.
pub fn eip712_json_typed_data_signing_digest<H>(
    input: &str,
    limits: Eip712JsonLimits,
    type_scratch: &mut [u8],
) -> Result<B256, Eip712JsonError>
where
    H: Default + Keccak256,
{
    check_len(input.len(), limits.max_input_bytes)?;
    let json: Json = serde_json::from_str(input).map_err(|_| Eip712JsonError::Json)?;
    let root = json.as_object()?;
    let owned = parse_types(root.get("types")?, limits)?;
    let field_sets = borrow_field_sets(&owned);
    let types = borrow_types(&owned, &field_sets);
    let primary_type = root.get("primaryType")?.as_str()?;
    let domain = parse_domain(root.get("domain")?, limits)?;
    let message = root.get("message")?;
    let domain_separator = eip712_domain_separator::<H>(domain, type_scratch)?;
    let message_hash =
        hash_json_struct::<H>(&types, primary_type, message, type_scratch, limits, 0)?;
    Ok(eip712_signing_digest(
        domain_separator,
        message_hash,
        H::default(),
    ))
}

fn hash_json_struct<H>(
    types: &[Eip712StructType<'_>],
    primary_type: &str,
    value: &Json,
    type_scratch: &mut [u8],
    limits: Eip712JsonLimits,
    depth: usize,
) -> Result<B256, Eip712JsonError>
where
    H: Default + Keccak256,
{
    check_depth(depth, limits)?;
    let object = value.as_object()?;
    let ty = find_struct(types, primary_type)?;
    let type_hash = eip712_type_hash::<H>(types, primary_type, type_scratch)?;
    let mut hasher = H::default();
    hasher.update(&type_hash.to_bytes());
    for field in ty.fields {
        let mut word = [0_u8; WORD_BYTES];
        encode_json_word::<H>(
            types,
            field.type_name,
            object.get(field.name)?,
            &mut word,
            type_scratch,
            limits,
            depth.saturating_add(1),
        )?;
        hasher.update(&word);
    }
    Ok(hasher.finalize())
}

fn encode_json_word<H>(
    types: &[Eip712StructType<'_>],
    type_name: &str,
    value: &Json,
    out: &mut [u8; WORD_BYTES],
    type_scratch: &mut [u8],
    limits: Eip712JsonLimits,
    depth: usize,
) -> Result<(), Eip712JsonError>
where
    H: Default + Keccak256,
{
    if let Some(array) = parse_array_type(type_name)? {
        check_depth(depth, limits)?;
        let values = value.as_array()?;
        check_len(values.len(), limits.max_array_items)?;
        if let Some(expected) = array.len
            && values.len() != expected
        {
            return Err(Eip712JsonError::Shape);
        }
        let mut hasher = H::default();
        for item in values {
            let mut word = [0_u8; WORD_BYTES];
            encode_json_word::<H>(
                types,
                array.base,
                item,
                &mut word,
                type_scratch,
                limits,
                depth.saturating_add(1),
            )?;
            hasher.update(&word);
        }
        *out = hasher.finalize().to_bytes();
        return Ok(());
    }
    match type_name {
        "bool" => {
            let last = out.last_mut().ok_or(Eip712EncodeError::OutputTooShort)?;
            *last = u8::from(value.as_bool()?);
        }
        "address" => {
            let address = parse_address(value.as_str()?)?;
            out.get_mut(ADDRESS_PADDING_BYTES..)
                .ok_or(Eip712EncodeError::OutputTooShort)?
                .copy_from_slice(&address.to_bytes());
        }
        "bytes" => {
            let bytes = scalar::parse_hex(value.as_str()?, limits.max_bytes_value)?;
            *out = hash_one(H::default(), &bytes).to_bytes();
        }
        "string" => {
            let text = value.as_str()?;
            check_len(text.len(), limits.max_string_bytes)?;
            *out = hash_one(H::default(), text.as_bytes()).to_bytes();
        }
        name if find_struct(types, name).is_ok() => {
            *out =
                hash_json_struct::<H>(types, name, value, type_scratch, limits, depth)?.to_bytes();
        }
        name => encode_json_numeric_or_bytes(name, value, out, limits)?,
    }
    Ok(())
}

struct OwnedType {
    name: String,
    fields: Vec<OwnedField>,
}

struct OwnedField {
    name: String,
    type_name: String,
}

fn parse_types(types: &Json, limits: Eip712JsonLimits) -> Result<Vec<OwnedType>, Eip712JsonError> {
    let object = types.as_object()?;
    let mut owned = Vec::new();
    for (name, fields) in object.entries() {
        if name == "EIP712Domain" {
            check_len(fields.as_array()?.len(), limits.max_fields_per_type)?;
            continue;
        }
        check_len(owned.len().saturating_add(1), limits.max_types)?;
        check_len(name.len(), limits.max_string_bytes)?;
        validate_identifier(name)?;
        let field_values = fields.as_array()?;
        check_len(field_values.len(), limits.max_fields_per_type)?;
        let mut parsed_fields = Vec::<OwnedField>::new();
        for field in field_values {
            let field = field.as_object()?;
            let field_name = field.get("name")?.as_str()?;
            let field_type = field.get("type")?.as_str()?;
            check_len(field_name.len(), limits.max_string_bytes)?;
            check_len(field_type.len(), limits.max_string_bytes)?;
            validate_identifier(field_name)?;
            if parsed_fields.iter().any(|field| field.name == field_name) {
                return Err(Eip712JsonError::Shape);
            }
            parsed_fields.push(OwnedField {
                name: field_name.to_string(),
                type_name: field_type.to_string(),
            });
        }
        owned.push(OwnedType {
            name: name.to_string(),
            fields: parsed_fields,
        });
    }
    Ok(owned)
}

fn borrow_field_sets(types: &[OwnedType]) -> Vec<Vec<Eip712Field<'_>>> {
    types
        .iter()
        .map(|ty| {
            ty.fields
                .iter()
                .map(|field| Eip712Field {
                    name: field.name.as_str(),
                    type_name: field.type_name.as_str(),
                })
                .collect()
        })
        .collect()
}

fn borrow_types<'a>(
    types: &'a [OwnedType],
    field_sets: &'a [Vec<Eip712Field<'a>>],
) -> Vec<Eip712StructType<'a>> {
    types
        .iter()
        .zip(field_sets.iter())
        .map(|(ty, fields)| Eip712StructType {
            name: ty.name.as_str(),
            fields,
        })
        .collect()
}

fn parse_domain(
    domain: &Json,
    limits: Eip712JsonLimits,
) -> Result<Eip712DomainData<'_>, Eip712JsonError> {
    let object = domain.as_object()?;
    reject_unknown_domain_fields(object)?;
    let name = optional_limited_str(object.get_optional("name"), limits)?;
    let version = optional_limited_str(object.get_optional("version"), limits)?;
    let chain_id = object
        .get_optional("chainId")
        .map(parse_chain_id)
        .transpose()?;
    let verifying_contract = object
        .get_optional("verifyingContract")
        .map(|value| parse_address(value.as_str()?))
        .transpose()?;
    let salt = object
        .get_optional("salt")
        .map(|value| parse_b256(value.as_str()?))
        .transpose()?;
    Ok(Eip712DomainData {
        name,
        version,
        chain_id,
        verifying_contract,
        salt,
    })
}

fn reject_unknown_domain_fields(object: &Object) -> Result<(), Eip712JsonError> {
    for (field, _) in object.entries() {
        if !matches!(
            field,
            "name" | "version" | "chainId" | "verifyingContract" | "salt"
        ) {
            return Err(Eip712JsonError::Shape);
        }
    }
    Ok(())
}

fn optional_limited_str(
    value: Option<&Json>,
    limits: Eip712JsonLimits,
) -> Result<Option<&str>, Eip712JsonError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let text = value.as_str()?;
    check_len(text.len(), limits.max_string_bytes)?;
    Ok(Some(text))
}

#[cfg(test)]
#[path = "eip712_json_tests.rs"]
mod tests;
