#![no_main]

use eth_valkyoth_evm_core::{
    EvmAccessAttempt, EvmAccessTracker, EvmAddress, EvmEmbeddedAccessTracker,
    EvmNodeAccessTracker, EvmWord,
};
use libfuzzer_sys::fuzz_target;

const CAPACITY: usize = 64;
const OPERATION_BYTES: usize = 53;

fuzz_target!(|data: &[u8]| {
    let Ok(mut embedded) = EvmEmbeddedAccessTracker::<CAPACITY, CAPACITY>::try_new() else {
        return;
    };
    let Ok(mut node) = EvmNodeAccessTracker::try_new(CAPACITY, CAPACITY) else {
        return;
    };
    if embedded.begin_attempt() != node.begin_attempt() {
        return;
    }

    for operation in data.chunks_exact(OPERATION_BYTES) {
        let Some((&selector, fields)) = operation.split_first() else {
            continue;
        };
        let Some((address_bytes, slot_bytes)) = fields.split_at_checked(EvmAddress::LEN) else {
            continue;
        };
        let Ok(address) = <[u8; EvmAddress::LEN]>::try_from(address_bytes) else {
            continue;
        };
        let Ok(slot) = <[u8; EvmWord::LEN]>::try_from(slot_bytes) else {
            continue;
        };
        match selector % 6 {
            0 => assert_eq!(
                embedded.warm_address(EvmAddress::from_bytes(address)),
                node.warm_address(EvmAddress::from_bytes(address))
            ),
            1 => assert_eq!(
                embedded.warm_storage(
                    EvmAddress::from_bytes(address),
                    EvmWord::from_be_bytes(slot),
                ),
                node.warm_storage(
                    EvmAddress::from_bytes(address),
                    EvmWord::from_be_bytes(slot),
                )
            ),
            2 => finish_and_restart(&mut embedded, &mut node, EvmAccessAttempt::Commit),
            3 => finish_and_restart(&mut embedded, &mut node, EvmAccessAttempt::Rollback),
            4 => nested_scope(
                &mut embedded,
                &mut node,
                EvmAddress::from_bytes(address),
                EvmWord::from_be_bytes(slot),
                EvmAccessAttempt::Commit,
            ),
            _ => nested_scope(
                &mut embedded,
                &mut node,
                EvmAddress::from_bytes(address),
                EvmWord::from_be_bytes(slot),
                EvmAccessAttempt::Rollback,
            ),
        }
        assert_eq!(embedded.address_len(), node.address_len());
        assert_eq!(embedded.storage_len(), node.storage_len());
    }
});

fn nested_scope(
    embedded: &mut EvmEmbeddedAccessTracker<CAPACITY, CAPACITY>,
    node: &mut EvmNodeAccessTracker,
    address: EvmAddress,
    slot: EvmWord,
    outcome: EvmAccessAttempt,
) {
    let (Ok(embedded_checkpoint), Ok(node_checkpoint)) =
        (embedded.checkpoint(), node.checkpoint())
    else {
        return;
    };
    assert_eq!(
        embedded.warm_storage(address, slot),
        node.warm_storage(address, slot)
    );
    match outcome {
        EvmAccessAttempt::Commit => assert_eq!(
            embedded.commit(embedded_checkpoint),
            node.commit(node_checkpoint)
        ),
        EvmAccessAttempt::Rollback => assert_eq!(
            embedded.revert(embedded_checkpoint),
            node.revert(node_checkpoint)
        ),
    }
}

fn finish_and_restart(
    embedded: &mut EvmEmbeddedAccessTracker<CAPACITY, CAPACITY>,
    node: &mut EvmNodeAccessTracker,
    outcome: EvmAccessAttempt,
) {
    assert_eq!(
        embedded.finish_attempt(outcome),
        node.finish_attempt(outcome)
    );
    assert_eq!(embedded.begin_attempt(), node.begin_attempt());
}
