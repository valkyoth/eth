use eth_valkyoth_codec::{
    DecodeLimits, encode_rlp_list_payload, encode_rlp_scalar, encoded_rlp_list_len,
    encoded_rlp_scalar_len,
};
use eth_valkyoth_protocol::{
    ACCESS_LIST_TRANSACTION_TYPE, BLOB_TRANSACTION_TYPE, DYNAMIC_FEE_TRANSACTION_TYPE,
    SET_CODE_TRANSACTION_TYPE,
};
use std::{vec, vec::Vec};

const LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 256,
    max_list_items: 64,
    max_nesting_depth: 8,
    max_total_allocation: 256,
    max_proof_nodes: 4,
    max_total_items: 256,
};

pub(crate) fn legacy_transaction() -> Option<Vec<u8>> {
    let chain = test_byte(b"c")?;
    let replay_base = test_byte(b"eip155-replay-domain-signature-base")?;
    list(&[
        scalar(&[test_byte(b"nonce")?])?,
        scalar(&[test_byte(b"gas-price")?])?,
        scalar(b"R\x08")?,
        scalar(&[])?,
        scalar(&[])?,
        scalar(&[])?,
        scalar(&[replay_base.checked_add(chain)?.checked_add(chain)?])?,
        scalar(&[test_byte(b"r")?])?,
        scalar(&[test_byte(b"ss")?])?,
    ])
}

pub(crate) fn access_list_transaction() -> Option<Vec<u8>> {
    typed(
        ACCESS_LIST_TRANSACTION_TYPE,
        &[
            scalar(&[test_byte(b"c")?])?,
            scalar(&[test_byte(b"nn")?])?,
            scalar(&[test_byte(b"fee")?])?,
            scalar(b"R\x08")?,
            scalar(&[])?,
            scalar(&[test_byte(b"value")?])?,
            scalar(&[])?,
            list(&[])?,
            scalar(&[u8::from(true)])?,
            scalar(&[test_byte(b"r")?])?,
            scalar(&[test_byte(b"ss")?])?,
        ],
    )
}

pub(crate) fn dynamic_fee_transaction() -> Option<Vec<u8>> {
    typed(
        DYNAMIC_FEE_TRANSACTION_TYPE,
        &[
            scalar(&[test_byte(b"c")?])?,
            scalar(&[test_byte(b"nn")?])?,
            scalar(&[test_byte(b"tip")?])?,
            scalar(&[test_byte(b"fees")?])?,
            scalar(b"R\x08")?,
            scalar(&[])?,
            scalar(&[test_byte(b"value")?])?,
            scalar(&[])?,
            list(&[])?,
            scalar(&[u8::from(true)])?,
            scalar(&[test_byte(b"r")?])?,
            scalar(&[test_byte(b"ss")?])?,
        ],
    )
}

pub(crate) fn blob_transaction() -> Option<Vec<u8>> {
    let address = repeated_label(b"blob-recipient");
    let versioned_hash = repeated_label(b"blob-versioned-hash");
    typed(
        BLOB_TRANSACTION_TYPE,
        &[
            scalar(&[test_byte(b"c")?])?,
            scalar(&[test_byte(b"nn")?])?,
            scalar(&[test_byte(b"tip")?])?,
            scalar(&[test_byte(b"fees")?])?,
            scalar(b"R\x08")?,
            scalar(address.get(..20)?)?,
            scalar(&[test_byte(b"value")?])?,
            scalar(&[])?,
            list(&[])?,
            scalar(&[test_byte(b"blob-fee")?])?,
            list(&[scalar(&versioned_hash)?])?,
            scalar(&[u8::from(true)])?,
            scalar(&[test_byte(b"r")?])?,
            scalar(&[test_byte(b"ss")?])?,
        ],
    )
}

pub(crate) fn set_code_transaction() -> Option<Vec<u8>> {
    let address = repeated_label(b"set-code-recipient");
    typed(
        SET_CODE_TRANSACTION_TYPE,
        &[
            scalar(&[test_byte(b"c")?])?,
            scalar(&[test_byte(b"nn")?])?,
            scalar(&[test_byte(b"tip")?])?,
            scalar(&[test_byte(b"fees")?])?,
            scalar(b"R\x08")?,
            scalar(address.get(..20)?)?,
            scalar(&[test_byte(b"value")?])?,
            scalar(&[])?,
            list(&[])?,
            list(&[])?,
            scalar(&[u8::from(true)])?,
            scalar(&[test_byte(b"r")?])?,
            scalar(&[test_byte(b"ss")?])?,
        ],
    )
}

fn typed(type_byte: u8, fields: &[Vec<u8>]) -> Option<Vec<u8>> {
    let payload = list(fields)?;
    let mut transaction = Vec::new();
    transaction
        .try_reserve_exact(payload.len().checked_add(1)?)
        .ok()?;
    transaction.push(type_byte);
    transaction.extend_from_slice(&payload);
    Some(transaction)
}

fn scalar(payload: &[u8]) -> Option<Vec<u8>> {
    let mut output = vec![u8::default(); encoded_rlp_scalar_len(payload).ok()?];
    let written = encode_rlp_scalar(payload, &mut output).ok()?;
    (written == output.len()).then_some(output)
}

fn list(items: &[Vec<u8>]) -> Option<Vec<u8>> {
    let payload_len = items
        .iter()
        .try_fold(0usize, |total, item| total.checked_add(item.len()))?;
    let mut payload = Vec::new();
    payload.try_reserve_exact(payload_len).ok()?;
    for item in items {
        payload.extend_from_slice(item);
    }
    let mut output = vec![u8::default(); encoded_rlp_list_len(&payload, LIMITS).ok()?];
    let written = encode_rlp_list_payload(&payload, LIMITS, &mut output).ok()?;
    (written == output.len()).then_some(output)
}

fn test_byte(label: &[u8]) -> Option<u8> {
    u8::try_from(label.len()).ok().filter(|value| *value != 0)
}

fn repeated_label(label: &[u8]) -> [u8; 32] {
    core::array::from_fn(|index| {
        let source = index.checked_rem(label.len()).unwrap_or_default();
        label.get(source).copied().unwrap_or_default()
    })
}
