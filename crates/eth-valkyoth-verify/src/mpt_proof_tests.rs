extern crate std;

use std::{cell::Cell, vec, vec::Vec};

use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};
use eth_valkyoth_hash::Keccak256Digest;

use super::*;

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 2048,
    max_list_items: 64,
    max_nesting_depth: 16,
    max_total_allocation: 4096,
    max_proof_nodes: 8,
    max_total_items: 256,
};

#[derive(Default)]
pub(super) struct TestHasher {
    state: [u8; 32],
}

impl Keccak256 for TestHasher {
    fn update(&mut self, input: &[u8]) {
        let mut slot = 0usize;
        let slot_count = self.state.len();
        for byte in input.iter().copied() {
            if let Some(target) = self.state.get_mut(slot) {
                *target = target
                    .wrapping_add(byte)
                    .wrapping_add(u8::try_from(slot).unwrap_or(u8::MAX));
            }
            slot = slot.saturating_add(1);
            if slot >= slot_count {
                slot = 0;
            }
        }
        if let Some(first) = self.state.first_mut() {
            *first = first.wrapping_add(u8::try_from(input.len()).unwrap_or(u8::MAX));
        }
    }

    fn finalize(self) -> Keccak256Digest {
        B256::from_bytes(self.state)
    }
}

#[test]
fn verifies_transaction_inclusion_leaf_root() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let verified =
        verify_transaction_inclusion(root, 0, &value, &proof, TEST_LIMITS, TestHasher::default)
            .map_err(|_| "proof should verify")?;

    assert_eq!(verified.index(), 0);
    assert_eq!(verified.root(), root);
    Ok(())
}

#[test]
fn verifies_receipt_inclusion_through_branch_child() -> Result<(), &'static str> {
    let key = index_key(1)?;
    let value = receipt_value();
    let key_nibbles = key_nibbles(&key);
    let child_path = key_nibbles.get(1..).ok_or("child path slice")?;
    let child = leaf_node_from_nibbles(child_path, &value);
    let child_hash = test_hash(&child).to_bytes();
    let root_node = branch_node(0x00, scalar(&child_hash), scalar(b""));
    let root = ReceiptTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..], &child[..]];

    let verified =
        verify_receipt_inclusion(root, 1, &value, &proof, TEST_LIMITS, TestHasher::default)
            .map_err(|_| "receipt proof should verify")?;

    assert_eq!(verified.index(), 1);
    assert_eq!(verified.root(), root);
    Ok(())
}

#[test]
fn verifies_transaction_inclusion_through_inline_child() -> Result<(), &'static str> {
    let key = index_key(2)?;
    let value = tx_value();
    let key_nibbles = key_nibbles(&key);
    let child_path = key_nibbles.get(1..).ok_or("child path slice")?;
    let inline_child = leaf_node_from_nibbles(child_path, &value);
    if inline_child.len() >= 32 {
        return Err("inline child fixture must stay below the hash threshold");
    }
    let root_node = branch_node(0x00, inline_child, scalar(b""));
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let verified =
        verify_transaction_inclusion(root, 2, &value, &proof, TEST_LIMITS, TestHasher::default)
            .map_err(|_| "inline-child proof should verify")?;

    assert_eq!(verified.index(), 2);
    assert_eq!(verified.root(), root);
    Ok(())
}

#[test]
fn rejects_absent_transaction_key() -> Result<(), &'static str> {
    let key = index_key(1)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let error =
        verify_transaction_inclusion(root, 2, &value, &proof, TEST_LIMITS, TestHasher::default);

    assert_eq!(error, Err(MptProofVerificationError::Absent));
    Ok(())
}

#[test]
fn rejects_wrong_root() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let wrong_root = TransactionTrieRoot::from_b256(test_hash(b"wrong-root"));
    let proof = [&root_node[..]];

    let error = verify_transaction_inclusion(
        wrong_root,
        0,
        &value,
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::WrongRoot));
    Ok(())
}

#[test]
fn rejects_value_mismatch() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let error = verify_transaction_inclusion(
        root,
        0,
        b"different",
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::ValueMismatch));
    Ok(())
}

#[test]
fn rejects_missing_hashed_child_node() {
    let child_hash = test_hash(b"child-not-provided").to_bytes();
    let root_node = branch_node(0x00, scalar(&child_hash), scalar(b""));
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let error =
        verify_transaction_inclusion(root, 1, b"tx", &proof, TEST_LIMITS, TestHasher::default);

    assert_eq!(error, Err(MptProofVerificationError::MissingProofNode));
}

#[test]
fn rejects_trailing_proof_nodes() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let extra = leaf_node(&key, &receipt_value());
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..], &extra[..]];

    let error =
        verify_transaction_inclusion(root, 0, &value, &proof, TEST_LIMITS, TestHasher::default);

    assert_eq!(error, Err(MptProofVerificationError::TrailingProofNodes));
    Ok(())
}

#[test]
fn rejects_redundant_hashed_extension() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let leaf = leaf_node(&key, &value);
    let leaf_hash = test_hash(&leaf).to_bytes();
    let redundant = extension_node_hash(0x00, &leaf_hash);
    let redundant_hash = test_hash(&redundant).to_bytes();
    let root_node = extension_node_hash(0x08, &redundant_hash);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..], &redundant[..], &leaf[..]];
    let calls = Cell::new(0usize);

    let error = verify_transaction_inclusion(root, 0, &value, &proof, TEST_LIMITS, || {
        calls.set(calls.get().saturating_add(1));
        TestHasher::default()
    });

    assert_eq!(
        error,
        Err(MptProofVerificationError::MalformedNode(
            MptNodeDecodeError::NonCanonicalExtensionChild
        ))
    );
    assert_eq!(calls.get(), 0);
    Ok(())
}

#[test]
fn preflight_rejects_malformed_trailing_node_before_hashing() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let malformed = [0xc0_u8];
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..], &malformed[..]];
    let calls = Cell::new(0usize);

    let error = verify_transaction_inclusion(root, 0, &value, &proof, TEST_LIMITS, || {
        calls.set(calls.get().saturating_add(1));
        TestHasher::default()
    });

    assert!(matches!(
        error,
        Err(MptProofVerificationError::MalformedNode(_))
    ));
    assert_eq!(calls.get(), 0);
    Ok(())
}

#[test]
fn preflight_rejects_hash_budget_before_hashing() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];
    let policy =
        DecodeSessionPolicy::reviewed_policy(TEST_LIMITS, 4096, 1024, 0, 4096, 4096, 4096, 16_384)
            .map_err(|_| "policy")?;
    let mut session = DecodeSession::new(policy).map_err(|_| "session")?;
    let calls = Cell::new(0usize);

    let error =
        verify_transaction_inclusion_in_session(root, 0, &value, &proof, &mut session, || {
            calls.set(calls.get().saturating_add(1));
            TestHasher::default()
        });

    assert!(matches!(
        error,
        Err(MptProofVerificationError::MalformedNode(_))
    ));
    assert_eq!(calls.get(), 0);
    assert_eq!(session.hashes(), 0);
    Ok(())
}

#[test]
fn shared_session_accounts_complete_leaf_proof_work() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];
    let policy =
        DecodeSessionPolicy::reviewed_policy(TEST_LIMITS, 4096, 1024, 8, 4096, 4096, 4096, 16_384)
            .map_err(|_| "policy")?;
    let mut session = DecodeSession::new(policy).map_err(|_| "session")?;

    verify_transaction_inclusion_in_session(
        root,
        0,
        &value,
        &proof,
        &mut session,
        TestHasher::default,
    )
    .map_err(|_| "proof verifies")?;

    assert_eq!(session.proof_nodes(), 1);
    assert_eq!(session.hashes(), 1);
    assert_eq!(session.hash_bytes(), root_node.len());
    assert_eq!(session.nibbles(), 10);
    assert_eq!(session.value_bytes(), value.len().saturating_mul(8));
    Ok(())
}

#[test]
fn proof_error_categories_distinguish_absent_and_wrong_root() {
    assert_eq!(
        MptProofVerificationError::Absent.category(),
        MptProofVerificationErrorCategory::Absent
    );
    assert_eq!(
        MptProofVerificationError::WrongRoot.category(),
        MptProofVerificationErrorCategory::WrongRoot
    );
    assert_eq!(
        MptProofVerificationError::MissingProofNode.category(),
        MptProofVerificationErrorCategory::Malformed
    );
    assert_eq!(
        MptProofVerificationError::ProofTooDeep.category(),
        MptProofVerificationErrorCategory::Malformed
    );
}

pub(super) fn test_hash(input: &[u8]) -> B256 {
    let mut hasher = TestHasher::default();
    hasher.update(input);
    hasher.finalize()
}

pub(super) fn index_key(index: u64) -> Result<Vec<u8>, &'static str> {
    let mut output = [0_u8; MAX_RLP_U64_BYTES];
    let len = encode_index_key(index, &mut output).map_err(|_| "index key encodes")?;
    output
        .get(..len)
        .map(<[u8]>::to_vec)
        .ok_or("index key slice")
}

pub(super) fn tx_value() -> Vec<u8> {
    scalar(b"tx")
}

fn receipt_value() -> Vec<u8> {
    let payload = (0..40)
        .map(|index| u8::try_from(index).unwrap_or(u8::MAX))
        .collect::<Vec<_>>();
    scalar(&payload)
}

pub(super) fn leaf_node(key: &[u8], value: &[u8]) -> Vec<u8> {
    list(&[scalar(&compact_path_leaf(key)), scalar(value)])
}

fn leaf_node_from_nibbles(nibbles: &[u8], value: &[u8]) -> Vec<u8> {
    list(&[scalar(&compact_path_leaf_nibbles(nibbles)), scalar(value)])
}

fn extension_node_hash(nibble: u8, child_hash: &[u8; 32]) -> Vec<u8> {
    list(&[scalar(&[0x10 | (nibble & 0x0f)]), scalar(child_hash)])
}

fn branch_node(child_nibble: u8, child: Vec<u8>, value: Vec<u8>) -> Vec<u8> {
    let mut items = Vec::new();
    for index in 0..16 {
        if index == usize::from(child_nibble) {
            items.push(child.clone());
        } else {
            items.push(scalar(b""));
        }
    }
    items.push(value);
    let mut occupied = items
        .iter()
        .filter(|item| item.as_slice() != [0x80])
        .count();
    let mut offset = 1u8;
    while occupied < 2 {
        let index = usize::from(child_nibble.wrapping_add(offset) & 0x0f);
        if let Some(item) = items.get_mut(index)
            && item.as_slice() == [0x80]
        {
            *item = leaf_node_from_nibbles(&[offset & 0x0f], b"d");
            occupied = occupied.saturating_add(1);
        }
        offset = offset.saturating_add(1);
    }
    list(&items)
}

fn compact_path_leaf(path: &[u8]) -> Vec<u8> {
    compact_path_leaf_nibbles(&key_nibbles(path))
}

fn compact_path_leaf_nibbles(nibbles: &[u8]) -> Vec<u8> {
    let nibble_count = nibbles.len();
    let mut out = Vec::new();
    if nibble_count.is_multiple_of(2) {
        out.push(0x20);
        append_packed_nibbles(nibbles, &mut out);
    } else {
        out.push(0x30 | nibbles.first().copied().unwrap_or(0));
        append_packed_nibbles(nibbles.get(1..).unwrap_or(&[]), &mut out);
    }
    out
}

fn append_packed_nibbles(nibbles: &[u8], out: &mut Vec<u8>) {
    for pair in nibbles.chunks(2) {
        let high = pair.first().copied().unwrap_or(0);
        let low = pair.get(1).copied().unwrap_or(0);
        out.push((high << 4) | low);
    }
}

fn key_nibbles(path: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for byte in path {
        out.push(byte >> 4);
        out.push(byte & 0x0f);
    }
    out
}

fn scalar(payload: &[u8]) -> Vec<u8> {
    if let [byte] = payload
        && *byte < 0x80
    {
        return vec![*byte];
    }
    let mut output = Vec::new();
    append_header(0x80, payload.len(), &mut output);
    output.extend_from_slice(payload);
    output
}

fn list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = items.iter().map(Vec::len).sum();
    let mut output = Vec::new();
    append_header(0xc0, payload_len, &mut output);
    for item in items {
        output.extend_from_slice(item);
    }
    output
}

fn append_header(offset: u8, payload_len: usize, output: &mut Vec<u8>) {
    if payload_len < 56 {
        output.push(
            offset
                .checked_add(u8::try_from(payload_len).unwrap_or(0))
                .unwrap_or(offset),
        );
        return;
    }
    let len_bytes = payload_len.to_be_bytes();
    let first = len_bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(len_bytes.len().saturating_sub(1));
    let encoded_len = len_bytes.get(first..).unwrap_or(&[]);
    let prefix = offset
        .checked_add(55)
        .and_then(|value| value.checked_add(u8::try_from(encoded_len.len()).unwrap_or(0)))
        .unwrap_or(offset);
    output.push(prefix);
    output.extend_from_slice(encoded_len);
}
