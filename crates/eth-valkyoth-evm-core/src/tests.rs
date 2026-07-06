use crate::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EVM_MAX_BYTECODE_LEN, EVM_MAX_GAS_LIMIT,
    EVM_MAX_STEP_LIMIT, EVM_MEMORY_LIMIT_BYTES, EvmCoreError, EvmExecution, EvmFork, EvmGas,
    EvmGasSchedule, EvmMemory, EvmOpcode, EvmStack, EvmWord, ExecutionLimits, ExecutionStatus,
    OpcodeClass, OpcodeTable, ProgramCounter,
};

fn execution_limits() -> Result<ExecutionLimits, EvmCoreError> {
    ExecutionLimits::try_new(
        EVM_DEFAULT_STEP_LIMIT,
        EVM_DEFAULT_GAS_LIMIT,
        EvmFork::CANCUN,
    )
}

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
fn historical_fork_identifiers_are_ordered_and_bounded() {
    assert_eq!(EvmFork::FRONTIER.get(), 0);
    assert_eq!(EvmFork::HOMESTEAD.get(), 1);
    assert_eq!(EvmFork::TANGERINE_WHISTLE.get(), 2);
    assert_eq!(EvmFork::SPURIOUS_DRAGON.get(), 3);
    assert_eq!(EvmFork::BYZANTIUM.get(), 4);
    assert_eq!(EvmFork::CONSTANTINOPLE.get(), 5);
    assert_eq!(EvmFork::PETERSBURG.get(), 6);
    assert_eq!(EvmFork::ISTANBUL.get(), 7);
    assert_eq!(EvmFork::BERLIN.get(), 8);
    assert_eq!(EvmFork::LONDON.get(), 9);
    assert_eq!(EvmFork::SHANGHAI.get(), 10);
    assert_eq!(EvmFork::CANCUN.get(), 11);
    assert_eq!(EvmFork::PRAGUE.get(), 12);
    assert_eq!(EvmFork::AMSTERDAM.get(), 13);

    assert!(EvmFork::PRAGUE.is_supported());
    assert!(EvmFork::AMSTERDAM.is_known());
    assert!(!EvmFork::AMSTERDAM.is_supported());
    assert!(!EvmFork::ISTANBUL.supports_warm_cold_state_access());
    assert!(EvmFork::BERLIN.supports_warm_cold_state_access());
}

#[test]
fn opcode_introduction_boundaries_are_explicit() {
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::ADD),
        Some(EvmFork::FRONTIER)
    );
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::REVERT),
        Some(EvmFork::BYZANTIUM)
    );
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::EXTCODEHASH),
        Some(EvmFork::CONSTANTINOPLE)
    );
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::SELFBALANCE),
        Some(EvmFork::ISTANBUL)
    );
    assert_eq!(EvmFork::opcode_introduced_in(EvmOpcode::new(0xef)), None);

    assert!(!EvmFork::FRONTIER.opcode_is_introduced(EvmOpcode::REVERT));
    assert!(EvmFork::BYZANTIUM.opcode_is_introduced(EvmOpcode::REVERT));
    assert!(!EvmFork::PETERSBURG.opcode_is_introduced(EvmOpcode::SELFBALANCE));
    assert!(EvmFork::ISTANBUL.opcode_is_introduced(EvmOpcode::SELFBALANCE));
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
    assert_eq!(
        OpcodeTable::try_new(EvmFork::FRONTIER)?.instruction(EvmOpcode::REVERT),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(
        OpcodeTable::try_new(EvmFork::BYZANTIUM)?
            .instruction(EvmOpcode::REVERT)?
            .class,
        OpcodeClass::ControlFlow
    );
    assert_eq!(
        OpcodeTable::try_new(EvmFork::FRONTIER)?
            .instruction(EvmOpcode::BALANCE)?
            .class,
        OpcodeClass::State
    );
    assert_eq!(
        OpcodeTable::try_new(EvmFork::CONSTANTINOPLE)?
            .instruction(EvmOpcode::EXTCODEHASH)?
            .class,
        OpcodeClass::State
    );
    assert_eq!(
        OpcodeTable::try_new(EvmFork::PETERSBURG)?.instruction(EvmOpcode::SELFBALANCE),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(
        OpcodeTable::try_new(EvmFork::LONDON)?
            .instruction(EvmOpcode::BALANCE)?
            .class,
        OpcodeClass::State
    );
    Ok(())
}

#[test]
fn opcode_table_rejects_unsupported_forks() {
    assert_eq!(
        OpcodeTable::try_new(EvmFork::AMSTERDAM),
        Err(EvmCoreError::UnsupportedFork)
    );
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
fn gas_schedule_charges_claimed_opcode_subset() -> Result<(), EvmCoreError> {
    let schedule = EvmGasSchedule::for_fork(EvmFork::CANCUN)?;

    assert_eq!(schedule.base_cost(EvmOpcode::STOP)?, EvmGas::new(0));
    assert_eq!(schedule.base_cost(EvmOpcode::ADD)?, EvmGas::new(3));
    assert_eq!(schedule.base_cost(EvmOpcode::MUL)?, EvmGas::new(5));
    assert_eq!(schedule.base_cost(EvmOpcode::POP)?, EvmGas::new(2));
    assert_eq!(schedule.base_cost(EvmOpcode::JUMP)?, EvmGas::new(8));
    assert_eq!(schedule.base_cost(EvmOpcode::JUMPI)?, EvmGas::new(10));
    assert_eq!(schedule.base_cost(EvmOpcode::JUMPDEST)?, EvmGas::new(1));
    assert_eq!(schedule.base_cost(EvmOpcode::PUSH1)?, EvmGas::new(3));
    assert_eq!(
        schedule.base_cost(EvmOpcode::MLOAD),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(
        schedule.account_access_cost(EvmOpcode::BALANCE, false)?,
        EvmGas::new(2_600)
    );
    assert_eq!(
        schedule.account_access_cost(EvmOpcode::BALANCE, true)?,
        EvmGas::new(100)
    );
    assert_eq!(schedule.storage_access_cost(false), EvmGas::new(2_100));
    assert_eq!(schedule.storage_access_cost(true), EvmGas::new(100));
    assert_eq!(schedule.selfbalance_cost(), EvmGas::new(5));
    assert_eq!(schedule.copy_cost(33)?, EvmGas::new(6));
    Ok(())
}

#[test]
fn gas_schedule_rejects_opcode_before_introduction() -> Result<(), EvmCoreError> {
    let frontier = EvmGasSchedule::for_fork(EvmFork::FRONTIER)?;
    let byzantium = EvmGasSchedule::for_fork(EvmFork::BYZANTIUM)?;

    assert_eq!(
        frontier.base_cost(EvmOpcode::REVERT),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(byzantium.base_cost(EvmOpcode::REVERT)?, EvmGas::new(0));
    Ok(())
}

#[test]
fn gas_schedule_computes_memory_expansion_costs() -> Result<(), EvmCoreError> {
    let schedule = EvmGasSchedule::for_fork(EvmFork::CANCUN)?;

    assert_eq!(
        schedule.memory_expansion_cost(0, 0, 0)?,
        (0, EvmGas::new(0))
    );
    assert_eq!(
        schedule.memory_expansion_cost(0, 0, 32)?,
        (1, EvmGas::new(3))
    );
    assert_eq!(
        schedule.memory_expansion_cost(1, 32, 1)?,
        (2, EvmGas::new(3))
    );
    assert_eq!(
        schedule.memory_expansion_cost(0, usize::MAX, 1),
        Err(EvmCoreError::GasOverflow)
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
    let report = execution.run(&code, execution_limits()?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(report.gas_used, EvmGas::new(30));
    assert_eq!(execution.stack().peek(1)?, EvmWord::from_bool(true));
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_bool(true));
    Ok(())
}

#[test]
fn execution_validates_dynamic_jumpdest() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x04, 0x56, 0x00, 0x5b, 0x60, 0x07, 0x00];
    let report = execution.run(&code, execution_limits()?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(report.gas_used, EvmGas::new(15));
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_be_slice(&[7])?);
    Ok(())
}

#[test]
fn execution_precomputes_jumpdests_with_bounded_bytecode() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let mut code = [0u8; EVM_MAX_BYTECODE_LEN];
    let target = EVM_MAX_BYTECODE_LEN - 1;
    code[0] = 0x61;
    code[1] = 0x5f;
    code[2] = 0xff;
    code[3] = 0x56;
    if let Some(slot) = code.get_mut(target) {
        *slot = 0x5b;
    }

    let report = execution.run(&code, execution_limits()?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(report.gas_used, EvmGas::new(12));
    assert_eq!(report.steps, 3);
    assert_eq!(report.pc.get(), target + 1);
    Ok(())
}

#[test]
fn execution_rejects_oversized_bytecode() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0u8; EVM_MAX_BYTECODE_LEN + 1];

    assert_eq!(
        execution.run(&code, execution_limits()?),
        Err(EvmCoreError::BytecodeTooLarge)
    );
    Ok(())
}

#[test]
fn execution_rejects_jump_into_push_immediate() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x01, 0x56];

    assert_eq!(
        execution.run(&code, execution_limits()?),
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
    let report = execution.run(&code, execution_limits()?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(report.gas_used, EvmGas::new(19));
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_be_slice(&[9])?);
    Ok(())
}

#[test]
fn execution_reports_return_and_revert_ranges() -> Result<(), EvmCoreError> {
    let mut return_memory = [0u8; 4];
    let mut return_execution = EvmExecution::<16>::try_new(&mut return_memory)?;
    let return_code = [0x60, 0x02, 0x60, 0x01, 0xf3];
    let return_report = return_execution.run(&return_code, execution_limits()?)?;
    assert_eq!(
        return_report.status,
        ExecutionStatus::Returned { offset: 1, len: 2 }
    );
    assert_eq!(return_report.gas_used, EvmGas::new(9));

    let mut revert_memory = [0u8; 4];
    let mut revert_execution = EvmExecution::<16>::try_new(&mut revert_memory)?;
    let revert_code = [0x60, 0x02, 0x60, 0x01, 0xfd];
    let revert_report = revert_execution.run(&revert_code, execution_limits()?)?;
    assert_eq!(
        revert_report.status,
        ExecutionStatus::Reverted { offset: 1, len: 2 }
    );
    assert_eq!(revert_report.gas_used, EvmGas::new(9));
    Ok(())
}

#[test]
fn execution_rejects_opcode_before_introduction() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 4];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x02, 0x60, 0x01, 0xfd];
    let limits = ExecutionLimits::try_new(
        EVM_DEFAULT_STEP_LIMIT,
        EVM_DEFAULT_GAS_LIMIT,
        EvmFork::FRONTIER,
    )?;

    assert_eq!(
        execution.run(&code, limits),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(execution.stack().len(), 2);
    Ok(())
}

#[test]
fn execution_charges_gas_before_stack_side_effects() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x01, 0x00];
    let limits = ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT, 2, EvmFork::CANCUN)?;

    assert_eq!(execution.run(&code, limits), Err(EvmCoreError::OutOfGas));
    assert!(execution.stack().is_empty());
    Ok(())
}

#[test]
fn execution_fails_closed_on_truncated_push_and_step_limit() -> Result<(), EvmCoreError> {
    let mut truncated_memory = [0u8; 0];
    let mut truncated_execution = EvmExecution::<16>::try_new(&mut truncated_memory)?;
    assert_eq!(
        truncated_execution.run(&[0x61, 0x01], execution_limits()?),
        Err(EvmCoreError::PushImmediateOutOfBounds)
    );

    let mut loop_memory = [0u8; 0];
    let mut loop_execution = EvmExecution::<16>::try_new(&mut loop_memory)?;
    let code = [0x5b, 0x60, 0x00, 0x56];
    assert_eq!(
        loop_execution.run(
            &code,
            ExecutionLimits::try_new(4, EVM_DEFAULT_GAS_LIMIT, EvmFork::CANCUN)?
        ),
        Err(EvmCoreError::ExecutionStepLimitReached)
    );
    assert_eq!(
        ExecutionLimits::try_new(0, EVM_DEFAULT_GAS_LIMIT, EvmFork::CANCUN),
        Err(EvmCoreError::ExecutionStepLimitZero)
    );
    assert_eq!(
        ExecutionLimits::try_new(
            EVM_MAX_STEP_LIMIT + 1,
            EVM_DEFAULT_GAS_LIMIT,
            EvmFork::CANCUN
        ),
        Err(EvmCoreError::ExecutionStepLimitTooLarge)
    );
    assert_eq!(
        ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT, 0, EvmFork::CANCUN),
        Err(EvmCoreError::ExecutionGasLimitZero)
    );
    assert_eq!(
        ExecutionLimits::try_new(
            EVM_DEFAULT_STEP_LIMIT,
            EVM_MAX_GAS_LIMIT + 1,
            EvmFork::CANCUN
        ),
        Err(EvmCoreError::ExecutionGasLimitTooLarge)
    );
    Ok(())
}
