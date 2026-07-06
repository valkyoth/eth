use core::fmt;

/// Deterministic error domain for the first-party EVM core types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmCoreError {
    /// The requested stack capacity is zero.
    StackCapacityZero,
    /// The requested stack capacity is larger than the EVM stack limit.
    StackCapacityTooLarge,
    /// Pushing would exceed the configured stack capacity.
    StackOverflow,
    /// Popping would read from an empty stack.
    StackUnderflow,
    /// The requested memory view is larger than the release hard limit.
    MemoryTooLarge,
    /// The requested memory offset is outside the memory view.
    MemoryOffsetOutOfBounds,
    /// Advancing the program counter would overflow `usize`.
    ProgramCounterOverflow,
    /// A word constructor received more than 32 bytes.
    WordInputTooLarge,
    /// The bytecode input is larger than the release hard limit.
    BytecodeTooLarge,
    /// The execution step limit is zero.
    ExecutionStepLimitZero,
    /// The execution step limit exceeds the release hard limit.
    ExecutionStepLimitTooLarge,
    /// Execution reached the configured step limit before halting.
    ExecutionStepLimitReached,
    /// The execution gas limit is zero.
    ExecutionGasLimitZero,
    /// The execution gas limit exceeds the release hard limit.
    ExecutionGasLimitTooLarge,
    /// Execution ran out of gas before applying opcode side effects.
    OutOfGas,
    /// Gas or memory expansion arithmetic overflowed.
    GasOverflow,
    /// The host state access list cannot track any address or storage slot.
    StateAccessListTooSmall,
    /// The host state access list capacity was exhausted.
    StateAccessListFull,
    /// A host account read failed.
    StateAccountReadFailed,
    /// A host storage read failed.
    StateStorageReadFailed,
    /// A host code read failed.
    StateCodeReadFailed,
    /// Host account code exceeds the release hard code-size cap.
    StateCodeTooLarge,
    /// A state opcode was executed without an explicit host state snapshot.
    StateAccessUnavailable,
    /// State writes are not admitted until the journaled call/create release.
    StateWriteUnsupported,
    /// A `PUSHn` immediate extends beyond the bytecode input.
    PushImmediateOutOfBounds,
    /// A dynamic jump target is not a valid `JUMPDEST`.
    InvalidJumpDestination,
    /// A return or revert range is outside the memory view.
    ReturnRangeOutOfBounds,
    /// The opcode byte is not supported by the current skeleton table.
    UnsupportedOpcode,
    /// The fork identifier is not supported by the current skeleton table.
    UnsupportedFork,
}

impl EvmCoreError {
    /// Returns a stable category string for logs and external reports.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::StackCapacityZero => "stack_capacity_zero",
            Self::StackCapacityTooLarge => "stack_capacity_too_large",
            Self::StackOverflow => "stack_overflow",
            Self::StackUnderflow => "stack_underflow",
            Self::MemoryTooLarge => "memory_too_large",
            Self::MemoryOffsetOutOfBounds => "memory_offset_out_of_bounds",
            Self::ProgramCounterOverflow => "program_counter_overflow",
            Self::WordInputTooLarge => "word_input_too_large",
            Self::BytecodeTooLarge => "bytecode_too_large",
            Self::ExecutionStepLimitZero => "execution_step_limit_zero",
            Self::ExecutionStepLimitTooLarge => "execution_step_limit_too_large",
            Self::ExecutionStepLimitReached => "execution_step_limit_reached",
            Self::ExecutionGasLimitZero => "execution_gas_limit_zero",
            Self::ExecutionGasLimitTooLarge => "execution_gas_limit_too_large",
            Self::OutOfGas => "out_of_gas",
            Self::GasOverflow => "gas_overflow",
            Self::StateAccessListTooSmall => "state_access_list_too_small",
            Self::StateAccessListFull => "state_access_list_full",
            Self::StateAccountReadFailed => "state_account_read_failed",
            Self::StateStorageReadFailed => "state_storage_read_failed",
            Self::StateCodeReadFailed => "state_code_read_failed",
            Self::StateCodeTooLarge => "state_code_too_large",
            Self::StateAccessUnavailable => "state_access_unavailable",
            Self::StateWriteUnsupported => "state_write_unsupported",
            Self::PushImmediateOutOfBounds => "push_immediate_out_of_bounds",
            Self::InvalidJumpDestination => "invalid_jump_destination",
            Self::ReturnRangeOutOfBounds => "return_range_out_of_bounds",
            Self::UnsupportedOpcode => "unsupported_opcode",
            Self::UnsupportedFork => "unsupported_fork",
        }
    }
}

impl fmt::Display for EvmCoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EvmCoreError {}
