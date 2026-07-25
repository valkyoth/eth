#![no_std]
#![forbid(unsafe_code)]
//! Optional sanitization bridge for secret-bearing Ethereum data.
//!
//! This crate is not part of the default `eth` dependency graph. Use it when an
//! application explicitly wants the `sanitization` crate's optimizer-resistant
//! secret clearing APIs under the `eth-valkyoth-*` namespace.
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

pub use sanitization::{
    DropSafeSanitize, ProtectionReport, ProtectionRequest, SecretBytes, SecureSanitize, wipe,
};

#[cfg(all(
    feature = "memory-lock",
    any(
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        ),
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        all(target_arch = "wasm32", feature = "wasm-compat"),
    )
))]
pub use sanitization::LockedSecretBytes;

#[cfg(feature = "derive")]
pub use eth_valkyoth_derive::{SecureSanitize, SecureSanitizeOnDrop};

/// Whether the legacy hardening feature set was selected at compile time.
///
/// This does not claim that any runtime memory protection succeeded. Inspect
/// the [`ProtectionReport`] returned by protected containers.
#[cfg(all(
    feature = "memory-lock",
    feature = "multi-pass-clear",
    feature = "cache-flush",
    feature = "register-scrub"
))]
pub const HARDENING_FEATURES_ENABLED: bool = true;

/// Whether the legacy hardening feature set was selected at compile time.
///
/// This does not claim that any runtime memory protection succeeded. Inspect
/// the [`ProtectionReport`] returned by protected containers.
#[cfg(not(all(
    feature = "memory-lock",
    feature = "multi-pass-clear",
    feature = "cache-flush",
    feature = "register-scrub"
)))]
pub const HARDENING_FEATURES_ENABLED: bool = false;

/// Whether the legacy hardening feature set was selected at compile time.
///
/// This compatibility alias does not prove that runtime memory protection
/// succeeded. Inspect [`ProtectionReport`] for protected containers.
pub const HARDENED_MODE: bool = HARDENING_FEATURES_ENABLED;

/// Sanitizes an ordinary byte slice in place.
///
/// New code should use [`wipe::bytes`].
pub fn sanitize_bytes(bytes: &mut [u8]) {
    wipe::bytes(bytes);
}

/// Legacy compatibility surface for the pre-0.8 best-effort API.
pub mod best_effort {
    /// Sanitizes an ordinary byte slice in place.
    ///
    /// New code should use [`crate::wipe::bytes`].
    pub fn sanitize_bytes_best_effort(bytes: &mut [u8]) {
        crate::wipe::bytes(bytes);
    }
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
    wipe::array(bytes);
}
