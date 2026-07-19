use std::vec::Vec;

use crate::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EvmCoreError, EvmExecution, EvmFork, EvmGas,
    EvmOpcode, EvmWord, ExecutionLimits, ExecutionStatus, jumpdest::JumpdestMap,
};

fn limits() -> Result<ExecutionLimits, EvmCoreError> {
    ExecutionLimits::try_new(
        EVM_DEFAULT_STEP_LIMIT,
        EVM_DEFAULT_GAS_LIMIT,
        EvmFork::CANCUN,
    )
}

fn push_opcode(width: usize) -> Result<u8, EvmCoreError> {
    let offset = u8::try_from(width).map_err(|_| EvmCoreError::UnsupportedOpcode)?;
    EvmOpcode::PUSH1
        .byte()
        .checked_add(offset.saturating_sub(1))
        .ok_or(EvmCoreError::UnsupportedOpcode)
}

fn reference_word(width: usize, payload: &[u8]) -> Result<EvmWord, EvmCoreError> {
    let mut padded = [0u8; EvmWord::LEN];
    for (source, target) in payload.iter().zip(padded.iter_mut()) {
        *target = *source;
    }
    EvmWord::from_be_slice(padded.get(..width).ok_or(EvmCoreError::WordInputTooLarge)?)
}

#[test]
fn every_truncated_push_right_pads_missing_bytes_with_zero() -> Result<(), EvmCoreError> {
    for width in 1..=EvmWord::LEN {
        for available in 0..width {
            let mut code = Vec::with_capacity(1 + available);
            code.push(push_opcode(width)?);
            for byte in 0..available {
                code.push(u8::try_from(byte + 1).map_err(|_| EvmCoreError::WordInputTooLarge)?);
            }

            let mut memory = [];
            let mut execution = EvmExecution::<1>::try_new(&mut memory)?;
            let report = execution.run(&code, limits()?)?;

            assert_eq!(report.status, ExecutionStatus::Stopped);
            assert_eq!(report.steps, 1);
            assert_eq!(report.pc.get(), width + 1);
            assert_eq!(report.gas_used, EvmGas::new(3));
            assert_eq!(
                execution.stack().peek(0)?,
                reference_word(
                    width,
                    code.get(1..).ok_or(EvmCoreError::ProgramCounterOverflow)?,
                )?
            );
        }
    }
    Ok(())
}

#[test]
fn known_client_truncated_push_vectors_match() -> Result<(), EvmCoreError> {
    let vectors: &[(u8, &[u8], &[u8])] = &[
        (EvmOpcode::PUSH1.byte(), &[], &[0]),
        (0x61, &[1], &[1, 0]),
        (0x62, &[1, 2], &[1, 2, 0]),
        (EvmOpcode::PUSH32.byte(), &[0xab], &[0xab, 0, 0, 0]),
    ];

    for (opcode, payload, expected_prefix) in vectors {
        let width = usize::from(EvmOpcode::new(*opcode).push_width().unwrap_or(0));
        let mut code = Vec::with_capacity(1 + payload.len());
        code.push(*opcode);
        code.extend_from_slice(payload);
        let mut memory = [];
        let mut execution = EvmExecution::<1>::try_new(&mut memory)?;
        let _ = execution.run(&code, limits()?)?;
        let expected = reference_word(width, expected_prefix)?;
        assert_eq!(execution.stack().peek(0)?, expected);
    }
    Ok(())
}

#[test]
fn legacy_push_error_is_compatible_but_never_returned() -> Result<(), EvmCoreError> {
    assert_eq!(
        EvmCoreError::PushImmediateOutOfBounds.code(),
        "push_immediate_out_of_bounds"
    );

    let mut memory = [];
    let mut execution = EvmExecution::<1>::try_new(&mut memory)?;
    let report = execution.run(&[0x61, 0x01], limits()?)?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(execution.stack().peek(0)?, EvmWord::from_be_slice(&[1, 0])?);
    Ok(())
}

#[test]
fn jumpdest_analysis_skips_all_truncated_push_payload_bytes() -> Result<(), EvmCoreError> {
    for width in 1..=EvmWord::LEN {
        for available in 0..width {
            let mut code = Vec::with_capacity(1 + available);
            code.push(push_opcode(width)?);
            code.resize(1 + available, EvmOpcode::JUMPDEST.byte());

            let map = JumpdestMap::try_new(&code)?;
            for offset in 1..code.len() {
                assert!(!map.contains(offset));
            }
        }
    }
    Ok(())
}

#[test]
fn execution_and_analysis_reject_jump_into_truncated_push_data() -> Result<(), EvmCoreError> {
    for width in 2..=EvmWord::LEN {
        let target = 4u8;
        let code = [
            EvmOpcode::PUSH1.byte(),
            target,
            EvmOpcode::JUMP.byte(),
            push_opcode(width)?,
            EvmOpcode::JUMPDEST.byte(),
        ];
        let mut memory = [];
        let mut execution = EvmExecution::<1>::try_new(&mut memory)?;

        assert_eq!(
            execution.run(&code, limits()?),
            Err(EvmCoreError::InvalidJumpDestination)
        );
    }
    Ok(())
}
