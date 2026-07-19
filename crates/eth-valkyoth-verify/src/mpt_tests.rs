extern crate std;

use std::vec;
use std::vec::Vec;

use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};

use super::{
    MPT_BRANCH_CHILD_COUNT, MptCompactPathKind, MptNode, MptNodeDecodeError,
    MptNodeDecodeErrorCategory, MptNodeField, MptNodeReference, decode_mpt_node,
    decode_mpt_node_in_session, decode_mpt_proof_nodes,
};

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 1024,
    max_list_items: 64,
    max_nesting_depth: 16,
    max_total_allocation: 4096,
    max_proof_nodes: 4,
    max_total_items: 256,
};

#[test]
fn shared_session_accounts_proof_parse_and_semantic_pass() -> Result<(), &'static str> {
    let items: Vec<Vec<u8>> = core::iter::repeat_with(empty).take(17).collect();
    let node = list(&items)?;
    let policy = DecodeSessionPolicy::reviewed_policy(TEST_LIMITS, 4096, 1024, 8, 4096, 16_384)
        .map_err(|_| "session policy must be valid")?;
    let mut session = DecodeSession::new(policy).map_err(|_| "session must initialize")?;

    decode_mpt_node_in_session(&node, &mut session)
        .map_err(|_| "MPT node must decode in one session")?;
    assert_eq!(session.proof_nodes(), 1);
    assert!(session.encoded_bytes() >= node.len() * 2);
    assert!(session.items() >= 36);
    assert_eq!(session.allocation_capacity(), 0);
    Ok(())
}

#[test]
fn decodes_branch_node_with_empty_children() -> Result<(), &'static str> {
    let items: Vec<Vec<u8>> = core::iter::repeat_with(empty).take(17).collect();
    let node = list(&items)?;
    let decoded = decode_ok(&node)?;
    let MptNode::Branch(branch) = decoded else {
        return Err("branch fixture must decode as branch");
    };

    assert_eq!(branch.children().count(), MPT_BRANCH_CHILD_COUNT);
    for child in branch.children() {
        assert_eq!(
            child.map_err(|_| "branch child must decode")?,
            MptNodeReference::Empty
        );
    }
    assert_eq!(branch.value(), &[] as &[u8]);
    Ok(())
}

#[test]
fn decodes_leaf_node_with_even_compact_path() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x20])?, scalar(b"dog")?])?;
    let decoded = decode_ok(&node)?;
    let MptNode::Leaf(leaf) = decoded else {
        return Err("leaf fixture must decode as leaf");
    };

    assert!(leaf.path.is_leaf());
    assert_eq!(leaf.path.raw(), &[0x20]);
    assert_eq!(leaf.path.nibble_count().map_err(|_| "nibbles")?, 0);
    assert_eq!(leaf.value, b"dog");
    Ok(())
}

#[test]
fn decodes_extension_node_with_hash_reference() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x00])?, scalar(&hash_bytes())?])?;
    let decoded = decode_ok(&node)?;
    let MptNode::Extension(extension) = decoded else {
        return Err("extension fixture must decode as extension");
    };

    assert_eq!(extension.path.kind, MptCompactPathKind::Extension);
    assert!(!extension.path.has_odd_nibbles());
    assert_eq!(extension.path.nibble_count().map_err(|_| "nibbles")?, 0);
    assert!(matches!(extension.child, MptNodeReference::Hash(_)));
    Ok(())
}

#[test]
fn decodes_inline_reference_without_recursive_budget_growth() -> Result<(), &'static str> {
    let inline = list(&[scalar(&[0x20])?, scalar(b"v")?])?;
    let parent = list(&[scalar(&[0x11])?, inline])?;
    let decoded = decode_ok(&parent)?;
    let MptNode::Extension(extension) = decoded else {
        return Err("parent fixture must decode as extension");
    };
    let MptNodeReference::Inline(inline) = extension.child else {
        return Err("child fixture must decode as inline");
    };

    assert!(matches!(
        inline.node().map_err(|_| "inline node must decode")?,
        MptNode::Leaf(_)
    ));
    Ok(())
}

#[test]
fn rejects_malformed_inline_reference_node() -> Result<(), &'static str> {
    let malformed_inline = list(&[empty()])?;
    let parent = list(&[scalar(&[0x11])?, malformed_inline])?;

    assert_eq!(
        decode_mpt_node(&parent, TEST_LIMITS),
        Err(MptNodeDecodeError::WrongFieldCount { found: 1 })
    );
    Ok(())
}

#[test]
fn rejects_inline_reference_at_hash_threshold() -> Result<(), &'static str> {
    let value: [u8; 29] = core::array::from_fn(|index| u8::try_from(index).unwrap_or(0));
    let oversized_inline = list(&[scalar(&[0x20])?, scalar(&value[..29])?])?;
    assert_eq!(oversized_inline.len(), 32);
    let parent = list(&[scalar(&[0x11])?, oversized_inline])?;

    assert_eq!(
        decode_mpt_node(&parent, TEST_LIMITS),
        Err(MptNodeDecodeError::InlineNodeTooLarge {
            field: MptNodeField::ExtensionChild,
            found: 32,
        })
    );
    Ok(())
}

#[test]
fn rejects_wrong_node_field_count() -> Result<(), &'static str> {
    let node = list(&[empty()])?;
    assert_eq!(
        decode_mpt_node(&node, TEST_LIMITS),
        Err(MptNodeDecodeError::WrongFieldCount { found: 1 })
    );
    Ok(())
}

#[test]
fn rejects_reserved_compact_path_flag() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x40])?, scalar(b"v")?])?;
    assert_eq!(
        decode_mpt_node(&node, TEST_LIMITS),
        Err(MptNodeDecodeError::InvalidCompactPathFlag { flag: 4 })
    );
    Ok(())
}

#[test]
fn rejects_nonzero_even_path_padding() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x01])?, scalar(&hash_bytes())?])?;
    assert_eq!(
        decode_mpt_node(&node, TEST_LIMITS),
        Err(MptNodeDecodeError::InvalidCompactPathPadding { found: 1 })
    );
    Ok(())
}

#[test]
fn rejects_empty_extension_child_reference() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x00])?, empty()])?;
    assert_eq!(
        decode_mpt_node(&node, TEST_LIMITS),
        Err(MptNodeDecodeError::EmptyNodeReference {
            field: MptNodeField::ExtensionChild
        })
    );
    Ok(())
}

#[test]
fn rejects_short_scalar_child_reference() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x00])?, scalar(&[1, 2])?])?;
    assert_eq!(
        decode_mpt_node(&node, TEST_LIMITS),
        Err(MptNodeDecodeError::InvalidNodeReferenceLength {
            field: MptNodeField::ExtensionChild,
            found: 2
        })
    );
    Ok(())
}

#[test]
fn proof_nodes_enforce_cumulative_count_and_bytes() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x20])?, scalar(b"v")?])?;
    let nodes = [&node[..], &node[..]];
    let limits = DecodeLimits {
        max_proof_nodes: 1,
        ..TEST_LIMITS
    };

    let error = decode_mpt_proof_nodes(&nodes, limits).map_err(|error| error.category());
    assert_eq!(error, Err(MptNodeDecodeErrorCategory::ResourceExhaustion));

    let limits = DecodeLimits {
        max_total_allocation: node.len(),
        ..TEST_LIMITS
    };
    let error = decode_mpt_proof_nodes(&nodes, limits).map_err(|error| error.category());
    assert_eq!(error, Err(MptNodeDecodeErrorCategory::ResourceExhaustion));
    Ok(())
}

#[test]
fn proof_nodes_return_checked_input_slice() -> Result<(), &'static str> {
    let node = list(&[scalar(&[0x20])?, scalar(b"v")?])?;
    let nodes = [&node[..]];
    let proof =
        decode_mpt_proof_nodes(&nodes, TEST_LIMITS).map_err(|_| "proof nodes must decode")?;

    assert_eq!(proof.len(), 1);
    assert!(!proof.is_empty());
    assert_eq!(proof.encoded_nodes(), &nodes);
    Ok(())
}

fn decode_ok(input: &[u8]) -> Result<MptNode<'_>, &'static str> {
    decode_mpt_node(input, TEST_LIMITS).map_err(|_| "MPT node fixture should decode")
}

fn hash_bytes() -> [u8; 32] {
    core::array::from_fn(|index| u8::try_from(index).unwrap_or(0).wrapping_add(1))
}

fn empty() -> Vec<u8> {
    vec![0x80]
}

fn scalar(payload: &[u8]) -> Result<Vec<u8>, &'static str> {
    if payload.is_empty() {
        return Ok(empty());
    }
    if payload.len() == 1 && payload.first().is_some_and(|byte| *byte <= 0x7f) {
        return Ok(payload.to_vec());
    }
    let len = u8::try_from(payload.len()).map_err(|_| "scalar fixture too large")?;
    let prefix = 0x80_u8.checked_add(len).ok_or("scalar length overflow")?;
    let mut out = Vec::with_capacity(payload.len().saturating_add(1));
    out.push(prefix);
    out.extend_from_slice(payload);
    Ok(out)
}

fn list(items: &[Vec<u8>]) -> Result<Vec<u8>, &'static str> {
    let payload_len = items
        .iter()
        .try_fold(0usize, |acc, item| acc.checked_add(item.len()))
        .ok_or("list payload overflow")?;
    let len = u8::try_from(payload_len).map_err(|_| "list fixture too large")?;
    let prefix = 0xc0_u8.checked_add(len).ok_or("list length overflow")?;
    let mut out = Vec::with_capacity(payload_len.saturating_add(1));
    out.push(prefix);
    for item in items {
        out.extend_from_slice(item);
    }
    Ok(out)
}
