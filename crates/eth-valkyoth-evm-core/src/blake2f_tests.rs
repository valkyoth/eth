extern crate std;

use crate::{
    EVM_BLAKE2F_INPUT_BYTES, EVM_BLAKE2F_OUTPUT_BYTES, EvmCoreError, EvmFork, EvmGas, EvmGasMeter,
    EvmPrecompileImplementation, EvmPrecompileKind, EvmPrecompilePlan, EvmPrecompileRegistry,
    blake2f::execute_blake2f,
};
use std::vec::Vec;

const SHORT_BLAKE2F_INPUT_BYTES: usize = EVM_BLAKE2F_INPUT_BYTES - 1;
const LONG_BLAKE2F_INPUT_BYTES: usize = EVM_BLAKE2F_INPUT_BYTES + 1;
const SHORT_BLAKE2F_OUTPUT_BYTES: usize = EVM_BLAKE2F_OUTPUT_BYTES - 1;

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn blake2f_matches_eip152_zero_round_vector() -> Result<(), EvmCoreError> {
    let input = hex_bytes(EIP152_ZERO_ROUND_INPUT);
    let mut output = [0u8; EVM_BLAKE2F_OUTPUT_BYTES];
    assert_eq!(
        execute_blake2f(&input, &mut output)?,
        EVM_BLAKE2F_OUTPUT_BYTES
    );
    assert_eq!(output, hex_array(EIP152_ZERO_ROUND_OUTPUT));
    Ok(())
}

#[test]
fn blake2f_matches_eip152_final_block_vector() -> Result<(), EvmCoreError> {
    let input = hex_bytes(EIP152_TWELVE_ROUND_FINAL_INPUT);
    let descriptor = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    assert_eq!(
        descriptor.implementation,
        EvmPrecompileImplementation::NativeBlake2F
    );
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(12)));

    let mut gas = EvmGasMeter::try_new(EvmGas::new(12))?;
    let mut output = [0u8; EVM_BLAKE2F_OUTPUT_BYTES];
    assert_eq!(
        plan.execute_blake2f(&mut gas, &input, &mut output)?,
        EVM_BLAKE2F_OUTPUT_BYTES
    );
    assert_eq!(gas.used(), EvmGas::new(12));
    assert_eq!(output, hex_array(EIP152_TWELVE_ROUND_FINAL_OUTPUT));
    Ok(())
}

#[test]
fn blake2f_matches_eip152_non_final_and_one_round_vectors() -> Result<(), EvmCoreError> {
    let cases = [
        (
            EIP152_TWELVE_ROUND_NON_FINAL_INPUT,
            EIP152_TWELVE_ROUND_NON_FINAL_OUTPUT,
        ),
        (EIP152_ONE_ROUND_INPUT, EIP152_ONE_ROUND_OUTPUT),
    ];
    for (input_hex, output_hex) in cases {
        let mut output = [0u8; EVM_BLAKE2F_OUTPUT_BYTES];
        assert_eq!(
            execute_blake2f(&hex_bytes(input_hex), &mut output)?,
            EVM_BLAKE2F_OUTPUT_BYTES
        );
        assert_eq!(output, hex_array(output_hex));
    }
    Ok(())
}

#[test]
fn blake2f_rejects_invalid_lengths_and_final_flag() -> Result<(), EvmCoreError> {
    assert_eq!(
        execute_blake2f(&[], &mut [0u8; EVM_BLAKE2F_OUTPUT_BYTES]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        execute_blake2f(
            &[0u8; EVM_BLAKE2F_INPUT_BYTES - 1],
            &mut [0u8; EVM_BLAKE2F_OUTPUT_BYTES]
        ),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        execute_blake2f(
            &[0u8; LONG_BLAKE2F_INPUT_BYTES],
            &mut [0u8; EVM_BLAKE2F_OUTPUT_BYTES]
        ),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );

    let mut invalid_flag = [0u8; EVM_BLAKE2F_INPUT_BYTES];
    if let Some(round_byte) = invalid_flag.get_mut(3) {
        *round_byte = 12;
    }
    if let Some(flag) = invalid_flag.get_mut(SHORT_BLAKE2F_INPUT_BYTES) {
        *flag = 2;
    }
    let descriptor = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &invalid_flag)?;
    let mut gas = EvmGasMeter::try_new(EvmGas::new(12))?;
    let mut output = [7u8; EVM_BLAKE2F_OUTPUT_BYTES];
    assert_eq!(
        plan.execute_blake2f(&mut gas, &invalid_flag, &mut output),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(gas.used(), EvmGas::new(12));
    assert_eq!(output, [7u8; EVM_BLAKE2F_OUTPUT_BYTES]);
    Ok(())
}

#[test]
fn blake2f_checks_output_and_plan_binding() -> Result<(), EvmCoreError> {
    let input = hex_bytes(EIP152_TWELVE_ROUND_FINAL_INPUT);
    let mut short_output = [9u8; SHORT_BLAKE2F_OUTPUT_BYTES];
    assert_eq!(
        execute_blake2f(&input, &mut short_output),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(short_output, [9u8; SHORT_BLAKE2F_OUTPUT_BYTES]);

    let descriptor = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &input)?;
    let mut output = [0u8; EVM_BLAKE2F_OUTPUT_BYTES];
    let mut gas = EvmGasMeter::try_new(EvmGas::new(12))?;
    assert_eq!(
        plan.execute_blake2f(
            &mut gas,
            input.get(..SHORT_BLAKE2F_INPUT_BYTES).unwrap_or(&[]),
            &mut output,
        ),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );

    let identity = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Identity)?;
    let wrong_plan = EvmPrecompilePlan::try_new(identity, b"abc")?;
    assert_eq!(
        wrong_plan.execute_blake2f(&mut gas, b"abc", &mut output),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[test]
fn blake2f_max_rounds_are_planned_but_not_executed() -> Result<(), EvmCoreError> {
    let input = hex_bytes(EIP152_MAX_ROUND_INPUT);
    let descriptor = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(descriptor, &input)?.gas_cost(),
        Some(EvmGas::new(u64::from(u32::MAX)))
    );
    Ok(())
}

const EIP152_ZERO_ROUND_INPUT: &str = "0000000048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001";
const EIP152_ZERO_ROUND_OUTPUT: &str = "08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d282e6ad7f520e511f6c3e2b8c68059b9442be0454267ce079217e1319cde05b";
const EIP152_TWELVE_ROUND_FINAL_INPUT: &str = "0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001";
const EIP152_TWELVE_ROUND_FINAL_OUTPUT: &str = "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923";
const EIP152_TWELVE_ROUND_NON_FINAL_INPUT: &str = "0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000";
const EIP152_TWELVE_ROUND_NON_FINAL_OUTPUT: &str = "75ab69d3190a562c51aef8d88f1c2775876944407270c42c9844252c26d2875298743e7f6d5ea2f2d3e8d226039cd31b4e426ac4f2d3d666a610c2116fde4735";
const EIP152_ONE_ROUND_INPUT: &str = "0000000148c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001";
const EIP152_ONE_ROUND_OUTPUT: &str = "b63a380cb2897d521994a85234ee2c181b5f844d2c624c002677e9703449d2fba551b3a8333bcdf5f2f7e08993d53923de3d64fcc68c034e717b9293fed7a421";
const EIP152_MAX_ROUND_INPUT: &str = "ffffffff48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001";

fn hex_array<const N: usize>(hex: &str) -> [u8; N] {
    let bytes = hex_bytes(hex);
    match bytes.try_into() {
        Ok(array) => array,
        Err(_) => [0u8; N],
    }
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert!(hex.len().is_multiple_of(2));
    hex.as_bytes()
        .chunks_exact(2)
        .filter_map(|pair| {
            let [high, low] = pair else {
                return None;
            };
            Some((hex_nibble(*high) << 4) | hex_nibble(*low))
        })
        .collect()
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte.wrapping_sub(b'0'),
        b'a'..=b'f' => byte.wrapping_sub(b'a').wrapping_add(10),
        b'A'..=b'F' => byte.wrapping_sub(b'A').wrapping_add(10),
        _ => 0,
    }
}
