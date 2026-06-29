#![no_std]
#![forbid(unsafe_code)]
//! Bounded decoding policy for untrusted Ethereum wire inputs.

#[cfg(feature = "std")]
extern crate std;

mod budget;
mod error;
mod exact;
mod rlp;

pub use budget::{DecodeAccumulator, DecodeLimits};
pub use error::{DecodeError, DecodeErrorCategory, ResourceError};
pub use exact::{
    checked_len_add, checked_range_end, require_exact_consumption, require_range_in_bounds,
};
pub use rlp::{
    MAX_RLP_LIST_TRAVERSAL_DEPTH, MAX_RLP_U256_BYTES, RlpInteger, RlpItem, RlpList, RlpListForm,
    RlpListItems, RlpScalar, RlpScalarForm, decode_rlp_integer, decode_rlp_integer_partial,
    decode_rlp_list, decode_rlp_list_partial, decode_rlp_scalar, decode_rlp_scalar_partial,
    decode_rlp_u64, decode_rlp_u128, decode_rlp_u256_bytes,
};
