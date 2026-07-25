#![cfg(feature = "sanitization")]
#![deny(deprecated)]
//! Patch-release compatibility checks for the optional sanitization facade.

/// Keeps the `eth 0.52.4` sanitization imports source-compatible in `0.52.5`.
#[test]
fn eth_0_52_4_sanitization_surface_remains_compatible() -> Result<(), &'static str> {
    let mut bytes = std::env::args()
        .next()
        .ok_or("test executable path")?
        .into_bytes();
    assert!(!bytes.is_empty());

    eth::sanitization::sanitize_bytes(&mut bytes);
    assert!(bytes.iter().all(|byte| *byte == u8::MIN));

    bytes = std::env::args()
        .next()
        .ok_or("test executable path")?
        .into_bytes();
    eth::sanitization::best_effort::sanitize_bytes_best_effort(&mut bytes);
    assert!(bytes.iter().all(|byte| *byte == u8::MIN));

    assert_eq!(
        eth::sanitization::HARDENED_MODE,
        eth::sanitization::HARDENING_FEATURES_ENABLED
    );
    Ok(())
}
