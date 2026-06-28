#![no_main]

use eth_valkyoth_codec::{
    DecodeLimits, RlpItem, RlpList, decode_rlp_list, decode_rlp_list_partial,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_list(data, DecodeLimits::TEST_FIXTURE);
    drive_list(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
    drive_list(
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

fn drive_list(data: &[u8], limits: DecodeLimits) {
    if let Ok(list) = decode_rlp_list(data, limits) {
        drive_items(list);
    }

    let mut accumulator = limits.accumulator();
    if let Ok(list) = decode_rlp_list_partial(data, &mut accumulator) {
        drive_items(list);
    }
}

fn drive_items(list: RlpList<'_>) {
    for item in list.items() {
        let Ok(item) = item else {
            continue;
        };
        let _ = item.encoded_len();
        if let RlpItem::List(child) = item {
            for nested in child.items() {
                let _ = nested.map(RlpItem::encoded_len);
            }
        }
    }
}
