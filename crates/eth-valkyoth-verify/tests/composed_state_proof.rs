#![cfg(feature = "json")]
//! Official Execution APIs Hive fixture for composed EIP-1186 verification.

use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};
use eth_valkyoth_hash::{TinyKeccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256, Wei};
use eth_valkyoth_verify::{
    AccountTrieRoot, StorageSlotKey, verify_account_proof_in_session,
    verify_account_storage_in_session,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/execution_apis_get_account_proof_with_storage.io");

#[test]
fn verifies_execution_apis_hive_account_and_storage_proof() -> Result<(), String> {
    let response = FIXTURE
        .lines()
        .find_map(|line| line.strip_prefix("<< "))
        .ok_or_else(|| String::from("fixture response"))?;
    let document: Value = serde_json::from_str(response).map_err(|error| error.to_string())?;
    let result = document
        .get("result")
        .ok_or_else(|| String::from("result"))?;
    let address = Address::from_bytes(hex_array(
        result
            .get("address")
            .and_then(Value::as_str)
            .ok_or_else(|| String::from("address"))?,
    )?);
    let account_nodes = proof_nodes(
        result
            .get("accountProof")
            .and_then(Value::as_array)
            .ok_or_else(|| String::from("account proof"))?,
    )?;
    let account_refs = node_refs(&account_nodes);
    // Test-only fixture setup: this fixture omits its independently trusted
    // header root. Production callers must never derive the root from the
    // untrusted proof nodes that the root is supposed to authenticate.
    let state_root = AccountTrieRoot::from_b256(hash_one(
        TinyKeccak256::default(),
        account_refs
            .first()
            .copied()
            .ok_or_else(|| String::from("account root node"))?,
    ));
    let mut session = fixture_session()?;
    let account = verify_account_proof_in_session(
        state_root,
        address,
        &account_refs,
        &mut session,
        TinyKeccak256::default,
    )
    .map_err(|error| error.to_string())?;
    let decoded = account
        .account()
        .ok_or_else(|| String::from("account unexpectedly absent"))?;

    assert_eq!(decoded.nonce().get(), 0);
    assert_eq!(decoded.balance(), Wei::from_u128(0x56));
    assert_eq!(
        decoded.storage_root().to_b256(),
        B256::from_bytes(hex_array(
            result
                .get("storageHash")
                .and_then(Value::as_str)
                .ok_or_else(|| String::from("storage hash"))?,
        )?)
    );
    assert_eq!(
        decoded.code_hash(),
        B256::from_bytes(hex_array(
            result
                .get("codeHash")
                .and_then(Value::as_str)
                .ok_or_else(|| String::from("code hash"))?,
        )?)
    );

    let storage = result
        .get("storageProof")
        .and_then(Value::as_array)
        .and_then(|entries| entries.first())
        .ok_or_else(|| String::from("storage proof entry"))?;
    let storage_nodes = proof_nodes(
        storage
            .get("proof")
            .and_then(Value::as_array)
            .ok_or_else(|| String::from("storage proof"))?,
    )?;
    let storage_refs = node_refs(&storage_nodes);
    let verified = verify_account_storage_in_session(
        &account,
        StorageSlotKey::from_b256(B256::from_bytes(hex_quantity_array(
            storage
                .get("key")
                .and_then(Value::as_str)
                .ok_or_else(|| String::from("storage key"))?,
        )?)),
        &storage_refs,
        &mut session,
        TinyKeccak256::default,
    )
    .map_err(|error| error.to_string())?;

    assert!(verified.is_present());
    assert_eq!(verified.value(), Wei::from_u128(0x2a));
    Ok(())
}

fn fixture_session() -> Result<DecodeSession, String> {
    let limits = DecodeLimits {
        max_input_bytes: 4096,
        max_list_items: 64,
        max_nesting_depth: 16,
        max_total_allocation: 16_384,
        max_proof_nodes: 8,
        max_total_items: 4096,
    };
    let policy =
        DecodeSessionPolicy::reviewed_policy(limits, 32_768, 4096, 16, 16_384, 8192, 8192, 131_072)
            .map_err(|error| error.to_string())?;
    DecodeSession::new(policy).map_err(|error| error.to_string())
}

fn proof_nodes(values: &[Value]) -> Result<Vec<Vec<u8>>, String> {
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| String::from("proof node string"))
                .and_then(hex_bytes)
        })
        .collect()
}

fn node_refs(nodes: &[Vec<u8>]) -> Vec<&[u8]> {
    nodes.iter().map(Vec::as_slice).collect()
}

fn hex_array<const N: usize>(value: &str) -> Result<[u8; N], String> {
    hex_bytes(value)?
        .try_into()
        .map_err(|_| String::from("hex value has wrong width"))
}

fn hex_quantity_array<const N: usize>(value: &str) -> Result<[u8; N], String> {
    let digits = value
        .strip_prefix("0x")
        .ok_or_else(|| String::from("missing hex prefix"))?;
    if digits.is_empty() || digits.len() > N.saturating_mul(2) {
        return Err(String::from("hex quantity has wrong width"));
    }
    let mut padded = String::with_capacity(digits.len().saturating_add(1));
    if !digits.len().is_multiple_of(2) {
        padded.push('0');
    }
    padded.push_str(digits);
    let decoded = hex_bytes(&format!("0x{padded}"))?;
    let mut output = Vec::with_capacity(N);
    output.resize(N.saturating_sub(decoded.len()), u8::MIN);
    output.extend_from_slice(&decoded);
    output
        .try_into()
        .map_err(|_| String::from("hex quantity has wrong width"))
}

fn hex_bytes(value: &str) -> Result<Vec<u8>, String> {
    let digits = value
        .strip_prefix("0x")
        .ok_or_else(|| String::from("missing hex prefix"))?;
    if !digits.len().is_multiple_of(2) {
        return Err(String::from("odd hex length"));
    }
    digits
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(
                *pair
                    .first()
                    .ok_or_else(|| String::from("missing high hex digit"))?,
            )?;
            let low = hex_nibble(
                *pair
                    .get(1)
                    .ok_or_else(|| String::from("missing low hex digit"))?,
            )?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte.saturating_sub(b'0')),
        b'a'..=b'f' => Ok(byte.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Ok(byte.saturating_sub(b'A').saturating_add(10)),
        _ => Err(String::from("invalid hex digit")),
    }
}
