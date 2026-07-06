#![no_std]
#![forbid(unsafe_code)]
//! Dependency-free `no_std` EVM core domains for `eth`.

#[cfg(feature = "std")]
extern crate std;

mod error;
mod execution;
mod fork;
mod gas;
mod memory;
mod opcode;
mod program_counter;
mod stack;
mod word;

pub use error::EvmCoreError;
pub use execution::{
    EVM_DEFAULT_STEP_LIMIT, EVM_MAX_BYTECODE_LEN, EVM_MAX_STEP_LIMIT, EvmExecution,
    ExecutionLimits, ExecutionReport, ExecutionStatus,
};
pub use fork::{EvmFork, OpcodeTable};
pub use gas::{EVM_DEFAULT_GAS_LIMIT, EVM_MAX_GAS_LIMIT, EvmGas, EvmGasMeter, EvmGasSchedule};
pub use memory::{EVM_MEMORY_LIMIT_BYTES, EvmMemory};
pub use opcode::{EvmOpcode, OpcodeClass, OpcodeInfo};
pub use program_counter::ProgramCounter;
pub use stack::{EVM_STACK_LIMIT, EvmStack};
pub use word::EvmWord;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
