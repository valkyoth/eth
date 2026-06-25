#![no_main]

use eth_valkyoth_codec::{DecodeLimits, decode_rlp_scalar, decode_rlp_scalar_prefix};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_scalar(data, DecodeLimits::TEST_FIXTURE);
    drive_scalar(data, DecodeLimits::DEPLOYMENT_TEMPLATE);
    drive_scalar(
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

fn drive_scalar(data: &[u8], limits: DecodeLimits) {
    let _ = decode_rlp_scalar(data, limits);

    let mut accumulator = limits.accumulator();
    let _ = decode_rlp_scalar_prefix(data, &mut accumulator);
}
