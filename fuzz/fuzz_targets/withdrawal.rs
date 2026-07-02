#![no_main]

use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_protocol::decode_withdrawals;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_withdrawal_decode(data, DecodeLimits::TEST_FIXTURE);
    drive_withdrawal_decode(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_withdrawal_decode(data: &[u8], limits: DecodeLimits) {
    let Ok(withdrawals) = decode_withdrawals(data, limits) else {
        return;
    };

    let _ = withdrawals.encoded_rlp().len();
    let _ = withdrawals.len();
    for entry in withdrawals.entries() {
        let Ok(entry) = entry else {
            return;
        };
        let _ = entry.index.get();
        let _ = entry.validator_index.get();
        let _ = entry.address.to_bytes();
        let _ = entry.amount.get();
    }
}
