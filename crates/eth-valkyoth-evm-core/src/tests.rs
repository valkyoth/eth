use crate::{
    EVM_DEFAULT_STEP_LIMIT, EVM_MAX_STEP_LIMIT, EVM_MEMORY_LIMIT_BYTES, EvmCoreError, EvmExecution,
    EvmFork, EvmMemory, EvmOpcode, EvmStack, EvmWord, ExecutionLimits, ExecutionStatus,
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
    assert_eq!(stack.debug_raw_slot(0), Some(EvmWord::ZERO));
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

#[test]
fn word_arithmetic_wraps_at_256_bits() -> Result<(), EvmCoreError> {
    let max = EvmWord::from_be_bytes([0xffu8; EvmWord::LEN]);
    assert_eq!(
        max.wrapping_add_word(EvmWord::from_be_slice(&[1])?),
        EvmWord::ZERO
    );
    assert_eq!(
        EvmWord::ZERO.wrapping_sub_word(EvmWord::from_be_slice(&[1])?),
        max
    );
    assert_eq!(
        EvmWord::from_be_slice(&[3])?.wrapping_mul_word(EvmWord::from_be_slice(&[7])?),
        EvmWord::from_be_slice(&[21])?
    );
    Ok(())
}

#[test]
fn execution_runs_arithmetic_bitwise_and_comparison_subset() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [
        0x60, 0x02, 0x60, 0x03, 0x01, 0x60, 0x05, 0x14, 0x60, 0xf0, 0x60, 0x0f, 0x17, 0x60, 0xff,
        0x14, 0x00,
    ];
    let report = execution.run(&code, ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT)?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(execution.stack().peek(1)?, EvmWord::from_bool(true));
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_bool(true));
    Ok(())
}

#[test]
fn execution_validates_dynamic_jumpdest() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x04, 0x56, 0x00, 0x5b, 0x60, 0x07, 0x00];
    let report = execution.run(&code, ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT)?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_be_slice(&[7])?);
    Ok(())
}

#[test]
fn execution_rejects_jump_into_push_immediate() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x01, 0x56];

    assert_eq!(
        execution.run(&code, ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT)?),
        Err(EvmCoreError::InvalidJumpDestination)
    );
    Ok(())
}

#[test]
fn execution_false_jumpi_advances_to_next_instruction() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [
        0x60, 0x00, 0x60, 0x08, 0x57, 0x60, 0x09, 0x00, 0x5b, 0x60, 0x01, 0x00,
    ];
    let report = execution.run(&code, ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT)?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_be_slice(&[9])?);
    Ok(())
}

#[test]
fn execution_reports_return_and_revert_ranges() -> Result<(), EvmCoreError> {
    let mut return_memory = [0u8; 4];
    let mut return_execution = EvmExecution::<16>::try_new(&mut return_memory)?;
    let return_code = [0x60, 0x02, 0x60, 0x01, 0xf3];
    let return_report = return_execution.run(
        &return_code,
        ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT)?,
    )?;
    assert_eq!(
        return_report.status,
        ExecutionStatus::Returned { offset: 1, len: 2 }
    );

    let mut revert_memory = [0u8; 4];
    let mut revert_execution = EvmExecution::<16>::try_new(&mut revert_memory)?;
    let revert_code = [0x60, 0x02, 0x60, 0x01, 0xfd];
    let revert_report = revert_execution.run(
        &revert_code,
        ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT)?,
    )?;
    assert_eq!(
        revert_report.status,
        ExecutionStatus::Reverted { offset: 1, len: 2 }
    );
    Ok(())
}

#[test]
fn execution_fails_closed_on_truncated_push_and_step_limit() -> Result<(), EvmCoreError> {
    let mut truncated_memory = [0u8; 0];
    let mut truncated_execution = EvmExecution::<16>::try_new(&mut truncated_memory)?;
    assert_eq!(
        truncated_execution.run(
            &[0x61, 0x01],
            ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT)?
        ),
        Err(EvmCoreError::PushImmediateOutOfBounds)
    );

    let mut loop_memory = [0u8; 0];
    let mut loop_execution = EvmExecution::<16>::try_new(&mut loop_memory)?;
    let code = [0x5b, 0x60, 0x00, 0x56];
    assert_eq!(
        loop_execution.run(&code, ExecutionLimits::try_new(4)?),
        Err(EvmCoreError::ExecutionStepLimitReached)
    );
    assert_eq!(
        ExecutionLimits::try_new(0),
        Err(EvmCoreError::ExecutionStepLimitZero)
    );
    assert_eq!(
        ExecutionLimits::try_new(EVM_MAX_STEP_LIMIT + 1),
        Err(EvmCoreError::ExecutionStepLimitTooLarge)
    );
    Ok(())
}
