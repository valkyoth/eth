#![no_std]
#![forbid(unsafe_code)]
//! Bounded decoding policy for untrusted Ethereum wire inputs.

#[cfg(feature = "std")]
extern crate std;

mod budget;
mod error;
mod exact;
mod rlp;
mod session;

pub use budget::{DecodeAccumulator, DecodeLimits};
pub use error::{DecodeError, DecodeErrorCategory, ResourceError};
pub use exact::{
    checked_len_add, checked_range_end, require_exact_consumption, require_range_in_bounds,
};
pub use rlp::{
    MAX_RLP_LIST_TRAVERSAL_DEPTH, MAX_RLP_U256_BYTES, RlpDecode, RlpDeriveError, RlpEncode,
    RlpInteger, RlpItem, RlpList, RlpListForm, RlpListItems, RlpListSessionItems, RlpScalar,
    RlpScalarForm, checked_encoded_len_add, decode_rlp_integer, decode_rlp_integer_partial,
    decode_rlp_list, decode_rlp_list_in_session, decode_rlp_list_partial,
    decode_rlp_list_partial_in_session, decode_rlp_scalar, decode_rlp_scalar_in_session,
    decode_rlp_scalar_partial, decode_rlp_scalar_partial_in_session, decode_rlp_u64,
    decode_rlp_u128, decode_rlp_u256_bytes, encode_decoded_integer, encode_decoded_item,
    encode_decoded_list, encode_decoded_scalar, encode_rlp_integer, encode_rlp_list_header,
    encode_rlp_list_payload, encode_rlp_scalar, encoded_rlp_integer_len,
    encoded_rlp_list_header_len, encoded_rlp_list_len, encoded_rlp_scalar_len,
    rlp_integer_payload_to_u64, rlp_integer_payload_to_u128, rlp_integer_payload_to_u256_bytes,
    validate_rlp_integer_payload,
};
pub use session::{DecodeSession, DecodeSessionCharges, DecodeSessionPolicy};
