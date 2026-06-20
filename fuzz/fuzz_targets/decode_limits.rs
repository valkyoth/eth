#![no_main]

use eth_valkyoth_codec::DecodeLimits;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = DecodeLimits::TEST_FIXTURE.check_input_len(data.len());
});
