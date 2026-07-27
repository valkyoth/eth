#![no_main]

use eth_valkyoth_codec::DecodeSessionPolicy;
use eth_valkyoth_evm::{BlockExecutionContext, ClassifiedEnvelope, ExecutionEnvironment};
use eth_valkyoth_primitives::{
    Address, B256, BlockNumber, ChainId, Gas, UnixTimestamp, Wei,
};
use eth_valkyoth_protocol::{ForkActivation, ForkSpec, Hardfork, ValidationContext};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, raw)) = data.split_first() else {
        return;
    };
    let hardfork = match selector % 8 {
        0 => Hardfork::Frontier,
        1 => Hardfork::Homestead,
        2 => Hardfork::Byzantium,
        3 => Hardfork::London,
        4 => Hardfork::Shanghai,
        5 => Hardfork::Cancun,
        6 => Hardfork::Prague,
        _ => Hardfork::Amsterdam,
    };
    let Some(environment) = environment(hardfork) else {
        return;
    };
    let Ok(classified) = ClassifiedEnvelope::decode(raw, DecodeSessionPolicy::TEST_FIXTURE) else {
        return;
    };
    let Ok(canonical) = classified.try_into_canonical() else {
        return;
    };
    let Ok(validated) = canonical.try_into_fork_validated(environment) else {
        return;
    };
    let ready = validated.into_execution_ready();
    assert_eq!(ready.raw(), raw);
    assert_eq!(ready.environment(), environment);
});

fn environment(hardfork: Hardfork) -> Option<ExecutionEnvironment> {
    let chain_id = ChainId::new(1);
    let block_number = BlockNumber::new(1);
    let timestamp = UnixTimestamp::new(1);
    let context = ValidationContext {
        fork: ForkSpec {
            chain_id,
            hardfork,
            activation: ForkActivation::BlockAndTimestamp {
                activation_block: block_number,
                activation_timestamp: timestamp,
            },
        },
        block_number,
        timestamp,
    };
    let block = BlockExecutionContext {
        chain_id,
        block_number,
        timestamp,
        beneficiary: Address::from_bytes(core::array::from_fn(|_| u8::default())),
        gas_limit: Gas::new(30_000_000),
        base_fee_per_gas: Wei::ZERO,
        prev_randao: B256::from_bytes(core::array::from_fn(|_| u8::default())),
    };
    ExecutionEnvironment::try_new(context, block).ok()
}
