#![no_main]

use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_protocol::{HeaderFieldSet, decode_block_header};
use libfuzzer_sys::fuzz_target;

const HEADER_FIELD_SETS: [HeaderFieldSet; 5] = [
    HeaderFieldSet::Legacy,
    HeaderFieldSet::London,
    HeaderFieldSet::Shanghai,
    HeaderFieldSet::Cancun,
    HeaderFieldSet::Prague,
];

fuzz_target!(|data: &[u8]| {
    drive_header_decode(data, DecodeLimits::TEST_FIXTURE);
    drive_header_decode(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_header_decode(data: &[u8], limits: DecodeLimits) {
    for field_set in HEADER_FIELD_SETS {
        let Ok(header) = decode_block_header(data, field_set, limits) else {
            continue;
        };
        let _ = header.encoded_rlp().len();
    }
}
