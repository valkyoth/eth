use eth_valkyoth_codec::{DecodeError, DecodeSession, RlpItem, RlpList, RlpScalar};

use super::*;

/// Decodes one canonical MPT node through a shared work session.
pub fn decode_mpt_node_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    session
        .account_proof_nodes(1)
        .map_err(|source| field_error(MptNodeField::ProofNode, source))?;
    let list = decode_rlp_list_partial_in_session(input, session)
        .map_err(|source| field_error(MptNodeField::Node, source))?;
    require_exact_consumption(list.encoded_len(), input.len())
        .map_err(|source| field_error(MptNodeField::Node, source))?;
    decode_mpt_node_from_list_in_session(list, MPT_INLINE_REFERENCE_DEPTH_LIMIT, session)
}

/// Validates proof-node syntax through one cumulative work session.
pub fn decode_mpt_proof_nodes_in_session<'a>(
    encoded_nodes: &'a [&'a [u8]],
    session: &mut DecodeSession,
) -> Result<MptProofNodes<'a>, MptNodeDecodeError> {
    for node in encoded_nodes {
        let _ = decode_mpt_node_in_session(node, session)?;
    }
    Ok(MptProofNodes { encoded_nodes })
}

pub(super) fn decode_mpt_node_from_list_in_session<'a>(
    list: RlpList<'a>,
    depth_remaining: usize,
    session: &mut DecodeSession,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    match list.item_count() {
        MPT_BRANCH_NODE_FIELD_COUNT => decode_branch(list, depth_remaining, session),
        MPT_COMPACT_NODE_FIELD_COUNT => decode_compact(list, depth_remaining, session),
        found => Err(MptNodeDecodeError::WrongFieldCount { found }),
    }
}

fn decode_branch<'a>(
    list: RlpList<'a>,
    depth_remaining: usize,
    session: &mut DecodeSession,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    let mut items = list.items();
    let mut children = [MptNodeReference::Empty; MPT_BRANCH_CHILD_COUNT];
    for slot in &mut children {
        let item = items.next_in_session(session).ok_or(field_error(
            MptNodeField::BranchChild,
            DecodeError::Malformed,
        ))?;
        *slot = decode_reference(
            item,
            MptNodeField::BranchChild,
            true,
            depth_remaining,
            session,
        )?;
    }
    let value = next_scalar(&mut items, MptNodeField::BranchValue, session)?.payload();
    Ok(MptNode::Branch(MptBranchNode { children, value }))
}

fn decode_compact<'a>(
    list: RlpList<'a>,
    depth_remaining: usize,
    session: &mut DecodeSession,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    let mut fields = list.items();
    let path = decode_compact_path(
        next_scalar(&mut fields, MptNodeField::CompactPath, session)?.payload(),
    )?;
    if path.is_leaf() {
        let value = next_scalar(&mut fields, MptNodeField::LeafValue, session)?.payload();
        Ok(MptNode::Leaf(MptLeafNode { path, value }))
    } else {
        let item = fields.next_in_session(session).ok_or(field_error(
            MptNodeField::ExtensionChild,
            DecodeError::Malformed,
        ))?;
        let child = decode_reference(
            item,
            MptNodeField::ExtensionChild,
            false,
            depth_remaining,
            session,
        )?;
        Ok(MptNode::Extension(MptExtensionNode { path, child }))
    }
}

fn decode_reference<'a>(
    item: Result<RlpItem<'a>, DecodeError>,
    field: MptNodeField,
    empty_allowed: bool,
    depth_remaining: usize,
    session: &mut DecodeSession,
) -> Result<MptNodeReference<'a>, MptNodeDecodeError> {
    match item.map_err(|source| field_error(field, source))? {
        RlpItem::Scalar(scalar) => decode_scalar_reference(scalar, field, empty_allowed),
        RlpItem::List(list) => {
            let found = list.encoded_len();
            if found >= MPT_MAX_INLINE_REFERENCE_BYTES {
                return Err(MptNodeDecodeError::InlineNodeTooLarge { field, found });
            }
            let next_depth = depth_remaining
                .checked_sub(1)
                .ok_or(MptNodeDecodeError::InlineNodeTooDeep)?;
            let _ = decode_mpt_node_from_list_in_session(list, next_depth, session)?;
            Ok(MptNodeReference::Inline(MptInlineNode {
                list,
                depth_remaining: next_depth,
            }))
        }
    }
}

fn next_scalar<'a>(
    fields: &mut eth_valkyoth_codec::RlpListItems<'a>,
    field: MptNodeField,
    session: &mut DecodeSession,
) -> Result<RlpScalar<'a>, MptNodeDecodeError> {
    let item = fields
        .next_in_session(session)
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(field_error(field, DecodeError::UnexpectedList)),
    }
}
