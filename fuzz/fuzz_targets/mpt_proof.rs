#![no_main]

use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};
use eth_valkyoth_hash::{TinyKeccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256};
use eth_valkyoth_verify::{
    AccountTrieRoot, ReceiptTrieRoot, StorageSlotKey, StorageTrieRoot, TransactionTrieRoot,
    verify_account_inclusion, verify_receipt_inclusion, verify_storage_inclusion,
    verify_transaction_inclusion, verify_transaction_inclusion_in_session,
};
use libfuzzer_sys::fuzz_target;
use std::vec::Vec;

const ROOT_BYTES: usize = 32;
const ADDRESS_BYTES: usize = 20;
const SLOT_BYTES: usize = 32;
const INDEX_BYTES: usize = 8;
const HEADER_BYTES: usize = ROOT_BYTES + ADDRESS_BYTES + SLOT_BYTES + INDEX_BYTES;
const MAX_VALUE_BYTES: usize = 256;
const MAX_PROOF_NODES: usize = 8;
const MAX_PROOF_NODE_BYTES: usize = 512;

fuzz_target!(|data: &[u8]| {
    drive_structured_leaf_proof(data);
    let Some(input) = ProofInput::parse(data) else {
        return;
    };
    input.drive(DecodeLimits::TEST_FIXTURE);
    input.drive(DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_structured_leaf_proof(data: &[u8]) {
    let Some(selector) = data.first().copied() else {
        return;
    };
    let value_end = data.len().min(MAX_VALUE_BYTES.saturating_add(1));
    let Some(value) = data.get(1..value_end) else {
        return;
    };
    if value.is_empty() {
        return;
    }

    let leaf = transaction_zero_leaf(value);
    let root = TransactionTrieRoot::from_b256(hash_one(TinyKeccak256::default(), &leaf));
    let proof = [&leaf[..]];
    let policy = DecodeSessionPolicy::TEST_FIXTURE;
    let mut session = DecodeSession::new(policy).expect("fixture policy is valid");
    let result = verify_transaction_inclusion_in_session(
        root,
        0,
        value,
        &proof,
        &mut session,
        TinyKeccak256::default,
    );
    assert!(result.is_ok());
    assert!(session.total_work() <= policy.max_total_work());
    assert!(session.hashes() <= policy.max_hashes());
    assert!(session.hash_bytes() <= policy.max_hash_bytes());
    assert!(session.nibbles() <= policy.max_nibbles());
    assert!(session.value_bytes() <= policy.max_value_bytes());

    let mut mutated = leaf;
    let mutation_index = usize::from(selector) % mutated.len();
    mutated[mutation_index] ^= u8::MAX;
    let mutated_proof = [&mutated[..]];
    assert!(
        verify_transaction_inclusion(
            root,
            0,
            value,
            &mutated_proof,
            DecodeLimits::TEST_FIXTURE,
            TinyKeccak256::default,
        )
        .is_err()
    );
}

fn transaction_zero_leaf(value: &[u8]) -> Vec<u8> {
    let compact_path = core::iter::once(2_u8 << 4)
        .chain(core::iter::once(1_u8 << 7))
        .collect::<Vec<_>>();
    let path = scalar(&compact_path);
    let value = scalar(value);
    list(&[path, value])
}

fn scalar(payload: &[u8]) -> Vec<u8> {
    if let [byte] = payload
        && *byte < (1_u8 << 7)
    {
        return Vec::from([*byte]);
    }
    let mut output = Vec::new();
    append_header(1_u8 << 7, payload.len(), &mut output);
    output.extend_from_slice(payload);
    output
}

fn list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = items.iter().map(Vec::len).sum();
    let mut output = Vec::new();
    append_header(3_u8 << 6, payload_len, &mut output);
    for item in items {
        output.extend_from_slice(item);
    }
    output
}

fn append_header(offset: u8, payload_len: usize, output: &mut Vec<u8>) {
    if payload_len < 56 {
        output.push(offset.saturating_add(u8::try_from(payload_len).unwrap_or(u8::MAX)));
        return;
    }
    let len_bytes = payload_len.to_be_bytes();
    let first = len_bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(len_bytes.len().saturating_sub(1));
    let encoded_len = len_bytes.get(first..).unwrap_or(&[]);
    output.push(
        offset
            .saturating_add(55)
            .saturating_add(u8::try_from(encoded_len.len()).unwrap_or(u8::MAX)),
    );
    output.extend_from_slice(encoded_len);
}

struct ProofInput<'a> {
    root: [u8; ROOT_BYTES],
    address: [u8; ADDRESS_BYTES],
    slot: [u8; SLOT_BYTES],
    index: u64,
    value: &'a [u8],
    proof_nodes: [&'a [u8]; MAX_PROOF_NODES],
    proof_node_count: usize,
}

impl<'a> ProofInput<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let root = read_array::<ROOT_BYTES>(data, 0)?;
        let address = read_array::<ADDRESS_BYTES>(data, ROOT_BYTES)?;
        let slot = read_array::<SLOT_BYTES>(data, ROOT_BYTES + ADDRESS_BYTES)?;
        let index = u64::from_be_bytes(read_array::<INDEX_BYTES>(
            data,
            ROOT_BYTES + ADDRESS_BYTES + SLOT_BYTES,
        )?);
        let mut cursor = HEADER_BYTES;
        let value_len = usize::from(*data.get(cursor)?) % (MAX_VALUE_BYTES + 1);
        cursor = cursor.saturating_add(1);
        let value = read_slice(data, cursor, value_len)?;
        cursor = cursor.saturating_add(value_len);

        let mut proof_nodes = [&[][..]; MAX_PROOF_NODES];
        let requested_nodes = usize::from(*data.get(cursor).unwrap_or(&0)) % (MAX_PROOF_NODES + 1);
        cursor = cursor.saturating_add(1);
        let mut proof_node_count = 0usize;
        for slot in proof_nodes.iter_mut().take(requested_nodes) {
            let Some(length_byte) = data.get(cursor) else {
                break;
            };
            cursor = cursor.saturating_add(1);
            let node_len = usize::from(*length_byte) % (MAX_PROOF_NODE_BYTES + 1);
            let Some(node) = read_slice(data, cursor, node_len) else {
                break;
            };
            cursor = cursor.saturating_add(node_len);
            *slot = node;
            proof_node_count = proof_node_count.saturating_add(1);
        }

        Some(Self {
            root,
            address,
            slot,
            index,
            value,
            proof_nodes,
            proof_node_count,
        })
    }

    fn drive(&self, limits: DecodeLimits) {
        let root = B256::from_bytes(self.root);
        let nodes = self.proof_nodes();
        let _ = verify_transaction_inclusion(
            TransactionTrieRoot::from_b256(root),
            self.index,
            self.value,
            nodes,
            limits,
            TinyKeccak256::default,
        );
        let _ = verify_receipt_inclusion(
            ReceiptTrieRoot::from_b256(root),
            self.index,
            self.value,
            nodes,
            limits,
            TinyKeccak256::default,
        );
        let _ = verify_account_inclusion(
            AccountTrieRoot::from_b256(root),
            Address::from_bytes(self.address),
            self.value,
            nodes,
            limits,
            TinyKeccak256::default,
        );
        let _ = verify_storage_inclusion(
            StorageTrieRoot::from_b256(root),
            StorageSlotKey::from_b256(B256::from_bytes(self.slot)),
            self.value,
            nodes,
            limits,
            TinyKeccak256::default,
        );
    }

    fn proof_nodes(&self) -> &[&'a [u8]] {
        self.proof_nodes
            .get(..self.proof_node_count)
            .unwrap_or(&[])
    }
}

fn read_array<const N: usize>(data: &[u8], offset: usize) -> Option<[u8; N]> {
    read_slice(data, offset, N).and_then(|slice| slice.try_into().ok())
}

fn read_slice(data: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    data.get(offset..offset.checked_add(len)?)
}
