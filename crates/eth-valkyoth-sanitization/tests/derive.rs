#![allow(missing_docs)]
#![cfg(feature = "derive")]

use eth_valkyoth_sanitization::{SecretBytes32, SecureSanitize};

#[derive(eth_valkyoth_sanitization::SecureSanitize)]
struct KeyHolder {
    key: SecretBytes32,
    #[eth_sanitization(skip, reason = "non-secret label")]
    label: u8,
}

#[derive(eth_valkyoth_sanitization::SecureSanitize)]
#[eth_sanitization(enum_inactive_variant_bytes = "acknowledged")]
enum SecretChoice {
    Key(SecretBytes32),
    Named { key: SecretBytes32 },
    Empty,
}

#[derive(
    eth_valkyoth_sanitization::SecureSanitize, eth_valkyoth_sanitization::SecureSanitizeOnDrop,
)]
struct DropHolder {
    key: SecretBytes32,
}

#[test]
fn derive_secure_sanitize_clears_struct_fields() {
    let mut holder = KeyHolder {
        key: SecretBytes32::from_array([0x11_u8; 32]),
        label: 7,
    };

    holder.secure_sanitize();

    assert!(holder.key.constant_time_eq(&[0_u8; 32]));
    assert_eq!(holder.label, 7);
}

#[test]
fn derive_secure_sanitize_clears_enum_tuple_variant() {
    let mut choice = SecretChoice::Key(SecretBytes32::from_array([0x22_u8; 32]));

    choice.secure_sanitize();

    let cleared = match choice {
        SecretChoice::Key(key) => key.constant_time_eq(&[0_u8; 32]),
        SecretChoice::Named { .. } | SecretChoice::Empty => false,
    };
    assert!(cleared);
}

#[test]
fn derive_secure_sanitize_clears_enum_named_variant() {
    let mut choice = SecretChoice::Named {
        key: SecretBytes32::from_array([0x33_u8; 32]),
    };

    choice.secure_sanitize();

    let cleared = match choice {
        SecretChoice::Named { key } => key.constant_time_eq(&[0_u8; 32]),
        SecretChoice::Key(_) | SecretChoice::Empty => false,
    };
    assert!(cleared);
}

#[test]
fn derive_secure_sanitize_on_drop_compiles() {
    let holder = DropHolder {
        key: SecretBytes32::from_array([0x44_u8; 32]),
    };

    assert!(!holder.key.constant_time_eq(&[0_u8; 32]));
    drop(holder);
}

#[test]
fn derive_secure_sanitize_handles_empty_enum_variant() {
    let mut choice = SecretChoice::Empty;

    choice.secure_sanitize();

    assert!(matches!(choice, SecretChoice::Empty));
}
