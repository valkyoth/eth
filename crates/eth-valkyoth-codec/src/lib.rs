#![no_std]
#![forbid(unsafe_code)]
//! Bounded decoding policy for untrusted Ethereum wire inputs.

#[cfg(feature = "std")]
extern crate std;

mod budget;
mod error;
mod exact;

pub use budget::{DecodeAccumulator, DecodeLimits};
pub use error::{DecodeError, DecodeErrorCategory, ResourceError};
pub use exact::{
    checked_len_add, checked_range_end, require_exact_consumption, require_range_in_bounds,
};
