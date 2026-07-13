extern crate std;

use std::vec::Vec;

use crate::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EvmCoreError, EvmExecution, EvmFork, EvmGas,
    EvmWord, ExecutionLimits, ExecutionStatus,
};

fn limits() -> Result<ExecutionLimits, EvmCoreError> {
    ExecutionLimits::try_new(
        EVM_DEFAULT_STEP_LIMIT,
        EVM_DEFAULT_GAS_LIMIT,
        EvmFork::CANCUN,
    )
}

fn push_max_word(code: &mut Vec<u8>) {
    code.push(0x7f);
    for _ in 0..EvmWord::LEN {
        code.push(u8::MAX);
    }
}

#[test]
fn false_jumpi_ignores_out_of_range_destination() -> Result<(), EvmCoreError> {
    let mut code = Vec::from([0x60, 0x00]);
    push_max_word(&mut code);
    code.extend_from_slice(&[0x57, 0x60, 0x2a, 0x00]);
    let mut memory = [];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;

    let report = execution.run(&code, limits()?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_usize(42));
    Ok(())
}

#[test]
fn false_jumpi_ignores_non_jumpdest_destination() -> Result<(), EvmCoreError> {
    let code = [0x60, 0x00, 0x60, 0x01, 0x57, 0x60, 0x2a, 0x00];
    let mut memory = [];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;

    let report = execution.run(&code, limits()?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_usize(42));
    Ok(())
}

#[test]
fn true_jumpi_still_validates_out_of_range_destination() -> Result<(), EvmCoreError> {
    let mut code = Vec::from([0x60, 0x01]);
    push_max_word(&mut code);
    code.push(0x57);
    let mut memory = [];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;

    assert_eq!(
        execution.run(&code, limits()?),
        Err(EvmCoreError::WordInputTooLarge)
    );
    Ok(())
}

#[test]
fn zero_length_return_and_revert_ignore_out_of_range_offset() -> Result<(), EvmCoreError> {
    for (opcode, expected) in [
        (0xf3, ExecutionStatus::Returned { offset: 0, len: 0 }),
        (0xfd, ExecutionStatus::Reverted { offset: 0, len: 0 }),
    ] {
        let mut code = Vec::from([0x60, 0x00]);
        push_max_word(&mut code);
        code.push(opcode);
        let mut memory = [];
        let mut execution = EvmExecution::<4>::try_new(&mut memory)?;

        let report = execution.run(&code, limits()?)?;

        assert_eq!(report.status, expected);
        assert_eq!(report.gas_used, EvmGas::new(6));
    }
    Ok(())
}

#[test]
fn zero_length_return_ignores_ordinary_offset_beyond_memory() -> Result<(), EvmCoreError> {
    let code = [0x60, 0x00, 0x60, 0x05, 0xf3];
    let mut memory = [0_u8; 4];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;

    let report = execution.run(&code, limits()?)?;

    assert_eq!(
        report.status,
        ExecutionStatus::Returned { offset: 0, len: 0 }
    );
    assert_eq!(report.gas_used, EvmGas::new(6));
    Ok(())
}

#[test]
fn nonempty_return_still_validates_out_of_range_offset() -> Result<(), EvmCoreError> {
    let mut code = Vec::from([0x60, 0x01]);
    push_max_word(&mut code);
    code.push(0xf3);
    let mut memory = [0_u8; 4];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;

    assert_eq!(
        execution.run(&code, limits()?),
        Err(EvmCoreError::WordInputTooLarge)
    );
    Ok(())
}

#[test]
fn precompile_plan_mismatch_error_has_stable_code() {
    assert_eq!(
        EvmCoreError::PrecompilePlanInputMismatch.code(),
        "precompile_plan_input_mismatch"
    );
}
