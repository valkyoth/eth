#![no_main]

use eth_valkyoth_codec::{
    DecodeLimits, decode_rlp_integer, decode_rlp_integer_partial, decode_rlp_u64, decode_rlp_u128,
    decode_rlp_u256_bytes, rlp_integer_payload_to_u64, rlp_integer_payload_to_u128,
    rlp_integer_payload_to_u256_bytes, validate_rlp_integer_payload,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_integer(data, DecodeLimits::TEST_FIXTURE);
    // Fuzzing intentionally exercises the deployment template directly.
    // Production callers must use a reviewed policy or call
    // validate_deployment_policy() before decoding externally reachable input.
    drive_integer(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
    drive_integer(
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

fn drive_integer(data: &[u8], limits: DecodeLimits) {
    let _ = validate_rlp_integer_payload(data);
    let _ = rlp_integer_payload_to_u64(data);
    let _ = rlp_integer_payload_to_u128(data);
    let _ = rlp_integer_payload_to_u256_bytes(data);

    if let Ok(integer) = decode_rlp_integer(data, limits) {
        let _ = integer.to_u64();
        let _ = integer.to_u128();
        let _ = integer.to_be_bytes32();
    }

    let _ = decode_rlp_u64(data, limits);
    let _ = decode_rlp_u128(data, limits);
    let _ = decode_rlp_u256_bytes(data, limits);

    let mut accumulator = limits.accumulator();
    if let Ok(integer) = decode_rlp_integer_partial(data, &mut accumulator) {
        let _ = integer.to_u64();
        let _ = integer.to_u128();
        let _ = integer.to_be_bytes32();
    }
}
