#![no_main]

use eth_valkyoth_codec::{
    DecodeLimits, MAX_RLP_LIST_TRAVERSAL_DEPTH, RlpItem, RlpList, decode_rlp_integer,
    decode_rlp_integer_partial, decode_rlp_list, decode_rlp_list_partial, decode_rlp_scalar,
    decode_rlp_scalar_partial, decode_rlp_u64, decode_rlp_u128, decode_rlp_u256_bytes,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_rlp(data, DecodeLimits::TEST_FIXTURE);
    drive_rlp(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_rlp(data: &[u8], limits: DecodeLimits) {
    let _ = decode_rlp_scalar(data, limits);
    let _ = decode_rlp_u64(data, limits);
    let _ = decode_rlp_u128(data, limits);
    let _ = decode_rlp_u256_bytes(data, limits);

    if let Ok(integer) = decode_rlp_integer(data, limits) {
        let _ = integer.to_u64();
        let _ = integer.to_u128();
        let _ = integer.to_be_bytes32();
    }

    if let Ok(list) = decode_rlp_list(data, limits) {
        drive_items(list, MAX_RLP_LIST_TRAVERSAL_DEPTH);
    }

    let mut accumulator = limits.accumulator();
    let _ = decode_rlp_scalar_partial(data, &mut accumulator);

    let mut accumulator = limits.accumulator();
    let _ = decode_rlp_integer_partial(data, &mut accumulator);

    let mut accumulator = limits.accumulator();
    if let Ok(list) = decode_rlp_list_partial(data, &mut accumulator) {
        drive_items(list, MAX_RLP_LIST_TRAVERSAL_DEPTH);
    }
}

fn drive_items(list: RlpList<'_>, depth: usize) {
    if depth == 0 {
        return;
    }

    let mut items = list.items();
    while let Some(item) = items.next() {
        let Ok(item) = item else {
            assert!(items.next().is_none());
            return;
        };
        let _ = item.encoded_len();
        if let RlpItem::List(child) = item {
            drive_items(child, depth - 1);
        }
    }
}
