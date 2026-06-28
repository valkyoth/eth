#![no_main]

use eth_valkyoth_codec::{
    DecodeLimits, RlpItem, RlpList, decode_rlp_list, decode_rlp_list_partial,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_list(data, DecodeLimits::TEST_FIXTURE);
    // Fuzzing intentionally exercises the deployment template directly.
    // Production callers must use a reviewed policy or call
    // validate_deployment_policy() before decoding externally reachable input.
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
        drive_items_recursive(list, 8);
    }

    let mut accumulator = limits.accumulator();
    if let Ok(list) = decode_rlp_list_partial(data, &mut accumulator) {
        drive_items_recursive(list, 8);
    }
}

fn drive_items_recursive(list: RlpList<'_>, depth: usize) {
    if depth == 0 {
        return;
    }

    let mut repeated = list.items();
    let _ = repeated.next();

    let mut items = list.items();
    let hint = items.size_hint();
    assert_eq!(hint.0, hint.1.unwrap_or(hint.0));

    for item in items.by_ref() {
        let Ok(item) = item else {
            assert!(items.next().is_none());
            return;
        };
        let _ = item.encoded_len();
        if let RlpItem::List(child) = item {
            drive_items_recursive(child, depth - 1);
        }
    }
    assert!(items.next().is_none());
}
