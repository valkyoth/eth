#![no_main]

use eth_valkyoth_primitives::{BlockNumber, ChainId, Gas, Nonce, UnixTimestamp, Wei};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = ChainId::try_from_canonical_be_slice(data);
    let _ = BlockNumber::try_from_canonical_be_slice(data);
    let _ = Gas::try_from_canonical_be_slice(data);
    let _ = Nonce::try_from_canonical_be_slice(data);
    let _ = UnixTimestamp::try_from_canonical_be_slice(data);
    let _ = Wei::try_from_canonical_be_slice(data);
});
