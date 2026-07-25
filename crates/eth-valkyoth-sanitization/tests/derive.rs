#![allow(missing_docs)]
#![cfg(feature = "derive")]

use eth_valkyoth_sanitization::{DropSafeSanitize, SecretBytes32, SecureSanitize};

#[derive(eth_valkyoth_sanitization::SecureSanitize)]
struct KeyHolder {
    key: SecretBytes32,
    #[eth_sanitization(skip, reason = "non-secret label")]
    label: u8,
}

#[derive(
    eth_valkyoth_sanitization::SecureSanitize, eth_valkyoth_sanitization::SecureSanitizeOnDrop,
)]
struct DropHolder {
    key: SecretBytes32,
}

#[derive(
    eth_valkyoth_sanitization::SecureSanitize, eth_valkyoth_sanitization::SecureSanitizeOnDrop,
)]
struct GenericDropHolder<T: SecureSanitize + Unpin> {
    key: T,
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
fn derive_secure_sanitize_attests_drop_contract() {
    fn require_drop_safe<T: DropSafeSanitize>() {}

    require_drop_safe::<KeyHolder>();
    require_drop_safe::<DropHolder>();
    require_drop_safe::<GenericDropHolder<SecretBytes32>>();
}

#[test]
fn derive_secure_sanitize_on_drop_compiles() {
    let holder = DropHolder {
        key: SecretBytes32::from_array([0x44_u8; 32]),
    };

    assert!(!holder.key.constant_time_eq(&[0_u8; 32]));
    drop(holder);

    let generic = GenericDropHolder {
        key: SecretBytes32::from_array([0x55_u8; 32]),
    };
    assert!(!generic.key.constant_time_eq(&[0_u8; 32]));
    drop(generic);
}
