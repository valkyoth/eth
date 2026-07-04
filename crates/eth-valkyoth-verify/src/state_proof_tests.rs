extern crate std;

use std::{vec, vec::Vec};

use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_hash::{Keccak256, Keccak256Digest};
use eth_valkyoth_primitives::{Address, B256};

use crate::MAX_PROOF_WALK_DEPTH;

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
struct TestHasher {
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
fn verifies_account_inclusion_leaf_root() {
    let address = test_address();
    let key = test_hash(&address.to_bytes()).to_bytes();
    let account = account_value();
    let root_node = leaf_node(&key, &account);
    let root = AccountTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let verified = verify_account_inclusion(
        root,
        address,
        &account,
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(verified.map(|value| value.address()), Ok(address));
}

#[test]
fn verifies_storage_inclusion_leaf_root() {
    let slot = test_slot();
    let key = test_hash(&slot.to_b256().to_bytes()).to_bytes();
    let value = storage_value();
    let root_node = leaf_node(&key, &value);
    let root = StorageTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let verified =
        verify_storage_inclusion(root, slot, &value, &proof, TEST_LIMITS, TestHasher::default);

    assert_eq!(verified.map(|value| value.slot()), Ok(slot));
}

#[test]
fn rejects_missing_account_proof_node() -> Result<(), &'static str> {
    let address = test_address();
    let key = test_hash(&address.to_bytes()).to_bytes();
    let nibbles = key_nibbles(&key);
    let child_nibble = nibbles.first().copied().ok_or("hashed key nibble")?;
    let child_hash = test_hash(b"account-child-not-provided").to_bytes();
    let root_node = branch_node(child_nibble, scalar(&child_hash), scalar(b""));
    let root = AccountTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let error = verify_account_inclusion(
        root,
        address,
        &account_value(),
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::MissingProofNode));
    Ok(())
}

#[test]
fn rejects_storage_value_mismatch() {
    let slot = test_slot();
    let key = test_hash(&slot.to_b256().to_bytes()).to_bytes();
    let value = storage_value();
    let root_node = leaf_node(&key, &value);
    let root = StorageTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let error = verify_storage_inclusion(
        root,
        slot,
        b"different-storage-value",
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::ValueMismatch));
}

#[test]
fn rejects_wrong_account_root() {
    let address = test_address();
    let key = test_hash(&address.to_bytes()).to_bytes();
    let account = account_value();
    let root_node = leaf_node(&key, &account);
    let wrong_root = AccountTrieRoot::from_b256(test_hash(b"not-the-account-root"));
    let proof = [&root_node[..]];

    let error = verify_account_inclusion(
        wrong_root,
        address,
        &account,
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::WrongRoot));
}

#[test]
fn rejects_wrong_storage_root() {
    let slot = test_slot();
    let key = test_hash(&slot.to_b256().to_bytes()).to_bytes();
    let value = storage_value();
    let root_node = leaf_node(&key, &value);
    let wrong_root = StorageTrieRoot::from_b256(test_hash(b"not-the-storage-root"));
    let proof = [&root_node[..]];

    let error = verify_storage_inclusion(
        wrong_root,
        slot,
        &value,
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::WrongRoot));
}

#[test]
fn rejects_absent_account_key() -> Result<(), &'static str> {
    let address = test_address();
    let key = test_hash(&address.to_bytes()).to_bytes();
    let child_nibble = key_nibbles(&key).first().copied().ok_or("key nibble")?;
    let absent_nibble = child_nibble.wrapping_add(1) & 0x0f;
    let root_node = branch_node(absent_nibble, scalar(b""), scalar(b""));
    let root = AccountTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let error = verify_account_inclusion(
        root,
        address,
        &account_value(),
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::Absent));
    Ok(())
}

#[test]
fn rejects_absent_storage_key() -> Result<(), &'static str> {
    let slot = test_slot();
    let key = test_hash(&slot.to_b256().to_bytes()).to_bytes();
    let child_nibble = key_nibbles(&key).first().copied().ok_or("key nibble")?;
    let absent_nibble = child_nibble.wrapping_add(1) & 0x0f;
    let root_node = branch_node(absent_nibble, scalar(b""), scalar(b""));
    let root = StorageTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];

    let error = verify_storage_inclusion(
        root,
        slot,
        &storage_value(),
        &proof,
        TEST_LIMITS,
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::Absent));
    Ok(())
}

#[test]
fn rejects_account_proof_walk_beyond_fixed_depth_cap() -> Result<(), &'static str> {
    let address = test_address();
    let key = test_hash(&address.to_bytes()).to_bytes();
    let value = account_value();
    let mut proof_nodes = vec![leaf_node(&key, &value)];
    for _ in 0..MAX_PROOF_WALK_DEPTH {
        let child = proof_nodes.first().ok_or("proof node exists")?;
        let child_hash = test_hash(child).to_bytes();
        proof_nodes.insert(0, extension_node_hash(&child_hash));
    }
    let root_node = proof_nodes.first().ok_or("root node exists")?;
    let root = AccountTrieRoot::from_b256(test_hash(root_node));
    let proof = proof_nodes
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<&[u8]>>();

    let error = verify_account_inclusion(
        root,
        address,
        &value,
        &proof,
        deep_limits(),
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::ProofTooDeep));
    Ok(())
}

#[test]
fn rejects_storage_proof_walk_beyond_fixed_depth_cap() -> Result<(), &'static str> {
    let slot = test_slot();
    let key = test_hash(&slot.to_b256().to_bytes()).to_bytes();
    let value = storage_value();
    let mut proof_nodes = vec![leaf_node(&key, &value)];
    for _ in 0..MAX_PROOF_WALK_DEPTH {
        let child = proof_nodes.first().ok_or("proof node exists")?;
        let child_hash = test_hash(child).to_bytes();
        proof_nodes.insert(0, extension_node_hash(&child_hash));
    }
    let root_node = proof_nodes.first().ok_or("root node exists")?;
    let root = StorageTrieRoot::from_b256(test_hash(root_node));
    let proof = proof_nodes
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<&[u8]>>();

    let error = verify_storage_inclusion(
        root,
        slot,
        &value,
        &proof,
        deep_limits(),
        TestHasher::default,
    );

    assert_eq!(error, Err(MptProofVerificationError::ProofTooDeep));
    Ok(())
}

fn test_hash(input: &[u8]) -> B256 {
    let mut hasher = TestHasher::default();
    hasher.update(input);
    hasher.finalize()
}

fn test_address() -> Address {
    Address::from_bytes(core::array::from_fn(|index| {
        u8::try_from(index)
            .map(|value| value.wrapping_add(1))
            .unwrap_or(u8::MAX)
    }))
}

fn test_slot() -> StorageSlotKey {
    StorageSlotKey::from_b256(B256::from_bytes(core::array::from_fn(|index| {
        u8::try_from(index)
            .map(|value| value.wrapping_add(33))
            .unwrap_or(u8::MAX)
    })))
}

fn account_value() -> Vec<u8> {
    list(&[scalar(b"account")])
}

fn storage_value() -> Vec<u8> {
    scalar(b"storage")
}

fn leaf_node(key: &[u8], value: &[u8]) -> Vec<u8> {
    list(&[scalar(&compact_path_leaf(key)), scalar(value)])
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
    list(&items)
}

fn extension_node_hash(child_hash: &[u8; 32]) -> Vec<u8> {
    list(&[scalar(&[0x00]), scalar(child_hash)])
}

fn deep_limits() -> DecodeLimits {
    DecodeLimits {
        max_input_bytes: 4096,
        max_list_items: 256,
        max_nesting_depth: 16,
        max_total_allocation: 16_384,
        max_proof_nodes: 256,
        max_total_items: 1024,
    }
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
