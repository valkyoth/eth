use crate::{EvmCoreError, EvmOpcode, EvmWord};

pub(crate) fn next_instruction_pc(pc: usize, opcode: EvmOpcode) -> Result<usize, EvmCoreError> {
    let immediate_width = usize::from(opcode.push_width().unwrap_or(0));
    pc.checked_add(1)
        .and_then(|next| next.checked_add(immediate_width))
        .ok_or(EvmCoreError::ProgramCounterOverflow)
}

pub(crate) fn push_immediate_word(
    bytecode: &[u8],
    pc: usize,
    opcode: EvmOpcode,
) -> Result<EvmWord, EvmCoreError> {
    let width = usize::from(opcode.push_width().ok_or(EvmCoreError::UnsupportedOpcode)?);
    let start = pc
        .checked_add(1)
        .ok_or(EvmCoreError::ProgramCounterOverflow)?;
    let target_start = EvmWord::LEN
        .checked_sub(width)
        .ok_or(EvmCoreError::ProgramCounterOverflow)?;
    let mut bytes = [0u8; EvmWord::LEN];

    for offset in 0..width {
        let source_index = start
            .checked_add(offset)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        let Some(source) = bytecode.get(source_index) else {
            break;
        };
        let target_index = target_start
            .checked_add(offset)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        let target = bytes
            .get_mut(target_index)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        *target = *source;
    }

    Ok(EvmWord::from_be_bytes(bytes))
}
