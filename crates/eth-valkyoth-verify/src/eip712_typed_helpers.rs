use eth_valkyoth_hash::{Keccak256, hash_one};

use super::{
    ADDRESS_PADDING_BYTES, Eip712DomainData, Eip712EncodeError, Eip712StructType, Eip712Value,
    Eip712ValueKind, MAX_TYPE_DEPTH, WORD_BYTES, hash_struct_inner,
};

pub(super) fn encode_value_word<H>(
    types: &[Eip712StructType<'_>],
    type_name: &str,
    value: Eip712ValueKind<'_>,
    out: &mut [u8; WORD_BYTES],
    type_scratch: &mut [u8],
    depth: usize,
) -> Result<(), Eip712EncodeError>
where
    H: Default + Keccak256,
{
    if let Some(array) = parse_array_type(type_name)? {
        let Eip712ValueKind::Array(values) = value else {
            return Err(Eip712EncodeError::TypeMismatch);
        };
        if let Some(expected) = array.len
            && values.len() != expected
        {
            return Err(Eip712EncodeError::TypeMismatch);
        }
        let mut hasher = H::default();
        for item in values {
            let mut word = [0_u8; WORD_BYTES];
            encode_value_word::<H>(types, array.base, *item, &mut word, type_scratch, depth)?;
            hasher.update(&word);
        }
        *out = hasher.finalize().to_bytes();
        return Ok(());
    }
    match (type_name, value) {
        ("bool", Eip712ValueKind::Bool(value)) => {
            let last = out.last_mut().ok_or(Eip712EncodeError::OutputTooShort)?;
            *last = u8::from(value);
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
        (name, Eip712ValueKind::Struct(values)) if find_struct(types, name).is_ok() => {
            *out = hash_struct_inner::<H>(types, name, values, type_scratch, depth)?.to_bytes();
        }
        (name, value) => encode_numeric_or_fixed_bytes(name, value, out)?,
    }
    Ok(())
}

pub(super) fn update_domain_fields<H>(domain: Eip712DomainData<'_>, hasher: &mut H)
where
    H: Default + Keccak256,
{
    if let Some(name) = domain.name {
        hasher.update(&hash_one(H::default(), name.as_bytes()).to_bytes());
    }
    if let Some(version) = domain.version {
        hasher.update(&hash_one(H::default(), version.as_bytes()).to_bytes());
    }
    if let Some(chain_id) = domain.chain_id {
        let mut word = [0_u8; WORD_BYTES];
        let _ = encode_uint(256, Eip712ValueKind::Uint64(chain_id.get()), &mut word);
        hasher.update(&word);
    }
    if let Some(contract) = domain.verifying_contract {
        let mut word = [0_u8; WORD_BYTES];
        let _ = encode_value_word::<H>(
            &[],
            "address",
            Eip712ValueKind::Address(contract),
            &mut word,
            &mut [],
            0,
        );
        hasher.update(&word);
    }
    if let Some(salt) = domain.salt {
        hasher.update(&salt.to_bytes());
    }
}

pub(super) fn encode_domain_type(
    domain: Eip712DomainData<'_>,
    output: &mut [u8],
) -> Result<usize, Eip712EncodeError> {
    let mut writer = SliceWriter::new(output);
    writer.write_str("EIP712Domain(")?;
    let mut needs_comma = false;
    write_domain_field(
        domain.name.is_some(),
        "string name",
        &mut needs_comma,
        &mut writer,
    )?;
    write_domain_field(
        domain.version.is_some(),
        "string version",
        &mut needs_comma,
        &mut writer,
    )?;
    write_domain_field(
        domain.chain_id.is_some(),
        "uint256 chainId",
        &mut needs_comma,
        &mut writer,
    )?;
    write_domain_field(
        domain.verifying_contract.is_some(),
        "address verifyingContract",
        &mut needs_comma,
        &mut writer,
    )?;
    write_domain_field(
        domain.salt.is_some(),
        "bytes32 salt",
        &mut needs_comma,
        &mut writer,
    )?;
    writer.write_str(")")?;
    Ok(writer.len())
}

pub(super) fn write_struct_type(
    ty: Eip712StructType<'_>,
    writer: &mut SliceWriter<'_>,
) -> Result<(), Eip712EncodeError> {
    writer.write_str(ty.name)?;
    writer.write_str("(")?;
    let mut first = true;
    for field in ty.fields {
        if !first {
            writer.write_str(",")?;
        }
        writer.write_str(field.type_name)?;
        writer.write_str(" ")?;
        writer.write_str(field.name)?;
        first = false;
    }
    writer.write_str(")")
}

pub(super) fn next_dependency<'a>(
    types: &'a [Eip712StructType<'a>],
    primary_type: &str,
    previous: Option<&str>,
) -> Result<Option<Eip712StructType<'a>>, Eip712EncodeError> {
    let mut best = None::<Eip712StructType<'a>>;
    for ty in types {
        if ty.name == primary_type {
            continue;
        }
        if let Some(previous) = previous
            && ty.name <= previous
        {
            continue;
        }
        if !struct_references_type(types, primary_type, ty.name, 0)? {
            continue;
        }
        if best.is_none_or(|candidate| ty.name < candidate.name) {
            best = Some(*ty);
        }
    }
    Ok(best)
}

pub(super) fn find_struct<'a>(
    types: &'a [Eip712StructType<'a>],
    name: &str,
) -> Result<Eip712StructType<'a>, Eip712EncodeError> {
    types
        .iter()
        .find(|ty| ty.name == name)
        .copied()
        .ok_or(Eip712EncodeError::UnknownStruct)
}

pub(super) fn find_value<'a>(
    values: &'a [Eip712Value<'a>],
    name: &str,
) -> Result<Eip712ValueKind<'a>, Eip712EncodeError> {
    values
        .iter()
        .find(|value| value.name == name)
        .map(|value| value.value)
        .ok_or(Eip712EncodeError::MissingValue)
}

#[derive(Clone, Copy)]
struct ArrayType<'a> {
    base: &'a str,
    len: Option<usize>,
}

pub(super) struct SliceWriter<'a> {
    output: &'a mut [u8],
    len: usize,
}

impl<'a> SliceWriter<'a> {
    pub(super) fn new(output: &'a mut [u8]) -> Self {
        Self { output, len: 0 }
    }

    pub(super) const fn len(&self) -> usize {
        self.len
    }

    pub(super) fn write_str(&mut self, input: &str) -> Result<(), Eip712EncodeError> {
        let end = self
            .len
            .checked_add(input.len())
            .ok_or(Eip712EncodeError::OutputTooShort)?;
        let target = self
            .output
            .get_mut(self.len..end)
            .ok_or(Eip712EncodeError::OutputTooShort)?;
        target.copy_from_slice(input.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn write_domain_field(
    present: bool,
    field: &str,
    needs_comma: &mut bool,
    writer: &mut SliceWriter<'_>,
) -> Result<(), Eip712EncodeError> {
    if !present {
        return Ok(());
    }
    if *needs_comma {
        writer.write_str(",")?;
    }
    writer.write_str(field)?;
    *needs_comma = true;
    Ok(())
}

fn struct_references_type(
    types: &[Eip712StructType<'_>],
    from: &str,
    target: &str,
    depth: usize,
) -> Result<bool, Eip712EncodeError> {
    if depth > MAX_TYPE_DEPTH {
        return Err(Eip712EncodeError::RecursionLimit);
    }
    let ty = find_struct(types, from)?;
    for field in ty.fields {
        let base = base_type(field.type_name)?;
        if base == target {
            return Ok(true);
        }
        if find_struct(types, base).is_ok()
            && base != from
            && struct_references_type(types, base, target, depth.saturating_add(1))?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn encode_numeric_or_fixed_bytes(
    type_name: &str,
    value: Eip712ValueKind<'_>,
    out: &mut [u8; WORD_BYTES],
) -> Result<(), Eip712EncodeError> {
    if let Some(width) = unsigned_width(type_name)? {
        return encode_uint(width, value, out);
    }
    if let Some(width) = signed_width(type_name)? {
        return encode_int(width, value, out);
    }
    if let Some(width) = fixed_bytes_width(type_name)? {
        let Eip712ValueKind::FixedBytes(bytes) = value else {
            return Err(Eip712EncodeError::TypeMismatch);
        };
        if bytes.len() != width {
            return Err(Eip712EncodeError::TypeMismatch);
        }
        out.get_mut(..width)
            .ok_or(Eip712EncodeError::OutputTooShort)?
            .copy_from_slice(bytes);
        return Ok(());
    }
    Err(Eip712EncodeError::InvalidType)
}

fn encode_uint(
    width: usize,
    value: Eip712ValueKind<'_>,
    out: &mut [u8; WORD_BYTES],
) -> Result<(), Eip712EncodeError> {
    match value {
        Eip712ValueKind::Uint64(value) => {
            if width < 64 && value >= (1_u64 << width) {
                return Err(Eip712EncodeError::TypeMismatch);
            }
            out.get_mut(24..)
                .ok_or(Eip712EncodeError::OutputTooShort)?
                .copy_from_slice(&value.to_be_bytes());
        }
        Eip712ValueKind::Uint256(bytes) => {
            reject_high_bytes(width, bytes)?;
            *out = bytes;
        }
        _ => return Err(Eip712EncodeError::TypeMismatch),
    }
    Ok(())
}

fn encode_int(
    width: usize,
    value: Eip712ValueKind<'_>,
    out: &mut [u8; WORD_BYTES],
) -> Result<(), Eip712EncodeError> {
    let Eip712ValueKind::Int256(bytes) = value else {
        return Err(Eip712EncodeError::TypeMismatch);
    };
    reject_non_sign_extended(width, bytes)?;
    *out = bytes;
    Ok(())
}

fn reject_high_bytes(width: usize, bytes: [u8; 32]) -> Result<(), Eip712EncodeError> {
    let used = width / 8;
    let high = WORD_BYTES
        .checked_sub(used)
        .ok_or(Eip712EncodeError::InvalidType)?;
    if bytes
        .get(..high)
        .ok_or(Eip712EncodeError::InvalidType)?
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(Eip712EncodeError::TypeMismatch);
    }
    Ok(())
}

fn reject_non_sign_extended(width: usize, bytes: [u8; 32]) -> Result<(), Eip712EncodeError> {
    let used = width / 8;
    let high = WORD_BYTES
        .checked_sub(used)
        .ok_or(Eip712EncodeError::InvalidType)?;
    let sign_byte = bytes
        .get(high)
        .copied()
        .ok_or(Eip712EncodeError::InvalidType)?;
    let expected = if sign_byte & 0x80 == 0 { 0x00 } else { 0xff };
    if bytes
        .get(..high)
        .ok_or(Eip712EncodeError::InvalidType)?
        .iter()
        .any(|byte| *byte != expected)
    {
        return Err(Eip712EncodeError::TypeMismatch);
    }
    Ok(())
}

fn parse_array_type(type_name: &str) -> Result<Option<ArrayType<'_>>, Eip712EncodeError> {
    let Some((base, suffix)) = type_name.rsplit_once('[') else {
        return Ok(None);
    };
    let len_text = suffix
        .strip_suffix(']')
        .ok_or(Eip712EncodeError::InvalidType)?;
    let len = if len_text.is_empty() {
        None
    } else {
        Some(parse_usize(len_text)?)
    };
    if base.is_empty() {
        return Err(Eip712EncodeError::InvalidType);
    }
    Ok(Some(ArrayType { base, len }))
}

fn base_type(type_name: &str) -> Result<&str, Eip712EncodeError> {
    let mut base = type_name;
    while let Some(array) = parse_array_type(base)? {
        base = array.base;
    }
    Ok(base)
}

fn unsigned_width(type_name: &str) -> Result<Option<usize>, Eip712EncodeError> {
    parse_numeric_width(type_name, "uint")
}

fn signed_width(type_name: &str) -> Result<Option<usize>, Eip712EncodeError> {
    parse_numeric_width(type_name, "int")
}

fn fixed_bytes_width(type_name: &str) -> Result<Option<usize>, Eip712EncodeError> {
    let Some(width) = parse_prefixed_width(type_name, "bytes")? else {
        return Ok(None);
    };
    if (1..=32).contains(&width) {
        return Ok(Some(width));
    }
    Err(Eip712EncodeError::InvalidType)
}

fn parse_numeric_width(type_name: &str, prefix: &str) -> Result<Option<usize>, Eip712EncodeError> {
    let Some(width) = parse_prefixed_width(type_name, prefix)? else {
        return Ok(None);
    };
    if (8..=256).contains(&width) && width % 8 == 0 {
        return Ok(Some(width));
    }
    Err(Eip712EncodeError::InvalidType)
}

fn parse_prefixed_width(type_name: &str, prefix: &str) -> Result<Option<usize>, Eip712EncodeError> {
    let Some(suffix) = type_name.strip_prefix(prefix) else {
        return Ok(None);
    };
    if suffix.is_empty() {
        return Err(Eip712EncodeError::InvalidType);
    }
    parse_usize(suffix).map(Some)
}

fn parse_usize(input: &str) -> Result<usize, Eip712EncodeError> {
    let mut value = 0usize;
    for byte in input.bytes() {
        if !byte.is_ascii_digit() {
            return Err(Eip712EncodeError::InvalidType);
        }
        let digit = usize::from(
            byte.checked_sub(b'0')
                .ok_or(Eip712EncodeError::InvalidType)?,
        );
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(digit))
            .ok_or(Eip712EncodeError::InvalidType)?;
    }
    Ok(value)
}
