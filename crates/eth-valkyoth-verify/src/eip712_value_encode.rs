use eth_valkyoth_hash::{Keccak256, hash_one};

use super::eip712_schema::ValidatedSchema;
use super::typed_helpers::{encode_numeric_or_fixed_bytes, parse_array_type};
use super::{
    ADDRESS_PADDING_BYTES, EIP712_MAX_ARRAY_ITEMS, Eip712EncodeError, Eip712ValueKind,
    MAX_TYPE_DEPTH, WORD_BYTES, hash_struct_inner,
};

pub(super) fn encode_value_word<H>(
    schema: &mut ValidatedSchema<'_>,
    type_name: &str,
    value: &Eip712ValueKind<'_>,
    out: &mut [u8; WORD_BYTES],
    type_scratch: &mut [u8],
    depth: usize,
) -> Result<(), Eip712EncodeError>
where
    H: Default + Keccak256,
{
    schema.charge_value_node()?;
    if let Some(array) = parse_array_type(type_name)? {
        if depth >= MAX_TYPE_DEPTH {
            return Err(Eip712EncodeError::RecursionLimit);
        }
        let Eip712ValueKind::Array(values) = value else {
            return Err(Eip712EncodeError::TypeMismatch);
        };
        if values.len() > EIP712_MAX_ARRAY_ITEMS {
            return Err(Eip712EncodeError::ResourceLimit);
        }
        if let Some(expected) = array.len
            && values.len() != expected
        {
            return Err(Eip712EncodeError::TypeMismatch);
        }
        let mut hasher = H::default();
        for item in *values {
            let mut word = [0_u8; WORD_BYTES];
            encode_value_word::<H>(
                schema,
                array.base,
                item,
                &mut word,
                type_scratch,
                depth.saturating_add(1),
            )?;
            hasher.update(&word);
        }
        *out = hasher.finalize().to_bytes();
        return Ok(());
    }
    match (type_name, value) {
        ("bool", Eip712ValueKind::Bool(value)) => {
            let last = out.last_mut().ok_or(Eip712EncodeError::OutputTooShort)?;
            *last = u8::from(*value);
        }
        ("address", Eip712ValueKind::Address(value)) => {
            let bytes = value.to_bytes();
            out.get_mut(ADDRESS_PADDING_BYTES..)
                .ok_or(Eip712EncodeError::OutputTooShort)?
                .copy_from_slice(&bytes);
        }
        ("bytes", Eip712ValueKind::Bytes(value)) => *out = hash_one(H::default(), value).to_bytes(),
        ("string", Eip712ValueKind::String(value)) => {
            *out = hash_one(H::default(), value.as_bytes()).to_bytes();
        }
        (name, Eip712ValueKind::Struct(values)) if schema.contains_struct(name) => {
            *out = hash_struct_inner::<H>(schema, name, values, type_scratch, depth)?.to_bytes();
        }
        (name, value) => encode_numeric_or_fixed_bytes(name, value, out)?,
    }
    Ok(())
}
