use crate::{
    EvmAccessSet, EvmAccessStatus, EvmAddress, EvmCoreError, EvmExecution, EvmGasMeter,
    EvmGasSchedule, EvmOpcode, EvmState, EvmStateContext, EvmWord,
};

pub(crate) trait StateExecutionHost {
    fn execute_state_opcode<const STACK: usize>(
        &mut self,
        execution: &mut EvmExecution<'_, STACK>,
        opcode: EvmOpcode,
        schedule: EvmGasSchedule,
        gas_meter: &mut EvmGasMeter,
    ) -> Result<(), EvmCoreError>;
}

pub(crate) struct NoState;

impl StateExecutionHost for NoState {
    fn execute_state_opcode<const STACK: usize>(
        &mut self,
        _execution: &mut EvmExecution<'_, STACK>,
        _opcode: EvmOpcode,
        _schedule: EvmGasSchedule,
        _gas_meter: &mut EvmGasMeter,
    ) -> Result<(), EvmCoreError> {
        Err(EvmCoreError::StateAccessUnavailable)
    }
}

pub(crate) struct HostState<'a, const ADDRESSES: usize, const STORAGE: usize, S: EvmState> {
    pub(crate) context: EvmStateContext,
    pub(crate) state: &'a mut S,
    pub(crate) accesses: &'a mut EvmAccessSet<ADDRESSES, STORAGE>,
}

impl<const ADDRESSES: usize, const STORAGE: usize, S: EvmState> StateExecutionHost
    for HostState<'_, ADDRESSES, STORAGE, S>
{
    fn execute_state_opcode<const STACK: usize>(
        &mut self,
        execution: &mut EvmExecution<'_, STACK>,
        opcode: EvmOpcode,
        schedule: EvmGasSchedule,
        gas_meter: &mut EvmGasMeter,
    ) -> Result<(), EvmCoreError> {
        schedule.require_warm_cold_state_access()?;
        match opcode.byte() {
            0x31 => balance(execution, schedule, gas_meter, self),
            0x3b => extcodesize(execution, schedule, gas_meter, self),
            0x3c => extcodecopy(execution, schedule, gas_meter, self),
            0x3f => extcodehash(execution, schedule, gas_meter, self),
            0x47 => selfbalance(execution, schedule, gas_meter, self),
            0x54 => sload(execution, schedule, gas_meter, self),
            0x55 => sstore_shell(execution, schedule, gas_meter, self),
            _ => Err(EvmCoreError::UnsupportedOpcode),
        }
    }
}

fn charge_account_access<const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    address: EvmAddress,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    let warm = host.accesses.warm_address(address)? == EvmAccessStatus::Warm;
    gas_meter.charge(schedule.account_access_cost(warm))
}

fn map_account_error(error: EvmCoreError) -> EvmCoreError {
    if error == EvmCoreError::StateCodeTooLarge {
        error
    } else {
        EvmCoreError::StateAccountReadFailed
    }
}

fn balance<const STACK: usize, const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    execution: &mut EvmExecution<'_, STACK>,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    let address = EvmAddress::from_word(execution.stack().peek(0)?);
    charge_account_access(address, schedule, gas_meter, host)?;
    let account = host.state.account(address).map_err(map_account_error)?;
    let _ = execution.stack_mut().pop()?;
    execution.stack_mut().push(account.balance)
}

fn extcodesize<const STACK: usize, const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    execution: &mut EvmExecution<'_, STACK>,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    let address = EvmAddress::from_word(execution.stack().peek(0)?);
    charge_account_access(address, schedule, gas_meter, host)?;
    let account = host.state.account(address).map_err(map_account_error)?;
    let _ = execution.stack_mut().pop()?;
    execution
        .stack_mut()
        .push(EvmWord::from_usize(account.code_len))
}

fn extcodehash<const STACK: usize, const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    execution: &mut EvmExecution<'_, STACK>,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    let address = EvmAddress::from_word(execution.stack().peek(0)?);
    charge_account_access(address, schedule, gas_meter, host)?;
    let account = host.state.account(address).map_err(map_account_error)?;
    let _ = execution.stack_mut().pop()?;
    execution.stack_mut().push(if account.exists {
        account.code_hash
    } else {
        EvmWord::ZERO
    })
}

fn extcodecopy<const STACK: usize, const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    execution: &mut EvmExecution<'_, STACK>,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    let address = EvmAddress::from_word(execution.stack().peek(0)?);
    let memory_offset = execution.stack().peek(1)?.to_usize()?;
    let code_offset = execution.stack().peek(2)?.to_usize()?;
    let len = execution.stack().peek(3)?.to_usize()?;
    execution.memory().check_range(memory_offset, len)?;
    charge_account_access(address, schedule, gas_meter, host)?;
    gas_meter.charge(schedule.copy_cost(len)?)?;
    gas_meter.charge_memory_range(schedule, memory_offset, len)?;
    let output = execution
        .memory_mut()
        .checked_range_mut(memory_offset, len)?;
    host.state
        .copy_code(address, code_offset, output)
        .map_err(|_| EvmCoreError::StateCodeReadFailed)?;
    for _ in 0..4 {
        let _ = execution.stack_mut().pop()?;
    }
    Ok(())
}

fn selfbalance<const STACK: usize, const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    execution: &mut EvmExecution<'_, STACK>,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    gas_meter.charge(schedule.selfbalance_cost())?;
    let account = host
        .state
        .account(host.context.address)
        .map_err(map_account_error)?;
    execution.stack_mut().push(account.balance)
}

fn sload<const STACK: usize, const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    execution: &mut EvmExecution<'_, STACK>,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    let key = execution.stack().peek(0)?;
    let warm = host.accesses.warm_storage(host.context.address, key)? == EvmAccessStatus::Warm;
    gas_meter.charge(schedule.storage_access_cost(warm))?;
    let value = host
        .state
        .storage(host.context.address, key)
        .map_err(|_| EvmCoreError::StateStorageReadFailed)?;
    let _ = execution.stack_mut().pop()?;
    execution.stack_mut().push(value)
}

fn sstore_shell<const STACK: usize, const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
    execution: &mut EvmExecution<'_, STACK>,
    schedule: EvmGasSchedule,
    gas_meter: &mut EvmGasMeter,
    host: &mut HostState<'_, ADDRESSES, STORAGE, S>,
) -> Result<(), EvmCoreError> {
    let key = execution.stack().peek(0)?;
    let _value = execution.stack().peek(1)?;
    let warm = host.accesses.warm_storage(host.context.address, key)? == EvmAccessStatus::Warm;
    gas_meter.charge(schedule.storage_access_cost(warm))?;
    Err(EvmCoreError::StateWriteUnsupported)
}
