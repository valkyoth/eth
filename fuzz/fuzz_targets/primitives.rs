#![no_main]

use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::{
    Address, B256, BlockNumber, ChainId, Gas, Nonce, UnixTimestamp, Wei,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = DecodeLimits::TEST_FIXTURE;

    let _ = ChainId::try_from_rlp(data, limits);
    let _ = BlockNumber::try_from_rlp(data, limits);
    let _ = Gas::try_from_rlp(data, limits);
    let _ = Nonce::try_from_rlp(data, limits);
    let _ = UnixTimestamp::try_from_rlp(data, limits);
    let _ = Wei::try_from_rlp(data, limits);
    let _ = Address::try_from_rlp(data, limits);
    let _ = B256::try_from_rlp(data, limits);

    let _ = ChainId::try_from_canonical_be_slice(data);
    let _ = BlockNumber::try_from_canonical_be_slice(data);
    let _ = Gas::try_from_canonical_be_slice(data);
    let _ = Nonce::try_from_canonical_be_slice(data);
    let _ = UnixTimestamp::try_from_canonical_be_slice(data);
    let _ = Wei::try_from_canonical_be_slice(data);
});
