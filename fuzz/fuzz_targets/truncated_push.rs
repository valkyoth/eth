#![no_main]

use eth_valkyoth_evm_core::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EvmCoreError, EvmExecution, EvmFork,
    EvmOpcode, EvmWord, ExecutionLimits, ExecutionStatus,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let selector = data.first().copied().unwrap_or(0);
    let width = usize::from(selector % 32) + 1;
    let available = data.len().saturating_sub(1).min(width.saturating_sub(1));
    let payload = data.get(1..=available).unwrap_or(&[]);
    let opcode = EvmOpcode::new(EvmOpcode::PUSH1.byte() + u8::try_from(width - 1).unwrap_or(0));
    let Ok(limits) = ExecutionLimits::try_new(
        EVM_DEFAULT_STEP_LIMIT,
        EVM_DEFAULT_GAS_LIMIT,
        EvmFork::CANCUN,
    ) else {
        return;
    };

    let mut code = Vec::with_capacity(1 + payload.len());
    code.push(opcode.byte());
    code.extend_from_slice(payload);
    let mut memory = [];
    let Ok(mut execution) = EvmExecution::<1>::try_new(&mut memory) else {
        return;
    };
    let Ok(report) = execution.run(&code, limits) else {
        panic!("truncated PUSH must execute");
    };
    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(report.pc.get(), width + 1);

    let mut padded = [0u8; EvmWord::LEN];
    for (source, target) in payload.iter().zip(padded.iter_mut()) {
        *target = *source;
    }
    let Some(padded_width) = padded.get(..width) else {
        return;
    };
    let Ok(expected) = EvmWord::from_be_slice(padded_width) else {
        return;
    };
    assert_eq!(execution.stack().peek(0), Ok(expected));

    if available == 0 {
        return;
    }
    let jump_target = 4u8;
    let mut jump_code = Vec::with_capacity(4 + available);
    jump_code.extend_from_slice(&[
        EvmOpcode::PUSH1.byte(),
        jump_target,
        EvmOpcode::JUMP.byte(),
        opcode.byte(),
        EvmOpcode::JUMPDEST.byte(),
    ]);
    jump_code.extend(payload.iter().skip(1));
    let mut jump_memory = [];
    let Ok(mut jump_execution) = EvmExecution::<1>::try_new(&mut jump_memory) else {
        return;
    };
    assert_eq!(
        jump_execution.run(&jump_code, limits),
        Err(EvmCoreError::InvalidJumpDestination)
    );
});
