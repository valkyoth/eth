use crate::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EvmAccessSet, EvmAccount, EvmAddress,
    EvmCoreError, EvmExecution, EvmFork, EvmGas, EvmState, EvmStateContext, EvmWord,
    ExecutionLimits, ExecutionStatus,
};

fn execution_limits() -> Result<ExecutionLimits, EvmCoreError> {
    ExecutionLimits::try_new(
        EVM_DEFAULT_STEP_LIMIT,
        EVM_DEFAULT_GAS_LIMIT,
        EvmFork::CANCUN,
    )
}

#[test]
fn state_access_requires_explicit_host() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let code = [0x60, 0x01, 0x31];

    assert_eq!(
        execution.run(&code, execution_limits()?),
        Err(EvmCoreError::StateAccessUnavailable)
    );
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_be_slice(&[1])?);
    Ok(())
}

#[test]
fn state_access_reads_balance_and_tracks_warm_accounts() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let mut state = FixtureState::new();
    let mut accesses = EvmAccessSet::<4, 4>::try_new()?;
    let code = [0x60, 0x02, 0x31, 0x60, 0x02, 0x31, 0x00];

    let report = execution.run_with_state(
        &code,
        execution_limits()?,
        EvmStateContext::new(FixtureState::SELF_ADDRESS),
        &mut state,
        &mut accesses,
    )?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(report.gas_used, EvmGas::new(2_706));
    assert_eq!(accesses.address_len(), 1);
    assert_eq!(state.account_reads, 2);
    assert_eq!(execution.stack().peek(1)?, FixtureState::BALANCE);
    assert_eq!(execution.stack().peek(0)?, FixtureState::BALANCE);
    Ok(())
}

#[test]
fn state_access_reads_storage_and_tracks_warm_slots() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let mut state = FixtureState::new();
    let mut accesses = EvmAccessSet::<4, 4>::try_new()?;
    let code = [0x60, 0x03, 0x54, 0x60, 0x03, 0x54, 0x00];

    let report = execution.run_with_state(
        &code,
        execution_limits()?,
        EvmStateContext::new(FixtureState::SELF_ADDRESS),
        &mut state,
        &mut accesses,
    )?;

    assert_eq!(report.gas_used, EvmGas::new(2_206));
    assert_eq!(accesses.address_len(), 1);
    assert_eq!(accesses.storage_len(), 1);
    assert_eq!(state.storage_reads, 2);
    assert_eq!(execution.stack().peek(1)?, FixtureState::STORAGE_VALUE);
    assert_eq!(execution.stack().peek(0)?, FixtureState::STORAGE_VALUE);
    Ok(())
}

#[test]
fn state_access_copies_external_code_into_bounded_memory() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 8];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let mut state = FixtureState::new();
    let mut accesses = EvmAccessSet::<4, 4>::try_new()?;
    let code = [0x60, 0x03, 0x60, 0x01, 0x60, 0x02, 0x60, 0x02, 0x3c, 0x00];

    let report = execution.run_with_state(
        &code,
        execution_limits()?,
        EvmStateContext::new(FixtureState::SELF_ADDRESS),
        &mut state,
        &mut accesses,
    )?;

    assert_eq!(report.gas_used, EvmGas::new(2_618));
    assert_eq!(
        execution
            .memory()
            .as_slice()
            .get(2..5)
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)?,
        [12, 13, 14]
    );
    assert_eq!(state.code_reads, 1);
    Ok(())
}

#[test]
fn state_write_shell_fails_without_popping_stack() -> Result<(), EvmCoreError> {
    let mut memory = [0u8; 0];
    let mut execution = EvmExecution::<16>::try_new(&mut memory)?;
    let mut state = FixtureState::new();
    let mut accesses = EvmAccessSet::<4, 4>::try_new()?;
    let code = [0x60, 0x09, 0x60, 0x03, 0x55];

    assert_eq!(
        execution.run_with_state(
            &code,
            execution_limits()?,
            EvmStateContext::new(FixtureState::SELF_ADDRESS),
            &mut state,
            &mut accesses,
        ),
        Err(EvmCoreError::StateWriteUnsupported)
    );
    assert_eq!(accesses.storage_len(), 1);
    assert_eq!(execution.stack().len(), 2);
    Ok(())
}

#[test]
fn state_access_set_rejects_missing_capacity() {
    assert_eq!(
        EvmAccessSet::<0, 1>::try_new(),
        Err(EvmCoreError::StateAccessListTooSmall)
    );
    assert_eq!(
        EvmAccessSet::<1, 0>::try_new(),
        Err(EvmCoreError::StateAccessListTooSmall)
    );
}

struct FixtureState {
    account_reads: usize,
    storage_reads: usize,
    code_reads: usize,
}

impl FixtureState {
    const BALANCE: EvmWord = EvmWord::from_be_bytes(value_word(44));
    const CODE_HASH: EvmWord = EvmWord::from_be_bytes(value_word(55));
    const SELF_ADDRESS: EvmAddress = EvmAddress::from_bytes(address_bytes(7));
    const STORAGE_VALUE: EvmWord = EvmWord::from_be_bytes(value_word(99));

    const fn new() -> Self {
        Self {
            account_reads: 0,
            storage_reads: 0,
            code_reads: 0,
        }
    }
}

impl EvmState for FixtureState {
    fn account(&mut self, _address: EvmAddress) -> Result<EvmAccount, EvmCoreError> {
        self.account_reads = self.account_reads.saturating_add(1);
        EvmAccount::try_new(true, Self::BALANCE, Self::CODE_HASH, 4)
    }

    fn storage(&mut self, _address: EvmAddress, _key: EvmWord) -> Result<EvmWord, EvmCoreError> {
        self.storage_reads = self.storage_reads.saturating_add(1);
        Ok(Self::STORAGE_VALUE)
    }

    fn copy_code(
        &mut self,
        _address: EvmAddress,
        code_offset: usize,
        output: &mut [u8],
    ) -> Result<(), EvmCoreError> {
        self.code_reads = self.code_reads.saturating_add(1);
        let code = [11, 12, 13, 14];
        for (index, slot) in output.iter_mut().enumerate() {
            let source = code_offset
                .checked_add(index)
                .ok_or(EvmCoreError::StateCodeReadFailed)?;
            *slot = code.get(source).copied().unwrap_or(0);
        }
        Ok(())
    }
}

const fn value_word(value: u8) -> [u8; EvmWord::LEN] {
    let mut bytes = [0u8; EvmWord::LEN];
    bytes[EvmWord::LEN - 1] = value;
    bytes
}

const fn address_bytes(value: u8) -> [u8; EvmAddress::LEN] {
    let mut bytes = [0u8; EvmAddress::LEN];
    bytes[EvmAddress::LEN - 1] = value;
    bytes
}
