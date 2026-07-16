use super::typed_helpers::{base_type, reject_reserved_struct_name, validate_identifier};
use super::{EIP712_MAX_TYPES, Eip712EncodeError, Eip712StructType};

pub(super) fn validate_schema(types: &[Eip712StructType<'_>]) -> Result<(), Eip712EncodeError> {
    if types.len() > EIP712_MAX_TYPES {
        return Err(Eip712EncodeError::SchemaTooLarge);
    }
    for (type_index, ty) in types.iter().enumerate() {
        reject_reserved_struct_name(ty.name)?;
        validate_identifier(ty.name)?;
        if types
            .iter()
            .take(type_index)
            .any(|previous| previous.name == ty.name)
        {
            return Err(Eip712EncodeError::DuplicateType);
        }
        for (field_index, field) in ty.fields.iter().enumerate() {
            validate_identifier(field.name)?;
            validate_identifier(base_type(field.type_name)?)?;
            if ty
                .fields
                .iter()
                .take(field_index)
                .any(|previous| previous.name == field.name)
            {
                return Err(Eip712EncodeError::DuplicateField);
            }
        }
    }
    Ok(())
}

pub(super) fn collect_reachable_types(
    types: &[Eip712StructType<'_>],
    primary_type: &str,
) -> Result<[bool; EIP712_MAX_TYPES], Eip712EncodeError> {
    if types.len() > EIP712_MAX_TYPES {
        return Err(Eip712EncodeError::SchemaTooLarge);
    }
    for ty in types {
        reject_reserved_struct_name(ty.name)?;
    }
    let root = types
        .iter()
        .position(|ty| ty.name == primary_type)
        .ok_or(Eip712EncodeError::UnknownStruct)?;
    let mut seen = [false; EIP712_MAX_TYPES];
    let mut pending = [0_usize; EIP712_MAX_TYPES];
    let mut pending_len = 1_usize;
    if let Some(slot) = pending.first_mut() {
        *slot = root;
    }
    if let Some(visited) = seen.get_mut(root) {
        *visited = true;
    }
    while pending_len != 0 {
        pending_len = pending_len.saturating_sub(1);
        let index = pending
            .get(pending_len)
            .copied()
            .ok_or(Eip712EncodeError::SchemaTooLarge)?;
        let ty = types.get(index).ok_or(Eip712EncodeError::UnknownStruct)?;
        for field in ty.fields {
            let base = base_type(field.type_name)?;
            let Some(child) = types.iter().position(|candidate| candidate.name == base) else {
                continue;
            };
            let visited = seen
                .get_mut(child)
                .ok_or(Eip712EncodeError::SchemaTooLarge)?;
            if *visited {
                continue;
            }
            *visited = true;
            let slot = pending
                .get_mut(pending_len)
                .ok_or(Eip712EncodeError::SchemaTooLarge)?;
            *slot = child;
            pending_len = pending_len
                .checked_add(1)
                .ok_or(Eip712EncodeError::SchemaTooLarge)?;
        }
    }
    Ok(seen)
}

pub(super) fn next_dependency<'a>(
    types: &'a [Eip712StructType<'a>],
    primary_type: &str,
    previous: Option<&str>,
    reachable: &[bool; EIP712_MAX_TYPES],
) -> Result<Option<Eip712StructType<'a>>, Eip712EncodeError> {
    let mut best = None::<Eip712StructType<'a>>;
    for (index, ty) in types.iter().enumerate() {
        reject_reserved_struct_name(ty.name)?;
        if ty.name == primary_type || !reachable.get(index).copied().unwrap_or(false) {
            continue;
        }
        if let Some(previous) = previous
            && ty.name <= previous
        {
            continue;
        }
        if best.is_none_or(|candidate| ty.name < candidate.name) {
            best = Some(*ty);
        }
    }
    Ok(best)
}
