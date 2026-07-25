#![cfg(feature = "sanitization")]
#![allow(deprecated)]
//! Patch-release compatibility checks for the optional sanitization facade.

/// Keeps the `eth 0.52.4` sanitization imports source-compatible in `0.52.5`.
#[test]
fn eth_0_52_4_sanitization_surface_still_compiles() {
    let mut bytes = [];

    eth::sanitization::sanitize_bytes(&mut bytes);
    eth::sanitization::best_effort::sanitize_bytes_best_effort(&mut bytes);
    let _ = core::hint::black_box(eth::sanitization::HARDENED_MODE);
}
