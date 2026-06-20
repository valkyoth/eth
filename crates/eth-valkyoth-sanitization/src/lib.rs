#![no_std]
#![forbid(unsafe_code)]
//! Optional sanitization bridge for secret-bearing Ethereum data.
//!
//! This crate is not part of the default `eth` dependency graph. Use it when an
//! application explicitly wants the `sanitization` crate's best-effort secret
//! clearing APIs under the `eth-valkyoth-*` namespace.

pub use sanitization::{SecretBytes, SecureSanitize, sanitize_bytes, sanitize_bytes_best_effort};

#[cfg(feature = "derive")]
pub use eth_valkyoth_derive::{SecureSanitize, SecureSanitizeOnDrop};

/// Secret byte storage for 20-byte Ethereum-adjacent values.
pub type SecretBytes20 = SecretBytes<20>;

/// Secret byte storage for 32-byte Ethereum scalars, seeds, and keys.
pub type SecretBytes32 = SecretBytes<32>;

/// Secret byte storage for 64-byte secret material.
pub type SecretBytes64 = SecretBytes<64>;

/// Secret byte storage for a secp256k1 private key.
pub type SecretPrivateKey = SecretBytes32;

/// Sanitizes a fixed-size byte array in place.
pub fn sanitize_fixed<const N: usize>(bytes: &mut [u8; N]) {
    sanitize_bytes(bytes);
}
