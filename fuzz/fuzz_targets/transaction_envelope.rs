#![no_main]

use eth_valkyoth_codec::{DecodeLimits, MAX_RLP_LIST_TRAVERSAL_DEPTH, RlpItem, RlpList};
use eth_valkyoth_protocol::{
    TransactionEnvelope, decode_access_list_transaction, decode_blob_transaction,
    decode_dynamic_fee_transaction, decode_legacy_transaction, decode_transaction_envelope,
    encode_access_list_transaction, encode_blob_transaction, encode_dynamic_fee_transaction,
    encode_legacy_transaction,
};
use libfuzzer_sys::fuzz_target;

const ENCODE_BUFFER_BYTES: usize = 4096;

fuzz_target!(|data: &[u8]| {
    drive_transaction_envelope(data, DecodeLimits::TEST_FIXTURE);
    drive_transaction_envelope(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_transaction_envelope(data: &[u8], limits: DecodeLimits) {
    let mut output = [0_u8; ENCODE_BUFFER_BYTES];
    if let Ok(transaction) = decode_legacy_transaction(data, limits) {
        let _ = encode_legacy_transaction(&transaction, &mut output);
    }
    if let Ok(transaction) = decode_access_list_transaction(data, limits) {
        let _ = encode_access_list_transaction(&transaction, &mut output);
    }
    if let Ok(transaction) = decode_dynamic_fee_transaction(data, limits) {
        let _ = encode_dynamic_fee_transaction(&transaction, &mut output);
    }
    if let Ok(transaction) = decode_blob_transaction(data, limits) {
        let _ = encode_blob_transaction(&transaction, &mut output);
    }

    let Ok(envelope) = decode_transaction_envelope(data, limits) else {
        return;
    };

    match envelope {
        TransactionEnvelope::Typed(typed) => {
            let _ = u8::from(typed.transaction_type);
            let _ = typed.payload.len();
        }
        TransactionEnvelope::Legacy(list) => {
            drive_legacy_items(list, MAX_RLP_LIST_TRAVERSAL_DEPTH);
        }
    }
}

fn drive_legacy_items(list: RlpList<'_>, depth: usize) {
    let _ = list.item_count();
    if depth == 0 {
        return;
    }

    let mut items = list.items();
    while let Some(item) = items.next() {
        let Ok(item) = item else {
            return;
        };
        match item {
            RlpItem::Scalar(scalar) => {
                let _ = scalar.payload().len();
            }
            RlpItem::List(child) => {
                drive_legacy_items(child, depth - 1);
            }
        }
    }
}
