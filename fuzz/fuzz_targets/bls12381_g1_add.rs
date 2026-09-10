#![no_main]

#[path = "../../crates/eth-valkyoth-evm-core/tests/support/bls12_g1_oracle.rs"]
mod oracle;

use eth_valkyoth_evm_core::{
    EvmBls12G1Add, EvmCoreError, EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind,
    EvmPrecompileRegistry, EvmPrecompileStatus,
};
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;

fn check(input: &[u8], gas: u64, capacity: usize) {
    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)
        .expect("fork")
        .descriptor(EvmPrecompileKind::Bls12G1Add)
        .expect("kind");
    let mut meter = EvmGasMeter::try_new(EvmGas::new(gas.max(1))).expect("bounded meter");
    if gas == 0 {
        meter.charge(EvmGas::new(1)).expect("exhaust child");
    }
    let before = meter.used();
    let mut output = [0xa5; 129];
    match descriptor.quote::<EvmBls12G1Add>(input) {
        Err(error) => {
            assert_ne!(input.len(), 256);
            assert_eq!(error, EvmCoreError::PrecompileInvalidInputLength);
        }
        Ok(quote) => {
            assert_eq!(quote.gas_cost(), EvmGas::new(375));
            let result =
                quote.authorize_and_execute_bls12_g1_add(&mut meter, &mut output[..capacity]);
            if capacity < 128 {
                assert_eq!(result, Err(EvmCoreError::PrecompileOutputTooSmall));
            } else if gas < 375 {
                assert_eq!(result, Err(EvmCoreError::OutOfGas));
            } else {
                let result = result.expect("admitted");
                let (left, right) = input.split_at(128);
                if let (Ok(a), Ok(b)) = (oracle::decode(left), oracle::decode(right)) {
                    let expected =
                        oracle::encode(&oracle::add(&a, &b).expect("field")).expect("point");
                    assert_eq!(result.status(), EvmPrecompileStatus::Success);
                    assert_eq!(result.output_len(), 128);
                    assert_eq!(result.gas_consumed(), EvmGas::new(375));
                    assert_eq!(&output[..128], &expected);
                    assert_eq!(output[128], 0xa5);
                    assert_eq!(meter.used(), EvmGas::new(375));
                } else {
                    assert_eq!(result.status(), EvmPrecompileStatus::CallFailure);
                    assert!(result.requires_rollback());
                    assert_eq!(result.output_len(), 0);
                    assert_eq!(result.gas_consumed(), EvmGas::new(gas));
                    assert_eq!(output, [0xa5; 129]);
                    assert_eq!(meter.used(), meter.limit());
                }
                return;
            }
        }
    }
    assert_eq!(output, [0xa5; 129]);
    assert_eq!(meter.used(), before);
}

fuzz_target!(|data: &[u8]| {
    let gas = u64::from(data.first().copied().unwrap_or(0)).saturating_mul(4);
    let capacity = usize::from(data.last().copied().unwrap_or(0)).min(129);
    check(data, gas, capacity);
    check(data, 375, 129);
    let mut seed = [0; 48];
    for (out, byte) in seed.iter_mut().zip(data) {
        *out = *byte;
    }
    let left = oracle::from_x(BigUint::from_bytes_be(&seed)).expect("field");
    let right = oracle::generator().expect("generator");
    let mut valid = [0; 256];
    valid[..128].copy_from_slice(&oracle::encode(&left).expect("point"));
    valid[128..].copy_from_slice(&oracle::encode(&right).expect("point"));
    check(&valid, gas, capacity);
    check(&valid, 375, 129);
});
