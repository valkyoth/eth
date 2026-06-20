#![allow(missing_docs)]

use eth_valkyoth_sanitization::{SecretBytes32, SecureSanitize, sanitize_fixed};

#[test]
fn sanitize_fixed_clears_array() {
    let mut bytes = [0x42_u8; 32];

    sanitize_fixed(&mut bytes);

    assert_eq!(bytes, [0_u8; 32]);
}

#[test]
fn secret_alias_can_be_sanitized() {
    let mut secret = SecretBytes32::from_array([0x11_u8; 32]);

    secret.secure_sanitize();

    assert!(secret.constant_time_eq(&[0_u8; 32]));
}

#[test]
fn best_effort_api_is_namespaced() {
    let mut bytes = [0x24_u8; 32];

    eth_valkyoth_sanitization::best_effort::sanitize_bytes_best_effort(&mut bytes);

    assert_eq!(bytes, [0_u8; 32]);
}
