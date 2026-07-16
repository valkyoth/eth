use super::EvmExecution;
use crate::{EvmCoreError, EvmMemory, EvmStack, ProgramCounter};

impl<'a, const STACK: usize> EvmExecution<'a, STACK> {
    /// Creates a fresh execution context over a zero-initialized memory view.
    pub fn try_new(memory: &'a mut [u8]) -> Result<Self, EvmCoreError> {
        Ok(Self {
            stack: EvmStack::try_new()?,
            memory: EvmMemory::try_new(memory)?,
            pc: ProgramCounter::new(0),
            started: false,
        })
    }

    /// Destructively resets stack, memory, and program counter for one new run.
    pub fn reset(&mut self) -> Result<(), EvmCoreError> {
        self.stack = EvmStack::try_new()?;
        self.memory.as_mut_slice().fill(0);
        self.pc = ProgramCounter::new(0);
        self.started = false;
        Ok(())
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

    pub(super) fn begin_run(&mut self) -> Result<(), EvmCoreError> {
        if self.started {
            return Err(EvmCoreError::ExecutionAlreadyStarted);
        }
        self.started = true;
        Ok(())
    }
}
