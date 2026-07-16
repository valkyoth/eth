use crate::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EvmAccessSet, EvmAccount, EvmAddress,
    EvmCoreError, EvmExecution, EvmFork, EvmState, EvmStateContext, EvmWord, ExecutionLimits,
    ExecutionStatus,
};

fn limits(gas: u64) -> Result<ExecutionLimits, EvmCoreError> {
    ExecutionLimits::try_new(EVM_DEFAULT_STEP_LIMIT, gas, EvmFork::CANCUN)
}

#[test]
fn execution_memory_starts_zero_even_when_caller_buffer_does_not() -> Result<(), EvmCoreError> {
    let mut memory = [0xa5_u8; 4];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;
    let report = execution.run(
        &[0x60, 0x02, 0x60, 0x00, 0xf3],
        limits(EVM_DEFAULT_GAS_LIMIT)?,
    )?;

    assert_eq!(
        report.status,
        ExecutionStatus::Returned { offset: 0, len: 2 }
    );
    assert_eq!(execution.memory().as_slice(), [0, 0, 0, 0]);
    Ok(())
}

#[test]
fn execution_requires_destructive_reset_before_reuse() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 4];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;

    let _ = execution.run(&[0x60, 0x2a, 0x00], limits(EVM_DEFAULT_GAS_LIMIT)?)?;
    assert_eq!(
        execution.run(&[0x00], limits(EVM_DEFAULT_GAS_LIMIT)?),
        Err(EvmCoreError::ExecutionAlreadyStarted)
    );

    execution.memory_mut().write_byte(0, 0xa5)?;
    execution.reset()?;
    assert!(execution.stack().is_empty());
    assert_eq!(execution.memory().as_slice(), [0, 0, 0, 0]);
    let report = execution.run(&[0x00], limits(EVM_DEFAULT_GAS_LIMIT)?)?;
    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(report.pc.get(), 0);
    Ok(())
}

#[test]
fn failed_state_run_restores_warm_access_tracking() -> Result<(), EvmCoreError> {
    let mut memory = [];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;
    let mut state = StateFixture;
    let mut accesses = EvmAccessSet::<2, 2>::try_new()?;
    let code = [0x60, 0x01, 0x31, 0x00];

    assert_eq!(
        execution.run_with_state(
            &code,
            limits(100)?,
            EvmStateContext::new(EvmAddress::ZERO),
            &mut state,
            &mut accesses,
        ),
        Err(EvmCoreError::OutOfGas)
    );
    assert_eq!(accesses.address_len(), 0);
    assert_eq!(accesses.storage_len(), 0);
    Ok(())
}

struct StateFixture;

impl EvmState for StateFixture {
    fn account(&mut self, _address: EvmAddress) -> Result<EvmAccount, EvmCoreError> {
        EvmAccount::try_new(true, EvmWord::ZERO, EvmWord::ZERO, 0)
    }

    fn storage(&mut self, _address: EvmAddress, _key: EvmWord) -> Result<EvmWord, EvmCoreError> {
        Ok(EvmWord::ZERO)
    }

    fn copy_code(
        &mut self,
        _address: EvmAddress,
        _code_offset: usize,
        _output: &mut [u8],
    ) -> Result<(), EvmCoreError> {
        Ok(())
    }
}
