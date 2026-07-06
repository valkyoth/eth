use crate::{EvmCoreError, EvmMemory, EvmOpcode, EvmStack, EvmWord, ProgramCounter};
use core::cmp::Ordering;

/// Default maximum instruction count for local deterministic execution tests.
pub const EVM_DEFAULT_STEP_LIMIT: usize = 100_000;
/// Hard maximum instruction count accepted by the bootstrap interpreter.
pub const EVM_MAX_STEP_LIMIT: usize = 1_000_000;

/// Bounded execution limits for the native EVM core interpreter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionLimits {
    max_steps: usize,
}

impl ExecutionLimits {
    /// Constructs execution limits.
    pub const fn try_new(max_steps: usize) -> Result<Self, EvmCoreError> {
        if max_steps == 0 {
            return Err(EvmCoreError::ExecutionStepLimitZero);
        }
        if max_steps > EVM_MAX_STEP_LIMIT {
            return Err(EvmCoreError::ExecutionStepLimitTooLarge);
        }
        Ok(Self { max_steps })
    }

    /// Returns the maximum number of executed instructions.
    #[must_use]
    pub const fn max_steps(self) -> usize {
        self.max_steps
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

    /// Executes bytecode until it halts or fails closed.
    pub fn run(
        &mut self,
        bytecode: &[u8],
        limits: ExecutionLimits,
    ) -> Result<ExecutionReport, EvmCoreError> {
        let mut steps = 0usize;
        loop {
            if steps >= limits.max_steps() {
                return Err(EvmCoreError::ExecutionStepLimitReached);
            }
            let pc = self.pc.get();
            let opcode_byte = match bytecode.get(pc) {
                Some(byte) => *byte,
                None => return Ok(self.report(ExecutionStatus::Stopped, steps)),
            };
            steps = steps
                .checked_add(1)
                .ok_or(EvmCoreError::ExecutionStepLimitReached)?;
            let opcode = EvmOpcode::new(opcode_byte);
            match opcode.byte() {
                0x00 => return Ok(self.report(ExecutionStatus::Stopped, steps)),
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
                0x56 => self.jump(bytecode)?,
                0x57 => self.jumpi(bytecode)?,
                0x58 => self.stack.push(EvmWord::from_usize(pc))?,
                0x5b => {}
                0x60..=0x7f => self.push_immediate(bytecode, opcode)?,
                0x80..=0x8f => self.dup(opcode)?,
                0x90..=0x9f => self.swap(opcode)?,
                0xf3 => return self.return_or_revert(steps, false),
                0xfd => return self.return_or_revert(steps, true),
                _ => return Err(EvmCoreError::UnsupportedOpcode),
            }
            self.pc = self.next_pc(bytecode, pc, opcode)?;
        }
    }

    fn report(&self, status: ExecutionStatus, steps: usize) -> ExecutionReport {
        ExecutionReport {
            status,
            steps,
            pc: self.pc,
            stack_len: self.stack.len(),
        }
    }

    fn next_pc(
        &self,
        bytecode: &[u8],
        pc: usize,
        opcode: EvmOpcode,
    ) -> Result<ProgramCounter, EvmCoreError> {
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
        if width > 0 && next > bytecode.len() {
            return Err(EvmCoreError::PushImmediateOutOfBounds);
        }
        Ok(ProgramCounter::new(next))
    }

    fn binary_word(&mut self, op: fn(EvmWord, EvmWord) -> EvmWord) -> Result<(), EvmCoreError> {
        let rhs = self.stack.pop()?;
        let lhs = self.stack.pop()?;
        self.stack.push(op(lhs, rhs))
    }

    fn unary_word(&mut self, op: fn(EvmWord) -> EvmWord) -> Result<(), EvmCoreError> {
        let value = self.stack.pop()?;
        self.stack.push(op(value))
    }

    fn compare_word(&mut self, expected: Ordering) -> Result<(), EvmCoreError> {
        let rhs = self.stack.pop()?;
        let lhs = self.stack.pop()?;
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

    fn jump(&mut self, bytecode: &[u8]) -> Result<(), EvmCoreError> {
        let target = self.stack.pop()?.to_usize()?;
        if !valid_jumpdest(bytecode, target)? {
            return Err(EvmCoreError::InvalidJumpDestination);
        }
        self.pc = ProgramCounter::new(target);
        Ok(())
    }

    fn jumpi(&mut self, bytecode: &[u8]) -> Result<(), EvmCoreError> {
        let target = self.stack.pop()?.to_usize()?;
        let condition = self.stack.pop()?;
        if condition.is_zero() {
            self.pc = self.pc.advance(1)?;
            return Ok(());
        }
        if !valid_jumpdest(bytecode, target)? {
            return Err(EvmCoreError::InvalidJumpDestination);
        }
        self.pc = ProgramCounter::new(target);
        Ok(())
    }

    fn return_or_revert(
        &mut self,
        steps: usize,
        revert: bool,
    ) -> Result<ExecutionReport, EvmCoreError> {
        let offset = self.stack.pop()?.to_usize()?;
        let len = self.stack.pop()?.to_usize()?;
        self.memory
            .check_range(offset, len)
            .map_err(|_| EvmCoreError::ReturnRangeOutOfBounds)?;
        let status = if revert {
            ExecutionStatus::Reverted { offset, len }
        } else {
            ExecutionStatus::Returned { offset, len }
        };
        Ok(self.report(status, steps))
    }
}

fn valid_jumpdest(bytecode: &[u8], target: usize) -> Result<bool, EvmCoreError> {
    if target >= bytecode.len() {
        return Ok(false);
    }
    let mut pc = 0usize;
    while pc < bytecode.len() {
        let opcode = match bytecode.get(pc) {
            Some(byte) => EvmOpcode::new(*byte),
            None => return Ok(false),
        };
        if pc == target {
            return Ok(opcode.byte() == EvmOpcode::JUMPDEST.byte());
        }
        let width = usize::from(opcode.push_width().unwrap_or(0));
        let advance = 1usize
            .checked_add(width)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        pc = pc
            .checked_add(advance)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
    }
    Ok(false)
}
