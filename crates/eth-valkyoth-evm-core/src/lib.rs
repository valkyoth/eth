#![no_std]
#![forbid(unsafe_code)]
//! Dependency-free `no_std` EVM core domains for `eth`.

#[cfg(feature = "std")]
extern crate std;

mod call;
mod error;
mod execution;
mod fork;
mod gas;
mod jumpdest;
mod memory;
mod opcode;
mod precompile;
mod program_counter;
mod stack;
mod state;
mod state_execution;
mod word;

pub use call::{
    EVM_CALL_DEPTH_LIMIT, EvmCallFramePolicy, EvmCallKind, EvmCallPlan, EvmCreateKind,
    EvmCreatePlan, EvmJournal, EvmJournalCheckpoint, EvmMemoryRange, EvmReturnDataRange,
};
pub use error::EvmCoreError;
pub use execution::{
    EVM_DEFAULT_STEP_LIMIT, EVM_MAX_BYTECODE_LEN, EVM_MAX_STEP_LIMIT, EvmExecution,
    ExecutionLimits, ExecutionReport, ExecutionStatus,
};
pub use fork::{EvmFork, OpcodeTable};
pub use gas::{EVM_DEFAULT_GAS_LIMIT, EVM_MAX_GAS_LIMIT, EvmGas, EvmGasMeter, EvmGasSchedule};
pub use memory::{EVM_MEMORY_LIMIT_BYTES, EvmMemory};
pub use opcode::{EvmOpcode, OpcodeClass, OpcodeInfo};
pub use precompile::{
    EVM_PRECOMPILE_INPUT_LIMIT, EvmPrecompileDescriptor, EvmPrecompileGasPolicy,
    EvmPrecompileImplementation, EvmPrecompileInputPolicy, EvmPrecompileKind, EvmPrecompilePlan,
    EvmPrecompileRegistry, execute_identity,
};
pub use program_counter::ProgramCounter;
pub use stack::{EVM_STACK_LIMIT, EvmStack};
pub use state::{EvmAccessSet, EvmAccessStatus, EvmAccount, EvmAddress, EvmState, EvmStateContext};
pub use word::EvmWord;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "historical_gas_tests.rs"]
mod historical_gas_tests;

#[cfg(test)]
#[path = "call_tests.rs"]
mod call_tests;

#[cfg(test)]
#[path = "precompile_tests.rs"]
mod precompile_tests;
