extern crate std;

use core::sync::atomic::{AtomicUsize, Ordering};
use std::{vec, vec::Vec};

use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};
use eth_valkyoth_hash::{KECCAK256_EMPTY, Keccak256, Keccak256Digest, TinyKeccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256, Wei};

use super::*;

const LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 2048,
    max_list_items: 64,
    max_nesting_depth: 16,
    max_total_allocation: 4096,
    max_proof_nodes: 8,
    max_total_items: 256,
};

#[test]
fn composes_account_and_storage_proofs() -> Result<(), &'static str> {
    let slot = test_slot(3);
    let storage = storage_leaf(slot, &[0x2a]);
    let storage_root = StorageTrieRoot::from_b256(hash(&storage));
    let fixture = account_leaf(storage_root, canonical_account(storage_root));

    let account = verify_account_proof(
        fixture.root,
        fixture.address,
        &[&fixture.node],
        LIMITS,
        TinyKeccak256::default,
    )
    .map_err(|_| "account proof")?;
    let state = account.account().ok_or("present account")?;
    assert_eq!(state.nonce().get(), 7);
    assert_eq!(state.balance(), Wei::from_u128(85));
    assert_eq!(state.storage_root(), storage_root);
    assert_eq!(state.code_hash(), B256::from_bytes(KECCAK256_EMPTY));

    let value = verify_account_storage(&account, slot, &[&storage], LIMITS, TinyKeccak256::default)
        .map_err(|_| "storage proof")?;
    assert!(value.is_present());
    assert_eq!(value.value(), Wei::from_u128(42));
    Ok(())
}

#[test]
fn rejects_storage_proof_from_an_unrelated_account_root() -> Result<(), &'static str> {
    let slot = test_slot(4);
    let authorized_storage = storage_leaf(slot, &[0x11]);
    let authorized_root = StorageTrieRoot::from_b256(hash(&authorized_storage));
    let account_fixture = account_leaf(authorized_root, canonical_account(authorized_root));
    let account = verify_account_proof(
        account_fixture.root,
        account_fixture.address,
        &[&account_fixture.node],
        LIMITS,
        TinyKeccak256::default,
    )
    .map_err(|_| "account proof")?;

    let substituted_storage = storage_leaf(slot, &[0x22]);
    let error = verify_account_storage(
        &account,
        slot,
        &[&substituted_storage],
        LIMITS,
        TinyKeccak256::default,
    );
    assert_eq!(
        error,
        Err(StateProofVerificationError::Proof(
            MptProofVerificationError::WrongRoot
        ))
    );
    Ok(())
}

#[test]
fn proves_account_and_storage_absence_as_zero() -> Result<(), &'static str> {
    let address = test_address(9);
    let empty_root = AccountTrieRoot::from_b256(StorageTrieRoot::EMPTY.to_b256());
    let account = verify_account_proof(empty_root, address, &[], LIMITS, TinyKeccak256::default)
        .map_err(|_| "empty account proof")?;
    assert!(account.is_absent());
    assert_eq!(account.storage_root(), StorageTrieRoot::EMPTY);

    let value = verify_account_storage(&account, test_slot(9), &[], LIMITS, TinyKeccak256::default)
        .map_err(|_| "empty storage proof")?;
    assert!(!value.is_present());
    assert_eq!(value.value(), Wei::ZERO);
    Ok(())
}

#[test]
fn path_divergence_is_verified_storage_zero() -> Result<(), &'static str> {
    let requested = test_slot(12);
    let other = test_slot(13);
    let storage = storage_leaf(other, &[0x33]);
    let storage_root = StorageTrieRoot::from_b256(hash(&storage));
    let account_fixture = account_leaf(storage_root, canonical_account(storage_root));
    let account = verify_account_proof(
        account_fixture.root,
        account_fixture.address,
        &[&account_fixture.node],
        LIMITS,
        TinyKeccak256::default,
    )
    .map_err(|_| "account proof")?;

    let value = verify_account_storage(
        &account,
        requested,
        &[&storage],
        LIMITS,
        TinyKeccak256::default,
    )
    .map_err(|_| "absence proof")?;
    assert!(!value.is_present());
    assert_eq!(value.value(), Wei::ZERO);
    Ok(())
}

#[test]
fn rejects_explicit_zero_storage_entry() -> Result<(), &'static str> {
    let slot = test_slot(15);
    let storage = storage_leaf(slot, &[]);
    let storage_root = StorageTrieRoot::from_b256(hash(&storage));
    let account_fixture = account_leaf(storage_root, canonical_account(storage_root));
    let account = verify_account_proof(
        account_fixture.root,
        account_fixture.address,
        &[&account_fixture.node],
        LIMITS,
        TinyKeccak256::default,
    )
    .map_err(|_| "account proof")?;

    let error = verify_account_storage(&account, slot, &[&storage], LIMITS, TinyKeccak256::default);
    assert_eq!(
        error,
        Err(StateProofVerificationError::ExplicitZeroStorageValue)
    );
    Ok(())
}

#[test]
fn rejects_malformed_account_before_proof_node_hashing() -> Result<(), &'static str> {
    HASH_CALLS.store(0, Ordering::Relaxed);
    let storage_root = StorageTrieRoot::EMPTY;
    let malformed = list(&[
        scalar(&[]),
        scalar(&[]),
        scalar(&storage_root.to_b256().to_bytes()),
    ]);
    let fixture = account_leaf(storage_root, malformed);
    let mut session = test_session()?;

    let error = verify_account_proof_in_session(
        fixture.root,
        fixture.address,
        &[&fixture.node],
        &mut session,
        CountingHasher::default,
    );
    assert_eq!(
        error,
        Err(StateProofVerificationError::Account(
            AccountDecodeError::FieldCount { found: 3 }
        ))
    );
    assert_eq!(HASH_CALLS.load(Ordering::Relaxed), 1);
    Ok(())
}

#[test]
fn rejects_noncanonical_account_integer_and_hash_width() {
    let storage_root = StorageTrieRoot::EMPTY;
    let bad_nonce = list(&[
        scalar(&[0, 1]),
        scalar(&[]),
        scalar(&storage_root.to_b256().to_bytes()),
        scalar(&KECCAK256_EMPTY),
    ]);
    let fixture = account_leaf(storage_root, bad_nonce);
    assert!(matches!(
        verify_account_proof(
            fixture.root,
            fixture.address,
            &[&fixture.node],
            LIMITS,
            TinyKeccak256::default,
        ),
        Err(StateProofVerificationError::Account(_))
    ));

    let bad_hash = list(&[
        scalar(&[]),
        scalar(&[]),
        scalar(&[1_u8; 31]),
        scalar(&KECCAK256_EMPTY),
    ]);
    let fixture = account_leaf(storage_root, bad_hash);
    assert_eq!(
        verify_account_proof(
            fixture.root,
            fixture.address,
            &[&fixture.node],
            LIMITS,
            TinyKeccak256::default,
        ),
        Err(StateProofVerificationError::Account(
            AccountDecodeError::FixedWidth {
                field: AccountField::StorageRoot,
                expected: 32,
                found: 31,
            }
        ))
    );
}

fn canonical_account(storage_root: StorageTrieRoot) -> Vec<u8> {
    list(&[
        scalar(&[7]),
        scalar(&[85]),
        scalar(&storage_root.to_b256().to_bytes()),
        scalar(&KECCAK256_EMPTY),
    ])
}

struct AccountFixture {
    address: Address,
    root: AccountTrieRoot,
    node: Vec<u8>,
}

fn account_leaf(storage_root: StorageTrieRoot, account: Vec<u8>) -> AccountFixture {
    let seed = storage_root.to_b256().to_bytes()[0];
    let address = test_address(seed);
    let key = hash(&address.to_bytes()).to_bytes();
    let node = leaf_node(&key, &account);
    AccountFixture {
        address,
        root: AccountTrieRoot::from_b256(hash(&node)),
        node,
    }
}

fn storage_leaf(slot: StorageSlotKey, integer_payload: &[u8]) -> Vec<u8> {
    let key = hash(&slot.to_b256().to_bytes()).to_bytes();
    leaf_node(&key, &scalar(integer_payload))
}

fn leaf_node(key: &[u8], value: &[u8]) -> Vec<u8> {
    list(&[scalar(&compact_path_leaf(key)), scalar(value)])
}

fn compact_path_leaf(path: &[u8]) -> Vec<u8> {
    let mut output = vec![0x20];
    output.extend_from_slice(path);
    output
}

fn test_address(seed: u8) -> Address {
    Address::from_bytes(core::array::from_fn(|index| {
        seed.wrapping_add(u8::try_from(index).unwrap_or(u8::MAX))
    }))
}

fn test_slot(seed: u8) -> StorageSlotKey {
    StorageSlotKey::from_b256(B256::from_bytes(core::array::from_fn(|index| {
        seed.wrapping_add(u8::try_from(index).unwrap_or(u8::MAX))
    })))
}

fn hash(input: &[u8]) -> B256 {
    hash_one(TinyKeccak256::default(), input)
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

fn test_session() -> Result<DecodeSession, &'static str> {
    let policy =
        DecodeSessionPolicy::reviewed_policy(LIMITS, 8192, 1024, 16, 8192, 8192, 8192, 65_536)
            .map_err(|_| "test policy")?;
    DecodeSession::new(policy).map_err(|_| "test session")
}

static HASH_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
struct CountingHasher(TinyKeccak256);

impl Keccak256 for CountingHasher {
    fn update(&mut self, input: &[u8]) {
        self.0.update(input);
    }

    fn finalize(self) -> Keccak256Digest {
        HASH_CALLS.fetch_add(1, Ordering::Relaxed);
        self.0.finalize()
    }
}
