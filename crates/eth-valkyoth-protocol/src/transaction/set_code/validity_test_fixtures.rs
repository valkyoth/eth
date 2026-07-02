use eth_valkyoth_primitives::{Address, ChainId, Gas, Nonce};
use std::vec::Vec;

pub(super) fn authority_address() -> Address {
    Address::from_bytes(test_address_from_label(b"authority"))
}

pub(super) fn expected_chain_id() -> ChainId {
    ChainId::new(u64::from(expected_chain_id_byte()))
}

pub(super) fn expected_chain_id_payload() -> Vec<u8> {
    byte_payload(expected_chain_id_byte())
}

pub(super) fn unexpected_chain_id_payload() -> Vec<u8> {
    byte_payload(expected_chain_id_byte().saturating_add(u8::from(true)))
}

pub(super) fn universal_chain_id_payload() -> Vec<u8> {
    Vec::new()
}

pub(super) fn starting_nonce() -> Nonce {
    Nonce::new(u64::from(starting_nonce_byte()))
}

pub(super) fn next_nonce() -> Nonce {
    Nonce::new(u64::from(next_nonce_byte()))
}

pub(super) fn uninitialized_nonce() -> Nonce {
    Nonce::new(u64::default())
}

pub(super) fn starting_nonce_payload() -> Vec<u8> {
    byte_payload(starting_nonce_byte())
}

pub(super) fn next_nonce_payload() -> Vec<u8> {
    byte_payload(next_nonce_byte())
}

pub(super) fn uninitialized_nonce_payload() -> Vec<u8> {
    Vec::new()
}

pub(super) fn max_nonce_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.resize(core::mem::size_of::<u64>(), u8::MAX);
    payload
}

pub(super) fn default_priority_fee_payload() -> Vec<u8> {
    byte_payload(fee_byte())
}

pub(super) fn priority_fee_too_high_payload() -> Vec<u8> {
    byte_payload(fee_byte())
}

pub(super) fn low_priority_fee_payload() -> Vec<u8> {
    byte_payload(expected_chain_id_byte())
}

pub(super) fn default_max_fee_payload() -> Vec<u8> {
    byte_payload(starting_nonce_byte())
}

pub(super) fn max_fee_payload() -> Vec<u8> {
    byte_payload(expected_chain_id_byte().saturating_add(u8::from(true)))
}

pub(super) fn gas_limit_payload() -> Vec<u8> {
    canonical_u64_payload(Gas::new(21_000).get())
}

fn transaction_nonce_payload() -> Vec<u8> {
    byte_payload(expected_chain_id_byte().saturating_add(u8::from(true)))
}

fn transaction_value_payload() -> Vec<u8> {
    byte_payload(next_nonce_byte())
}

fn valid_y_parity_payload() -> Vec<u8> {
    byte_payload(u8::from(true))
}

fn signature_r_payload() -> Vec<u8> {
    byte_payload(u8::from(true))
}

fn signature_s_payload() -> Vec<u8> {
    byte_payload(signature_s_byte())
}

fn expected_chain_id_byte() -> u8 {
    u8::from(true)
}

fn fee_byte() -> u8 {
    u8::try_from("fee".len()).unwrap_or_default()
}

fn starting_nonce_byte() -> u8 {
    u8::try_from("auth".len()).unwrap_or_default()
}

fn next_nonce_byte() -> u8 {
    starting_nonce_byte().saturating_add(u8::from(true))
}

pub(super) fn inactive_block_number() -> u64 {
    u64::try_from("inactive".len())
        .unwrap_or_default()
        .saturating_add(u64::from(u8::from(true)))
}

pub(super) fn active_block_number() -> u64 {
    u64::try_from("activation".len()).unwrap_or_default()
}

pub(super) fn active_timestamp() -> u64 {
    active_block_number().saturating_mul(u64::from(signature_s_byte()))
}

fn signature_s_byte() -> u8 {
    expected_chain_id_byte().saturating_add(u8::from(true))
}

fn byte_payload(byte: u8) -> Vec<u8> {
    core::iter::once(byte).collect()
}

fn canonical_u64_payload(value: u64) -> Vec<u8> {
    let mut bytes = value.to_be_bytes().to_vec();
    let first_non_zero = bytes
        .iter()
        .position(|byte| *byte != u8::default())
        .unwrap_or(bytes.len());
    bytes.drain(..first_non_zero);
    bytes
}

pub(super) fn set_code_tx(
    chain_id: &[u8],
    priority_fee: &[u8],
    max_fee: &[u8],
    gas_limit: &[u8],
    authorizations: &[Vec<u8>],
) -> Vec<u8> {
    let mut fields = Vec::new();
    push_scalar(&mut fields, chain_id);
    push_scalar(&mut fields, &transaction_nonce_payload());
    push_scalar(&mut fields, priority_fee);
    push_scalar(&mut fields, max_fee);
    push_scalar(&mut fields, gas_limit);
    push_scalar(&mut fields, &test_address_from_label(b"set-code-to"));
    push_scalar(&mut fields, &transaction_value_payload());
    push_scalar(&mut fields, &[]);
    push_list(&mut fields, &[]);

    let mut auth_list = Vec::new();
    for authorization in authorizations {
        auth_list.extend_from_slice(authorization);
    }
    push_list(&mut fields, &auth_list);

    push_scalar(&mut fields, &valid_y_parity_payload());
    push_scalar(&mut fields, &signature_r_payload());
    push_scalar(&mut fields, &signature_s_payload());

    let mut tx = Vec::new();
    tx.push(crate::SET_CODE_TRANSACTION_TYPE);
    push_list(&mut tx, &fields);
    tx
}

pub(super) fn authorization_tuple(chain_id: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut fields = Vec::new();
    push_scalar(&mut fields, chain_id);
    push_scalar(&mut fields, &test_address_from_label(b"set-code-auth"));
    push_scalar(&mut fields, nonce);
    push_scalar(&mut fields, &valid_y_parity_payload());
    push_scalar(&mut fields, &signature_r_payload());
    push_scalar(&mut fields, &signature_s_payload());

    let mut tuple = Vec::new();
    push_list(&mut tuple, &fields);
    tuple
}

fn test_address_from_label(label: &[u8]) -> [u8; 20] {
    let mut bytes = [0_u8; 20];
    if label.is_empty() {
        return bytes;
    }
    for (index, byte) in bytes.iter_mut().enumerate() {
        let Some(label_index) = index.checked_rem(label.len()) else {
            return bytes;
        };
        let label_byte = label.get(label_index).copied().unwrap_or_default();
        *byte = label_byte.wrapping_add(u8::try_from(index).unwrap_or_default());
    }
    bytes
}

fn push_scalar(out: &mut Vec<u8>, payload: &[u8]) {
    if let Some(byte) = payload
        .first()
        .copied()
        .filter(|byte| payload.len() == 1 && *byte < 0x80)
    {
        out.push(byte);
    } else {
        push_prefixed(out, 0x80, payload);
    }
}

fn push_list(out: &mut Vec<u8>, payload: &[u8]) {
    push_prefixed(out, 0xc0, payload);
}

fn push_prefixed(out: &mut Vec<u8>, short_base: u8, payload: &[u8]) {
    if payload.len() <= 55 {
        let Ok(length) = u8::try_from(payload.len()) else {
            return;
        };
        let Some(prefix) = short_base.checked_add(length) else {
            return;
        };
        out.push(prefix);
    } else {
        let Some(prefix) = short_base.checked_add(56) else {
            return;
        };
        let Ok(length) = u8::try_from(payload.len()) else {
            return;
        };
        out.push(prefix);
        out.push(length);
    }
    out.extend_from_slice(payload);
}
