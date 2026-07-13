use crate::{
    EvmAccessSet, EvmCallFramePolicy, EvmCallKind, EvmCallPlan, EvmCoreError, EvmCreateKind,
    EvmCreatePlan, EvmFork, EvmGas, EvmGasMeter, EvmGasSchedule, EvmMemory, EvmMemoryRange,
    EvmOpcode, EvmStack, EvmState, EvmStateContext, EvmWord, ProgramCounter,
    call::EvmCallCreatePlan,
    jumpdest::JumpdestMap,
    state_execution::{HostState, NoState, StateExecutionHost},
};
use core::cmp::Ordering;

/// Default maximum instruction count for local deterministic execution tests.
pub const EVM_DEFAULT_STEP_LIMIT: usize = 100_000;
/// Hard maximum instruction count accepted by the bootstrap interpreter.
pub const EVM_MAX_STEP_LIMIT: usize = 1_000_000;
/// Hard maximum bytecode length accepted by the bootstrap interpreter.
pub const EVM_MAX_BYTECODE_LEN: usize = 24_576;

/// Bounded execution limits for the native EVM core interpreter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionLimits {
    max_steps: usize,
    gas_limit: EvmGas,
    fork: EvmFork,
}

impl ExecutionLimits {
    /// Constructs execution limits.
    pub const fn try_new(
        max_steps: usize,
        gas_limit: u64,
        fork: EvmFork,
    ) -> Result<Self, EvmCoreError> {
        if max_steps == 0 {
            return Err(EvmCoreError::ExecutionStepLimitZero);
        }
        if max_steps > EVM_MAX_STEP_LIMIT {
            return Err(EvmCoreError::ExecutionStepLimitTooLarge);
        }
        let gas_limit = EvmGas::new(gas_limit);
        match EvmGasSchedule::for_fork(fork) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
        match EvmGasMeter::try_new(gas_limit) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
        Ok(Self {
            max_steps,
            gas_limit,
            fork,
        })
    }

    /// Returns the maximum number of executed instructions.
    #[must_use]
    pub const fn max_steps(self) -> usize {
        self.max_steps
    }

    /// Returns the maximum gas admitted for this execution.
    #[must_use]
    pub const fn gas_limit(self) -> EvmGas {
        self.gas_limit
    }

    /// Returns the fork used for opcode gas costs.
    #[must_use]
    pub const fn fork(self) -> EvmFork {
        self.fork
    }
}

/// Final execution status for the supported opcode subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    /// Execution stopped normally.
    Stopped,
    /// Execution returned a memory range.
    Returned {
        /// Return data offset in memory.
        offset: usize,
        /// Return data length in memory.
        len: usize,
    },
    /// Execution reverted with a memory range.
    Reverted {
        /// Revert data offset in memory.
        offset: usize,
        /// Revert data length in memory.
        len: usize,
    },
}

/// Deterministic execution report for the supported opcode subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionReport {
    /// Final status.
    pub status: ExecutionStatus,
    /// Number of executed instructions.
    pub steps: usize,
    /// Final program counter.
    pub pc: ProgramCounter,
    /// Final stack depth.
    pub stack_len: usize,
    /// Gas consumed by executed opcodes and memory expansion.
    pub gas_used: EvmGas,
    /// Gas remaining when execution halted.
    pub gas_remaining: EvmGas,
}

/// Small no-alloc interpreter for the audited bootstrap opcode set.
#[derive(Debug, Eq, PartialEq)]
pub struct EvmExecution<'a, const STACK: usize> {
    stack: EvmStack<STACK>,
    memory: EvmMemory<'a>,
    pc: ProgramCounter,
}

impl<'a, const STACK: usize> EvmExecution<'a, STACK> {
    /// Creates a fresh execution context over a caller-provided memory view.
    pub fn try_new(memory: &'a mut [u8]) -> Result<Self, EvmCoreError> {
        Ok(Self {
            stack: EvmStack::try_new()?,
            memory: EvmMemory::try_new(memory)?,
            pc: ProgramCounter::new(0),
        })
    }

    /// Returns the execution stack.
    #[must_use]
    pub const fn stack(&self) -> &EvmStack<STACK> {
        &self.stack
    }

    /// Mutably returns the execution stack.
    pub fn stack_mut(&mut self) -> &mut EvmStack<STACK> {
        &mut self.stack
    }

    /// Returns the execution memory view.
    #[must_use]
    pub const fn memory(&self) -> &EvmMemory<'a> {
        &self.memory
    }

    /// Mutably returns the execution memory view inside this crate.
    pub(crate) fn memory_mut(&mut self) -> &mut EvmMemory<'a> {
        &mut self.memory
    }

    /// Executes bytecode until it halts or fails closed.
    pub fn run(
        &mut self,
        bytecode: &[u8],
        limits: ExecutionLimits,
    ) -> Result<ExecutionReport, EvmCoreError> {
        self.run_inner(bytecode, limits, &mut NoState)
    }

    /// Executes bytecode with an explicit host state snapshot.
    pub fn run_with_state<const ADDRESSES: usize, const STORAGE: usize, S: EvmState>(
        &mut self,
        bytecode: &[u8],
        limits: ExecutionLimits,
        context: EvmStateContext,
        state: &mut S,
        accesses: &mut EvmAccessSet<ADDRESSES, STORAGE>,
    ) -> Result<ExecutionReport, EvmCoreError> {
        let mut host = HostState {
            context,
            state,
            accesses,
        };
        self.run_inner(bytecode, limits, &mut host)
    }

    fn run_inner<H: StateExecutionHost>(
        &mut self,
        bytecode: &[u8],
        limits: ExecutionLimits,
        host: &mut H,
    ) -> Result<ExecutionReport, EvmCoreError> {
        let jumpdests = JumpdestMap::try_new(bytecode)?;
        let schedule = EvmGasSchedule::for_fork(limits.fork())?;
        let frame = EvmCallFramePolicy::root();
        let mut gas_meter = EvmGasMeter::try_new(limits.gas_limit())?;
        let mut steps = 0usize;
        loop {
            if steps >= limits.max_steps() {
                return Err(EvmCoreError::ExecutionStepLimitReached);
            }
            let pc = self.pc.get();
            let opcode_byte = match bytecode.get(pc) {
                Some(byte) => *byte,
                None => return self.report(ExecutionStatus::Stopped, steps, gas_meter),
            };
            steps = steps
                .checked_add(1)
                .ok_or(EvmCoreError::ExecutionStepLimitReached)?;
            let opcode = EvmOpcode::new(opcode_byte);
            if !schedule.fork().opcode_is_introduced(opcode) {
                return Err(EvmCoreError::UnsupportedOpcode);
            }
            if opcode.is_state_access() {
                host.execute_state_opcode(self, opcode, schedule, &mut gas_meter)?;
            } else if opcode.is_call_create() {
                let _ = self.plan_call_create(opcode, frame)?;
                return Err(EvmCoreError::CallCreateExecutionUnsupported);
            } else {
                gas_meter.charge(schedule.base_cost(opcode)?)?;
                match opcode.byte() {
                    0x00 => return self.report(ExecutionStatus::Stopped, steps, gas_meter),
                    0x01 => self.binary_word(EvmWord::wrapping_add_word)?,
                    0x02 => self.binary_word(EvmWord::wrapping_mul_word)?,
                    0x03 => self.binary_word(EvmWord::wrapping_sub_word)?,
                    0x10 => self.compare_word(Ordering::Less)?,
                    0x11 => self.compare_word(Ordering::Greater)?,
                    0x14 => self.equal_word()?,
                    0x15 => self.iszero_word()?,
                    0x16 => self.binary_word(EvmWord::bitand_word)?,
                    0x17 => self.binary_word(EvmWord::bitor_word)?,
                    0x18 => self.binary_word(EvmWord::bitxor_word)?,
                    0x19 => self.unary_word(EvmWord::bitnot_word)?,
                    0x50 => {
                        let _ = self.stack.pop()?;
                    }
                    0x56 => self.jump(&jumpdests)?,
                    0x57 => self.jumpi(&jumpdests)?,
                    0x58 => self.stack.push(EvmWord::from_usize(pc))?,
                    0x5b => {}
                    0x60..=0x7f => self.push_immediate(bytecode, opcode)?,
                    0x80..=0x8f => self.dup(opcode)?,
                    0x90..=0x9f => self.swap(opcode)?,
                    0xf3 => return self.return_or_revert(steps, false, schedule, &mut gas_meter),
                    0xfd => return self.return_or_revert(steps, true, schedule, &mut gas_meter),
                    _ => return Err(EvmCoreError::UnsupportedOpcode),
                }
            }
            self.pc = self.next_pc(pc, opcode)?;
        }
    }

    fn report(
        &self,
        status: ExecutionStatus,
        steps: usize,
        gas_meter: EvmGasMeter,
    ) -> Result<ExecutionReport, EvmCoreError> {
        Ok(ExecutionReport {
            status,
            steps,
            pc: self.pc,
            stack_len: self.stack.len(),
            gas_used: gas_meter.used(),
            gas_remaining: gas_meter.remaining()?,
        })
    }

    fn next_pc(&self, pc: usize, opcode: EvmOpcode) -> Result<ProgramCounter, EvmCoreError> {
        if opcode.byte() == EvmOpcode::JUMP.byte() || opcode.byte() == EvmOpcode::JUMPI.byte() {
            return Ok(self.pc);
        }
        let width = usize::from(opcode.push_width().unwrap_or(0));
        let advance = 1usize
            .checked_add(width)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        let next = pc
            .checked_add(advance)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        Ok(ProgramCounter::new(next))
    }

    fn binary_word(&mut self, op: fn(EvmWord, EvmWord) -> EvmWord) -> Result<(), EvmCoreError> {
        let lhs = self.stack.pop()?;
        let rhs = self.stack.pop()?;
        self.stack.push(op(lhs, rhs))
    }

    fn unary_word(&mut self, op: fn(EvmWord) -> EvmWord) -> Result<(), EvmCoreError> {
        let value = self.stack.pop()?;
        self.stack.push(op(value))
    }

    fn compare_word(&mut self, expected: Ordering) -> Result<(), EvmCoreError> {
        let lhs = self.stack.pop()?;
        let rhs = self.stack.pop()?;
        self.stack
            .push(EvmWord::from_bool(lhs.cmp(&rhs) == expected))
    }

    fn equal_word(&mut self) -> Result<(), EvmCoreError> {
        let rhs = self.stack.pop()?;
        let lhs = self.stack.pop()?;
        self.stack.push(EvmWord::from_bool(lhs == rhs))
    }

    fn iszero_word(&mut self) -> Result<(), EvmCoreError> {
        let value = self.stack.pop()?;
        self.stack.push(EvmWord::from_bool(value.is_zero()))
    }

    fn push_immediate(&mut self, bytecode: &[u8], opcode: EvmOpcode) -> Result<(), EvmCoreError> {
        let width = usize::from(
            opcode
                .push_width()
                .ok_or(EvmCoreError::PushImmediateOutOfBounds)?,
        );
        let start = self
            .pc
            .get()
            .checked_add(1)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        let end = start
            .checked_add(width)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        if end > bytecode.len() {
            return Err(EvmCoreError::PushImmediateOutOfBounds);
        }
        let mut bytes = [0u8; EvmWord::LEN];
        for offset in 0..width {
            let source_index = start
                .checked_add(offset)
                .ok_or(EvmCoreError::ProgramCounterOverflow)?;
            let target_index = EvmWord::LEN
                .checked_sub(width)
                .and_then(|base| base.checked_add(offset))
                .ok_or(EvmCoreError::ProgramCounterOverflow)?;
            let source = bytecode
                .get(source_index)
                .ok_or(EvmCoreError::PushImmediateOutOfBounds)?;
            let target = bytes
                .get_mut(target_index)
                .ok_or(EvmCoreError::PushImmediateOutOfBounds)?;
            *target = *source;
        }
        self.stack.push(EvmWord::from_be_bytes(bytes))
    }

    fn dup(&mut self, opcode: EvmOpcode) -> Result<(), EvmCoreError> {
        let depth = usize::from(opcode.dup_depth().ok_or(EvmCoreError::UnsupportedOpcode)?);
        self.stack.dup(depth)
    }

    fn swap(&mut self, opcode: EvmOpcode) -> Result<(), EvmCoreError> {
        let depth = usize::from(opcode.swap_depth().ok_or(EvmCoreError::UnsupportedOpcode)?);
        self.stack.swap_with_top(depth)
    }

    fn jump(&mut self, jumpdests: &JumpdestMap) -> Result<(), EvmCoreError> {
        let target = self.stack.pop()?.to_usize()?;
        if !jumpdests.contains(target) {
            return Err(EvmCoreError::InvalidJumpDestination);
        }
        self.pc = ProgramCounter::new(target);
        Ok(())
    }

    fn jumpi(&mut self, jumpdests: &JumpdestMap) -> Result<(), EvmCoreError> {
        let destination = self.stack.pop()?;
        let condition = self.stack.pop()?;
        if condition.is_zero() {
            self.pc = self.pc.advance(1)?;
            return Ok(());
        }
        let target = destination.to_usize()?;
        if !jumpdests.contains(target) {
            return Err(EvmCoreError::InvalidJumpDestination);
        }
        self.pc = ProgramCounter::new(target);
        Ok(())
    }

    fn return_or_revert(
        &mut self,
        steps: usize,
        revert: bool,
        schedule: EvmGasSchedule,
        gas_meter: &mut EvmGasMeter,
    ) -> Result<ExecutionReport, EvmCoreError> {
        let len = self.stack.peek(1)?.to_usize()?;
        let offset = if len == 0 {
            0
        } else {
            self.stack.peek(0)?.to_usize()?
        };
        self.memory
            .check_range(offset, len)
            .map_err(|_| EvmCoreError::ReturnRangeOutOfBounds)?;
        gas_meter.charge_memory_range(schedule, offset, len)?;
        let _ = self.stack.pop()?;
        let _ = self.stack.pop()?;
        let status = if revert {
            ExecutionStatus::Reverted { offset, len }
        } else {
            ExecutionStatus::Returned { offset, len }
        };
        self.report(status, steps, *gas_meter)
    }

    pub(crate) fn plan_call_create(
        &self,
        opcode: EvmOpcode,
        frame: EvmCallFramePolicy,
    ) -> Result<EvmCallCreatePlan, EvmCoreError> {
        match opcode.byte() {
            0xf0 => {
                let value = self.stack.peek(0)?;
                let init_code = self.memory_range(1, 2)?;
                let plan = EvmCreatePlan::try_new(
                    EvmCreateKind::Create,
                    value,
                    init_code,
                    EvmWord::ZERO,
                    frame,
                )?;
                Ok(EvmCallCreatePlan::Create(plan))
            }
            0xf1 | 0xf2 => {
                let kind = if opcode.byte() == 0xf1 {
                    EvmCallKind::Call
                } else {
                    EvmCallKind::CallCode
                };
                let plan = EvmCallPlan::try_new(
                    kind,
                    self.stack.peek(0)?,
                    crate::EvmAddress::from_word(self.stack.peek(1)?),
                    self.stack.peek(2)?,
                    self.memory_range(3, 4)?,
                    self.memory_range(5, 6)?,
                    frame,
                )?;
                Ok(EvmCallCreatePlan::Call(plan))
            }
            0xf4 | 0xfa => {
                let kind = if opcode.byte() == 0xf4 {
                    EvmCallKind::DelegateCall
                } else {
                    EvmCallKind::StaticCall
                };
                let plan = EvmCallPlan::try_new(
                    kind,
                    self.stack.peek(0)?,
                    crate::EvmAddress::from_word(self.stack.peek(1)?),
                    EvmWord::ZERO,
                    self.memory_range(2, 3)?,
                    self.memory_range(4, 5)?,
                    frame,
                )?;
                Ok(EvmCallCreatePlan::Call(plan))
            }
            0xf5 => {
                let value = self.stack.peek(0)?;
                let init_code = self.memory_range(1, 2)?;
                let plan = EvmCreatePlan::try_new(
                    EvmCreateKind::Create2,
                    value,
                    init_code,
                    self.stack.peek(3)?,
                    frame,
                )?;
                Ok(EvmCallCreatePlan::Create(plan))
            }
            _ => Err(EvmCoreError::UnsupportedOpcode),
        }
    }

    fn memory_range(
        &self,
        offset_depth: usize,
        len_depth: usize,
    ) -> Result<EvmMemoryRange, EvmCoreError> {
        let len = self.stack.peek(len_depth)?.to_usize()?;
        let offset = if len == 0 {
            0
        } else {
            self.stack.peek(offset_depth)?.to_usize()?
        };
        self.memory.check_range(offset, len)?;
        EvmMemoryRange::try_new(offset, len)
    }
}
