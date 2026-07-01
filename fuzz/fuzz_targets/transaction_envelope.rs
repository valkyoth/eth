#![no_main]

use eth_valkyoth_codec::{DecodeLimits, RlpItem};
use eth_valkyoth_protocol::{TransactionEnvelope, decode_transaction_envelope};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_transaction_envelope(data, DecodeLimits::TEST_FIXTURE);
    drive_transaction_envelope(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_transaction_envelope(data: &[u8], limits: DecodeLimits) {
    let Ok(envelope) = decode_transaction_envelope(data, limits) else {
        return;
    };

    match envelope {
        TransactionEnvelope::Typed(typed) => {
            let _ = u8::from(typed.transaction_type);
            let _ = typed.payload.len();
        }
        TransactionEnvelope::Legacy(list) => {
            let _ = list.item_count();
            let mut items = list.items();
            while let Some(item) = items.next() {
                let Ok(item) = item else {
                    return;
                };
                if let RlpItem::Scalar(scalar) = item {
                    let _ = scalar.payload().len();
                }
            }
        }
    }
}
