use crate::{
    EVM_CALL_DEPTH_LIMIT, EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EvmAddress,
    EvmCallFramePolicy, EvmCallKind, EvmCoreError, EvmCreateKind, EvmExecution, EvmFork,
    EvmJournal, EvmOpcode, EvmReturnDataRange, EvmWord, ExecutionLimits, OpcodeClass, OpcodeTable,
    call::EvmCallCreatePlan,
};

fn limits(fork: EvmFork) -> Result<ExecutionLimits, EvmCoreError> {
    ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT, EVM_DEFAULT_GAS_LIMIT, fork)
}

fn push_words<const STACK: usize>(
    execution: &mut EvmExecution<'_, STACK>,
    words: &[EvmWord],
) -> Result<(), EvmCoreError> {
    for word in words {
        execution.stack_mut().push(*word)?;
    }
    Ok(())
}

fn address(value: u8) -> EvmAddress {
    let mut bytes = [0u8; EvmAddress::LEN];
    bytes[EvmAddress::LEN - 1] = value;
    EvmAddress::from_bytes(bytes)
}

#[test]
fn call_create_opcode_boundaries_are_fork_aware() -> Result<(), EvmCoreError> {
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::CALL),
        Some(EvmFork::FRONTIER)
    );
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::DELEGATECALL),
        Some(EvmFork::HOMESTEAD)
    );
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::STATICCALL),
        Some(EvmFork::BYZANTIUM)
    );
    assert_eq!(
        EvmFork::opcode_introduced_in(EvmOpcode::CREATE2),
        Some(EvmFork::CONSTANTINOPLE)
    );

    assert_eq!(
        OpcodeTable::try_new(EvmFork::FRONTIER)?
            .instruction(EvmOpcode::CALL)?
            .class,
        OpcodeClass::CallCreate
    );
    assert_eq!(
        OpcodeTable::try_new(EvmFork::FRONTIER)?.instruction(EvmOpcode::DELEGATECALL),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(
        OpcodeTable::try_new(EvmFork::BYZANTIUM)?
            .instruction(EvmOpcode::STATICCALL)?
            .class,
        OpcodeClass::CallCreate
    );
    Ok(())
}

#[test]
fn call_frame_policy_enforces_static_and_depth_rules() -> Result<(), EvmCoreError> {
    let root = EvmCallFramePolicy::root();
    let static_child = root.enter_call(EvmCallKind::StaticCall, EvmWord::ZERO)?;

    assert_eq!(static_child.depth(), 1);
    assert!(static_child.is_static());
    assert_eq!(
        static_child.enter_call(EvmCallKind::Call, EvmWord::from_usize(1)),
        Err(EvmCoreError::StaticStateChange)
    );
    assert!(
        static_child
            .enter_call(EvmCallKind::CallCode, EvmWord::from_usize(1))
            .is_ok()
    );
    assert_eq!(
        static_child.enter_create(EvmCreateKind::Create),
        Err(EvmCoreError::StaticStateChange)
    );

    let deepest = EvmCallFramePolicy::try_new(EVM_CALL_DEPTH_LIMIT - 1, false)?;
    assert_eq!(
        deepest.enter_call(EvmCallKind::Call, EvmWord::ZERO),
        Err(EvmCoreError::CallDepthLimitReached)
    );
    Ok(())
}

#[test]
fn call_plan_reads_stack_operands_in_yellow_paper_order() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 16];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let target = address(0x77);

    push_words(
        &mut execution,
        &[
            EvmWord::from_usize(6),
            EvmWord::from_usize(5),
            EvmWord::from_usize(4),
            EvmWord::from_usize(3),
            EvmWord::from_usize(2),
            target.to_word(),
            EvmWord::from_usize(1),
        ],
    )?;

    let plan = execution.plan_call_create(EvmOpcode::CALL, EvmCallFramePolicy::root())?;
    let EvmCallCreatePlan::Call(plan) = plan else {
        return Err(EvmCoreError::UnsupportedOpcode);
    };

    assert_eq!(plan.kind, EvmCallKind::Call);
    assert_eq!(plan.gas, EvmWord::from_usize(1));
    assert_eq!(plan.target, target);
    assert_eq!(plan.value, EvmWord::from_usize(2));
    assert_eq!(plan.input.offset, 3);
    assert_eq!(plan.input.len, 4);
    assert_eq!(plan.output.offset, 5);
    assert_eq!(plan.output.len, 6);
    assert_eq!(plan.child_frame.depth(), 1);
    Ok(())
}

#[test]
fn value_less_call_plan_reads_shifted_operands_in_yellow_paper_order() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 16];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let target = address(0x99);

    push_words(
        &mut execution,
        &[
            EvmWord::from_usize(6),
            EvmWord::from_usize(5),
            EvmWord::from_usize(4),
            EvmWord::from_usize(3),
            target.to_word(),
            EvmWord::from_usize(1),
        ],
    )?;

    let plan = execution.plan_call_create(EvmOpcode::STATICCALL, EvmCallFramePolicy::root())?;
    let EvmCallCreatePlan::Call(plan) = plan else {
        return Err(EvmCoreError::UnsupportedOpcode);
    };

    assert_eq!(plan.kind, EvmCallKind::StaticCall);
    assert_eq!(plan.gas, EvmWord::from_usize(1));
    assert_eq!(plan.target, target);
    assert_eq!(plan.value, EvmWord::ZERO);
    assert_eq!(plan.input.offset, 3);
    assert_eq!(plan.input.len, 4);
    assert_eq!(plan.output.offset, 5);
    assert_eq!(plan.output.len, 6);
    assert!(plan.child_frame.is_static());
    Ok(())
}

#[test]
fn create_plan_reads_stack_operands_in_yellow_paper_order() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 16];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;

    push_words(
        &mut execution,
        &[
            EvmWord::from_usize(4),
            EvmWord::from_usize(3),
            EvmWord::from_usize(2),
        ],
    )?;

    let plan = execution.plan_call_create(EvmOpcode::CREATE, EvmCallFramePolicy::root())?;
    let EvmCallCreatePlan::Create(plan) = plan else {
        return Err(EvmCoreError::UnsupportedOpcode);
    };

    assert_eq!(plan.kind, EvmCreateKind::Create);
    assert_eq!(plan.value, EvmWord::from_usize(2));
    assert_eq!(plan.init_code.offset, 3);
    assert_eq!(plan.init_code.len, 4);
    assert_eq!(plan.salt, EvmWord::ZERO);

    let mut memory = [0u8; 16];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    push_words(
        &mut execution,
        &[
            EvmWord::from_usize(7),
            EvmWord::from_usize(4),
            EvmWord::from_usize(3),
            EvmWord::from_usize(2),
        ],
    )?;

    let plan = execution.plan_call_create(EvmOpcode::CREATE2, EvmCallFramePolicy::root())?;
    let EvmCallCreatePlan::Create(plan) = plan else {
        return Err(EvmCoreError::UnsupportedOpcode);
    };

    assert_eq!(plan.kind, EvmCreateKind::Create2);
    assert_eq!(plan.value, EvmWord::from_usize(2));
    assert_eq!(plan.init_code.offset, 3);
    assert_eq!(plan.init_code.len, 4);
    assert_eq!(plan.salt, EvmWord::from_usize(7));
    Ok(())
}

#[test]
fn static_call_value_check_uses_call_value_operand() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 16];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let static_frame = EvmCallFramePolicy::try_new(0, true)?;

    push_words(
        &mut execution,
        &[
            EvmWord::ZERO,
            EvmWord::ZERO,
            EvmWord::ZERO,
            EvmWord::ZERO,
            EvmWord::from_usize(1),
            address(0x55).to_word(),
            EvmWord::from_usize(2),
        ],
    )?;

    assert_eq!(
        execution.plan_call_create(EvmOpcode::CALL, static_frame),
        Err(EvmCoreError::StaticStateChange)
    );
    Ok(())
}

#[test]
fn return_data_range_checks_bounded_copies() -> Result<(), EvmCoreError> {
    let returndata = EvmReturnDataRange::try_new(4, 8)?;

    assert_eq!(returndata.range().offset, 4);
    assert_eq!(returndata.range().len, 8);
    assert!(returndata.check_copy(2, 6).is_ok());
    assert_eq!(
        returndata.check_copy(3, 6),
        Err(EvmCoreError::ReturnDataOutOfBounds)
    );
    assert_eq!(
        EvmReturnDataRange::try_new(usize::MAX, 1),
        Err(EvmCoreError::ReturnDataOutOfBounds)
    );
    Ok(())
}

#[test]
fn journal_checkpoint_policy_is_lifo_and_bounded() -> Result<(), EvmCoreError> {
    assert_eq!(
        EvmJournal::<0>::try_new(),
        Err(EvmCoreError::JournalCapacityZero)
    );

    let mut journal = EvmJournal::<2>::try_new()?;
    let first = journal.begin()?;
    let second = journal.begin()?;

    assert_eq!(journal.depth(), 2);
    assert_eq!(
        journal.begin(),
        Err(EvmCoreError::JournalCheckpointOverflow)
    );
    assert_eq!(
        journal.commit(first),
        Err(EvmCoreError::JournalCheckpointMismatch)
    );
    journal.revert(second)?;
    journal.commit(first)?;
    assert_eq!(journal.depth(), 0);
    assert_eq!(
        journal.revert(first),
        Err(EvmCoreError::JournalCheckpointMissing)
    );
    Ok(())
}

#[test]
fn call_opcode_plans_then_fails_closed_without_stack_changes() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [
        0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0xf1,
    ];

    assert_eq!(
        execution.run(&code, limits(EvmFork::FRONTIER)?),
        Err(EvmCoreError::CallCreateExecutionUnsupported)
    );
    assert_eq!(execution.stack().len(), 7);
    Ok(())
}

#[test]
fn create_opcode_plans_then_fails_closed_without_stack_changes() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0xf0];

    assert_eq!(
        execution.run(&code, limits(EvmFork::FRONTIER)?),
        Err(EvmCoreError::CallCreateExecutionUnsupported)
    );
    assert_eq!(execution.stack().len(), 3);
    Ok(())
}

#[test]
fn call_create_opcodes_reject_before_introduction() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let staticcall = [
        0x60, 0x00, 0x60, 0x01, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0xfa,
    ];

    assert_eq!(
        execution.run(&staticcall, limits(EvmFork::HOMESTEAD)?),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(execution.stack().len(), 6);

    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let create2 = [0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0xf5];

    assert_eq!(
        execution.run(&create2, limits(EvmFork::BYZANTIUM)?),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(execution.stack().len(), 4);
    Ok(())
}

#[test]
fn call_create_zero_length_ranges_ignore_offsets_before_fail_closed() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 4];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [
        0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x05, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0xf1,
    ];

    assert_eq!(
        execution.run(&code, limits(EvmFork::FRONTIER)?),
        Err(EvmCoreError::CallCreateExecutionUnsupported)
    );
    assert_eq!(execution.stack().len(), 7);
    assert_eq!(execution.stack().peek(0)?, EvmWord::ZERO);
    assert_eq!(execution.stack().peek(1)?, EvmWord::ZERO);
    assert_eq!(execution.stack().peek(2)?, EvmWord::ZERO);
    assert_eq!(execution.stack().peek(3)?, EvmWord::from_usize(5));
    assert_eq!(execution.stack().peek(4)?, EvmWord::ZERO);
    assert_eq!(execution.stack().peek(5)?, EvmWord::ZERO);
    assert_eq!(execution.stack().peek(6)?, EvmWord::ZERO);
    Ok(())
}

#[test]
fn call_create_nonempty_ranges_are_checked_before_fail_closed() -> Result<(), EvmCoreError> {
    let mut memory = [0_u8; 4];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [
        0x60, 0x00, 0x60, 0x00, 0x60, 0x01, 0x60, 0x05, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0xf1,
    ];

    assert_eq!(
        execution.run(&code, limits(EvmFork::FRONTIER)?),
        Err(EvmCoreError::MemoryOffsetOutOfBounds)
    );
    assert_eq!(execution.stack().len(), 7);
    Ok(())
}
