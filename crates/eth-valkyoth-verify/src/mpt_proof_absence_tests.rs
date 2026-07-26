extern crate std;

use std::vec::Vec;

use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};

use super::tests::{TestHasher, branch_node, leaf_node_from_nibbles, list, scalar, test_hash};
use super::*;
use crate::MPT_MAX_INLINE_REFERENCE_BYTES;

#[test]
fn empty_root_branch_terminal_is_absent() -> Result<(), &'static str> {
    let root_node = terminal_branch();
    assert_absent(&[], &[&root_node])
}

#[test]
fn empty_hashed_child_branch_terminal_is_absent() -> Result<(), &'static str> {
    let terminal = hashed_terminal_branch();
    if terminal.len() < MPT_MAX_INLINE_REFERENCE_BYTES {
        return Err("hashed terminal branch must reach the hash threshold");
    }
    let terminal_hash = test_hash(&terminal).to_bytes();
    let intermediate = branch_node(0, scalar(&terminal_hash), scalar(b""));
    let intermediate_hash = test_hash(&intermediate).to_bytes();
    let root = branch_node(0, scalar(&intermediate_hash), scalar(b""));

    assert_absent(&[0x00], &[&root, &intermediate, &terminal])
}

#[test]
fn empty_inline_child_branch_terminal_is_absent() -> Result<(), &'static str> {
    let terminal = terminal_branch();
    if terminal.len() >= MPT_MAX_INLINE_REFERENCE_BYTES {
        return Err("terminal branch must remain inline");
    }
    let even_extension_path = scalar(&[0x00, 0x00]);
    let root = list(&[even_extension_path, terminal]);

    assert_absent(&[0x00], &[&root])
}

fn assert_absent(key: &[u8], proof: &[&[u8]]) -> Result<(), &'static str> {
    let root = MptProofRoot::from_b256(test_hash(
        proof.first().copied().ok_or("proof root must exist")?,
    ));
    let mut session = test_session()?;

    let error =
        verify_key_inclusion_in_session(root, key, &[], proof, &mut session, TestHasher::default);

    assert_eq!(error, Err(MptProofVerificationError::Absent));
    Ok(())
}

fn terminal_branch() -> Vec<u8> {
    branch_node(0, leaf_node_from_nibbles(&[0], b"left"), scalar(b""))
}

fn hashed_terminal_branch() -> Vec<u8> {
    let child_hash = test_hash(b"hashed terminal child").to_bytes();
    branch_node(0, scalar(&child_hash), scalar(b""))
}

fn test_session() -> Result<DecodeSession, &'static str> {
    let limits = DecodeLimits {
        max_input_bytes: 4096,
        max_list_items: 64,
        max_nesting_depth: 16,
        max_total_allocation: 8192,
        max_proof_nodes: 8,
        max_total_items: 1024,
    };
    let policy = DecodeSessionPolicy::reviewed_policy(
        limits, 16_384, 1024, 8, 16_384, 16_384, 16_384, 65_536,
    )
    .map_err(|_| "test policy")?;
    DecodeSession::new(policy).map_err(|_| "test session")
}
