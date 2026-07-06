use crate::{
    EVM_MEMORY_LIMIT_BYTES, EvmCoreError, EvmFork, EvmMemory, EvmOpcode, EvmStack, EvmWord,
    OpcodeClass, OpcodeTable, ProgramCounter,
};

#[test]
fn stack_push_pop_tracks_depth_and_clears_slots() -> Result<(), EvmCoreError> {
    let mut stack = EvmStack::<2>::try_new()?;
    let word = EvmWord::from_be_bytes([7u8; EvmWord::LEN]);

    assert!(stack.is_empty());
    stack.push(word)?;
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.pop()?, word);
    assert!(stack.is_empty());
    assert_eq!(stack.pop(), Err(EvmCoreError::StackUnderflow));
    Ok(())
}

#[test]
fn stack_rejects_invalid_capacity_and_overflow() -> Result<(), EvmCoreError> {
    assert_eq!(
        EvmStack::<0>::try_new(),
        Err(EvmCoreError::StackCapacityZero)
    );
    assert_eq!(
        EvmStack::<1025>::try_new(),
        Err(EvmCoreError::StackCapacityTooLarge)
    );

    let mut stack = EvmStack::<1>::try_new()?;
    stack.push(EvmWord::ZERO)?;
    assert_eq!(stack.push(EvmWord::ZERO), Err(EvmCoreError::StackOverflow));
    Ok(())
}

#[test]
fn memory_view_enforces_bounds() -> Result<(), EvmCoreError> {
    let mut bytes = [0u8; 4];
    let mut memory = EvmMemory::try_new(&mut bytes)?;

    assert_eq!(memory.len(), 4);
    assert_eq!(memory.read_byte(2)?, 0);
    memory.write_byte(2, 9)?;
    assert_eq!(memory.read_byte(2)?, 9);
    assert_eq!(
        memory.read_byte(4),
        Err(EvmCoreError::MemoryOffsetOutOfBounds)
    );
    Ok(())
}

#[test]
fn memory_length_validation_does_not_require_allocation() {
    assert_eq!(EvmMemory::validate_len(EVM_MEMORY_LIMIT_BYTES), Ok(()));
    assert_eq!(
        EvmMemory::validate_len(16_777_217),
        Err(EvmCoreError::MemoryTooLarge)
    );
}

#[test]
fn program_counter_advances_with_overflow_guard() -> Result<(), EvmCoreError> {
    let counter = ProgramCounter::new(2).advance(3)?;
    assert_eq!(counter.get(), 5);
    assert_eq!(
        ProgramCounter::new(usize::MAX).advance(1),
        Err(EvmCoreError::ProgramCounterOverflow)
    );
    Ok(())
}

#[test]
fn opcode_domain_exposes_push_widths() {
    assert_eq!(EvmOpcode::PUSH1.push_width(), Some(1));
    assert_eq!(EvmOpcode::PUSH32.push_width(), Some(32));
    assert_eq!(EvmOpcode::ADD.push_width(), None);
}

#[test]
fn opcode_table_supports_known_skeleton_and_rejects_unknowns() -> Result<(), EvmCoreError> {
    let table = OpcodeTable::try_new(EvmFork::CANCUN)?;
    let info = table.instruction(EvmOpcode::ADD)?;
    assert_eq!(info.class, OpcodeClass::Arithmetic);

    assert_eq!(
        table.instruction(EvmOpcode::new(0xef)),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    Ok(())
}

#[test]
fn opcode_table_rejects_unsupported_forks() {
    assert_eq!(
        OpcodeTable::try_new(EvmFork::new(999)),
        Err(EvmCoreError::UnsupportedFork)
    );
}
