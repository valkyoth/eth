use eth_valkyoth_codec::{DecodeSession, DecodeSessionCharges};

use crate::mpt::{
    MPT_MAX_INLINE_REFERENCE_BYTES, MptInlineNode, MptNode, MptNodeDecodeError, MptNodeReference,
    decode_mpt_node_body_in_session,
};

use super::{
    MAX_PROOF_WALK_DEPTH, MptProofVerificationError, compact_path_nibble, key_nibble,
    key_nibble_len, proof_resource_error,
};

/// Computes a conservative, noncryptographic plan for the remaining walk.
pub(super) fn plan_remaining_work(
    key: &[u8],
    expected_value: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
) -> Result<DecodeSessionCharges, MptProofVerificationError> {
    let mut future = DecodeSession::new(session.policy()).map_err(proof_resource_error)?;
    let mut cursor = PlanningCursor::new(proof_nodes);
    let Some(first) = cursor.next_node(false, false, session, &mut future)? else {
        return Ok(future.charges());
    };
    plan_walk(
        first,
        key,
        expected_value,
        &mut cursor,
        session,
        &mut future,
    )?;
    Ok(future.charges())
}

fn plan_walk<'a>(
    mut node: MptNode<'a>,
    key: &[u8],
    expected_value: &[u8],
    cursor: &mut PlanningCursor<'a>,
    session: &mut DecodeSession,
    future: &mut DecodeSession,
) -> Result<(), MptProofVerificationError> {
    let mut key_offset = 0usize;
    let mut depth = 0usize;

    loop {
        depth = depth
            .checked_add(1)
            .ok_or(MptProofVerificationError::ProofTooDeep)?;
        if depth > MAX_PROOF_WALK_DEPTH {
            return Ok(());
        }

        let reference = match node {
            MptNode::Branch(branch) => {
                if key_offset == key_nibble_len(key) {
                    plan_value_comparison(branch.value(), expected_value, session, future)?;
                    return Ok(());
                }
                let child_index = key_nibble(key, key_offset)?;
                key_offset = key_offset.saturating_add(1);
                match branch.children().nth(usize::from(child_index)) {
                    Some(Ok(reference)) => reference,
                    Some(Err(error)) => {
                        return Err(MptProofVerificationError::MalformedNode(error));
                    }
                    None => return Ok(()),
                }
            }
            MptNode::Extension(extension) => {
                let Some(consumed) =
                    plan_compact_path(extension.path, key, key_offset, session, future)?
                else {
                    return Ok(());
                };
                key_offset = key_offset.saturating_add(consumed);
                extension.child
            }
            MptNode::Leaf(leaf) => {
                let Some(consumed) =
                    plan_compact_path(leaf.path, key, key_offset, session, future)?
                else {
                    return Ok(());
                };
                if key_offset.saturating_add(consumed) == key_nibble_len(key) {
                    plan_value_comparison(leaf.value, expected_value, session, future)?;
                }
                return Ok(());
            }
        };

        node = match reference {
            MptNodeReference::Empty => return Ok(()),
            MptNodeReference::Hash(_) => {
                let require_branch = matches!(node, MptNode::Extension(_));
                let Some(next) = cursor.next_node(true, require_branch, session, future)? else {
                    return Ok(());
                };
                next
            }
            MptNodeReference::Inline(inline) => plan_inline_node(inline, session, future)?,
        };
    }
}

fn plan_compact_path(
    path: crate::mpt::MptCompactPath<'_>,
    key: &[u8],
    key_offset: usize,
    session: &mut DecodeSession,
    future: &mut DecodeSession,
) -> Result<Option<usize>, MptProofVerificationError> {
    if !path.is_leaf() && key_offset == key_nibble_len(key) {
        return Ok(None);
    }
    let count = path
        .nibble_count()
        .map_err(MptProofVerificationError::MalformedNode)?;
    session
        .account_nibbles(count)
        .map_err(proof_resource_error)?;
    future
        .account_nibbles(count)
        .map_err(proof_resource_error)?;
    if key_offset.saturating_add(count) > key_nibble_len(key) {
        return Ok(None);
    }
    for index in 0..count {
        let key_index = key_offset
            .checked_add(index)
            .ok_or_else(|| proof_resource_error(eth_valkyoth_codec::DecodeError::WorkExceeded))?;
        if compact_path_nibble(path, index)? != key_nibble(key, key_index)? {
            return Ok(None);
        }
    }
    Ok(Some(count))
}

fn plan_value_comparison(
    found: &[u8],
    expected: &[u8],
    session: &mut DecodeSession,
    future: &mut DecodeSession,
) -> Result<(), MptProofVerificationError> {
    let compared = found
        .len()
        .checked_add(expected.len())
        .ok_or_else(|| proof_resource_error(eth_valkyoth_codec::DecodeError::ValueBytesExceeded))?;
    session
        .account_value_bytes(compared)
        .map_err(proof_resource_error)?;
    future
        .account_value_bytes(compared)
        .map_err(proof_resource_error)
}

fn plan_inline_node<'a>(
    inline: MptInlineNode<'a>,
    session: &mut DecodeSession,
    future: &mut DecodeSession,
) -> Result<MptNode<'a>, MptProofVerificationError> {
    let mut decoded = None;
    let replay = session
        .measure_replay_charges(|session| {
            decoded = Some(inline.node_in_session(session));
        })
        .map_err(proof_resource_error)?;
    future
        .account_charges(replay)
        .map_err(proof_resource_error)?;
    match decoded {
        Some(result) => result.map_err(MptProofVerificationError::MalformedNode),
        None => Err(proof_resource_error(
            eth_valkyoth_codec::DecodeError::Malformed,
        )),
    }
}

struct PlanningCursor<'a> {
    nodes: &'a [&'a [u8]],
    index: usize,
}

impl<'a> PlanningCursor<'a> {
    const fn new(nodes: &'a [&'a [u8]]) -> Self {
        Self { nodes, index: 0 }
    }

    fn next_node(
        &mut self,
        is_child: bool,
        require_branch: bool,
        session: &mut DecodeSession,
        future: &mut DecodeSession,
    ) -> Result<Option<MptNode<'a>>, MptProofVerificationError> {
        let Some(encoded) = self.nodes.get(self.index).copied() else {
            return Ok(None);
        };
        if is_child && encoded.len() < MPT_MAX_INLINE_REFERENCE_BYTES {
            return Err(MptProofVerificationError::MalformedNode(
                MptNodeDecodeError::HashedNodeTooShort {
                    found: encoded.len(),
                },
            ));
        }
        let mut decoded = None;
        let replay = session
            .measure_replay_charges(|session| {
                decoded = Some(decode_mpt_node_body_in_session(encoded, session));
            })
            .map_err(proof_resource_error)?;
        future
            .account_charges(replay)
            .map_err(proof_resource_error)?;
        let node = match decoded {
            Some(result) => result.map_err(MptProofVerificationError::MalformedNode)?,
            None => {
                return Err(proof_resource_error(
                    eth_valkyoth_codec::DecodeError::Malformed,
                ));
            }
        };
        if require_branch && !matches!(node, MptNode::Branch(_)) {
            return Err(MptProofVerificationError::MalformedNode(
                MptNodeDecodeError::NonCanonicalExtensionChild,
            ));
        }
        future
            .account_hashes(1, encoded.len())
            .map_err(proof_resource_error)?;
        self.index = self.index.saturating_add(1);
        Ok(Some(node))
    }
}
