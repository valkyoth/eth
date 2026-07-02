#![no_main]

use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_protocol::{ReceiptEnvelope, decode_receipt, decode_receipt_envelope};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_receipt_decode(data, DecodeLimits::TEST_FIXTURE);
    drive_receipt_decode(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_receipt_decode(data: &[u8], limits: DecodeLimits) {
    if let Ok(envelope) = decode_receipt_envelope(data, limits) {
        match envelope {
            ReceiptEnvelope::Legacy(list) => {
                let _ = list.item_count();
            }
            ReceiptEnvelope::Typed(typed) => {
                let _ = typed.receipt_type.get();
                let _ = typed.payload.len();
            }
        }
    }

    let Ok(receipt) = decode_receipt(data, limits) else {
        return;
    };
    let _ = receipt.encoded_payload().len();
    let _ = receipt.logs.len();
    for log in receipt.logs.entries() {
        let Ok(log) = log else {
            return;
        };
        let _ = log.data.len();
        for topic in log.topics.topics() {
            let _ = topic;
        }
    }
}
