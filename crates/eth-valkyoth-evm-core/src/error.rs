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
