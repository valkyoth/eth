#![no_std]
#![forbid(unsafe_code)]
//! Dependency-free `no_std` EVM core domains for `eth`.

#[cfg(feature = "std")]
extern crate std;

mod bn254;
mod bn254_field;
mod bn254_g2;
mod bn254_pairing;
mod bn254_tower;
mod call;
mod ecrecover;
mod error;
mod execution;
mod fork;
mod gas;
mod hash_precompile;
mod jumpdest;
mod memory;
mod modexp;
mod modexp_execute;
#[cfg(feature = "testing")]
mod modexp_testing;
mod opcode;
mod precompile;
mod precompile_charge;
mod precompile_execute;
mod program_counter;
mod ripemd160;
mod sha256;
mod stack;
mod state;
mod state_execution;
mod word;

pub use bn254::{
    EVM_BN254_ADD_INPUT_BYTES, EVM_BN254_MUL_INPUT_BYTES, EVM_BN254_POINT_BYTES, execute_bn254_add,
    execute_bn254_mul,
};
pub use bn254_pairing::{
    EVM_BN254_PAIRING_ITEM_BYTES, EVM_BN254_PAIRING_OUTPUT_BYTES, execute_bn254_pairing,
    parse_bn254_pairing_input,
};
pub use call::{
    EVM_CALL_DEPTH_LIMIT, EvmCallFramePolicy, EvmCallKind, EvmCallPlan, EvmCreateKind,
    EvmCreatePlan, EvmJournal, EvmJournalCheckpoint, EvmMemoryRange, EvmReturnDataRange,
};
pub use ecrecover::{
    EVM_ECRECOVER_INPUT_BYTES, EVM_ECRECOVER_PUBLIC_KEY_BYTES, EvmEcRecoverBackend,
    EvmEcRecoverSignature, EvmPrecompileKeccak256, execute_ecrecover,
};
pub use error::EvmCoreError;
pub use execution::{
    EVM_DEFAULT_STEP_LIMIT, EVM_MAX_BYTECODE_LEN, EVM_MAX_STEP_LIMIT, EvmExecution,
    ExecutionLimits, ExecutionReport, ExecutionStatus,
};
pub use fork::{EvmFork, OpcodeTable};
pub use gas::{EVM_DEFAULT_GAS_LIMIT, EVM_MAX_GAS_LIMIT, EvmGas, EvmGasMeter, EvmGasSchedule};
pub use memory::{EVM_MEMORY_LIMIT_BYTES, EvmMemory};
pub use modexp::{
    EVM_MODEXP_HEADER_BYTES, EVM_MODEXP_MAX_OPERAND_BYTES, EvmModExpInput, execute_modexp,
    parse_modexp_input,
};
#[cfg(feature = "testing")]
pub use modexp_testing::testing_modexp_gas_cost;
pub use opcode::{EvmOpcode, OpcodeClass, OpcodeInfo};
pub use precompile::{
    EVM_PRECOMPILE_INPUT_LIMIT, EvmPrecompileDescriptor, EvmPrecompileGasPolicy,
    EvmPrecompileImplementation, EvmPrecompileInputPolicy, EvmPrecompileKind, EvmPrecompilePlan,
    EvmPrecompileRegistry, execute_identity, execute_ripemd160, execute_sha256,
};
pub use precompile_charge::EvmPrecompileGasCharge;
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

#[cfg(test)]
#[path = "ecrecover_tests.rs"]
mod ecrecover_tests;

#[cfg(test)]
#[path = "modexp_tests.rs"]
mod modexp_tests;

#[cfg(test)]
#[path = "bn254_tests.rs"]
mod bn254_tests;

#[cfg(test)]
#[path = "bn254_pairing_tests.rs"]
mod bn254_pairing_tests;

#[cfg(test)]
#[path = "bn254_tower_tests.rs"]
mod bn254_tower_tests;
