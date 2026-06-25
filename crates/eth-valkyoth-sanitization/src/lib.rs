#![no_std]
#![forbid(unsafe_code)]
//! Optional sanitization bridge for secret-bearing Ethereum data.
//!
//! This crate is not part of the default `eth` dependency graph. Use it when an
//! application explicitly wants the `sanitization` crate's best-effort secret
//! clearing APIs under the `eth-valkyoth-*` namespace.
//!
//! Deployment checklist for private-key or seed material:
//!
//! - enable `memory-lock` to reduce swap exposure;
//! - enable `multi-pass-clear` when policy requires multiple overwrite passes;
//! - enable `cache-flush` where supported by the target threat model;
//! - enable `register-scrub` where supported by the target toolchain;
//! - keep crash dumps, logs, serde, and copies outside this crate's boundary.

#[cfg(all(
    feature = "hardened-only",
    not(all(
        feature = "memory-lock",
        feature = "multi-pass-clear",
        feature = "cache-flush",
        feature = "register-scrub"
    ))
))]
compile_error!(
    "eth-valkyoth-sanitization: hardened-only requires memory-lock, \
     multi-pass-clear, cache-flush, and register-scrub"
);

pub use sanitization::{SecretBytes, SecureSanitize, sanitize_bytes};

#[cfg(feature = "derive")]
pub use eth_valkyoth_derive::{SecureSanitize, SecureSanitizeOnDrop};

/// Whether the memory-clearing bridge was built with the hardened feature set.
#[cfg(all(
    feature = "memory-lock",
    feature = "multi-pass-clear",
    feature = "cache-flush",
    feature = "register-scrub"
))]
pub const HARDENED_MODE: bool = true;

/// Whether the memory-clearing bridge was built with the hardened feature set.
#[cfg(not(all(
    feature = "memory-lock",
    feature = "multi-pass-clear",
    feature = "cache-flush",
    feature = "register-scrub"
)))]
pub const HARDENED_MODE: bool = false;

/// Best-effort clearing APIs.
///
/// These helpers make a best-effort attempt to clear the supplied storage, but
/// cannot guarantee the compiler has not copied or moved the data earlier.
/// Prefer [`sanitize_bytes`] when the stronger API applies.
pub mod best_effort {
    pub use sanitization::sanitize_bytes_best_effort;
}

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
