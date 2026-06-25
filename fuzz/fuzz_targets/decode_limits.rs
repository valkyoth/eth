#![no_main]

use core::mem::size_of;

use eth_valkyoth_codec::DecodeLimits;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_limits(data, DecodeLimits::TEST_FIXTURE);
    drive_limits(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
    drive_limits(
        data,
        DecodeLimits {
            max_input_bytes: usize::MAX,
            max_list_items: usize::MAX,
            max_nesting_depth: usize::MAX,
            max_total_allocation: usize::MAX,
            max_proof_nodes: usize::MAX,
            max_total_items: usize::MAX,
        },
    );
});

fn drive_limits(data: &[u8], limits: DecodeLimits) {
    let mut accumulator = limits.accumulator();

    let _ = limits.check_input_len(data.len());
    let _ = accumulator.check_input_len(data.len());

    for chunk in data.chunks(size_of::<usize>()) {
        let value = usize_from_chunk(chunk);

        let _ = limits.check_list_count(value);
        let _ = limits.check_nesting_depth(value);
        let _ = limits.check_single_allocation_limit(value);
        let _ = limits.check_proof_node_count(value);
        let _ = limits.check_item_count(value);
        let _ = accumulator.check_list_count(value);
        let _ = accumulator.check_nesting_depth(value);
        let _ = accumulator.check_allocation(value);
        let _ = accumulator.account_proof_nodes(value);
        let _ = accumulator.account_items(value);
    }
}

fn usize_from_chunk(chunk: &[u8]) -> usize {
    let mut bytes = [0_u8; size_of::<usize>()];
    bytes[..chunk.len()].copy_from_slice(chunk);
    usize::from_le_bytes(bytes)
}
