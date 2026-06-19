#![no_std]
#![forbid(unsafe_code)]
//! Future signer isolation boundary for `eth`.

use eth_primitives::Address;

/// Minimal signer identity boundary. Signing APIs are admitted later.
pub trait SignerIdentity {
    /// Returns the signer address without exposing key material.
    fn address(&self) -> Address;
}
