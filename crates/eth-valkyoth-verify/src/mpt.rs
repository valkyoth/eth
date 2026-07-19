use eth_valkyoth_codec::{
    DecodeAccumulator, DecodeError, DecodeLimits, DecodeSession, RlpItem, RlpList, RlpScalar,
    decode_rlp_list_partial, decode_rlp_list_partial_in_session, require_exact_consumption,
};
use eth_valkyoth_primitives::B256;

mod error;
mod session;
pub use error::{MptNodeDecodeError, MptNodeDecodeErrorCategory, MptNodeField};
pub(crate) use session::decode_mpt_node_body_in_session;
pub use session::{decode_mpt_node_in_session, decode_mpt_proof_nodes_in_session};

/// Number of child references in an MPT branch node.
pub const MPT_BRANCH_CHILD_COUNT: usize = 16;
/// Number of fields in an MPT branch node.
pub const MPT_BRANCH_NODE_FIELD_COUNT: usize = 17;
/// Number of fields in an MPT extension or leaf node.
pub const MPT_COMPACT_NODE_FIELD_COUNT: usize = 2;
/// Number of bytes in a hashed MPT child reference.
pub const MPT_HASH_REFERENCE_BYTES: usize = 32;
/// Maximum encoded byte length for an inline child reference.
pub const MPT_MAX_INLINE_REFERENCE_BYTES: usize = 32;
/// Maximum inline child-node depth validated by the MPT decoder.
pub const MPT_INLINE_REFERENCE_DEPTH_LIMIT: usize = 64;

/// Borrowed syntactic MPT node.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptNode<'a> {
    /// Branch node with sixteen child references plus one value slot.
    Branch(MptBranchNode<'a>),
    /// Extension node with a compact path and required child reference.
    Extension(MptExtensionNode<'a>),
    /// Leaf node with a compact path and scalar value.
    Leaf(MptLeafNode<'a>),
}

/// Borrowed MPT branch node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptBranchNode<'a> {
    children: [MptNodeReference<'a>; MPT_BRANCH_CHILD_COUNT],
    value: &'a [u8],
}

impl<'a> MptBranchNode<'a> {
    /// Returns an iterator over the sixteen child references.
    #[must_use]
    pub fn children(self) -> MptBranchChildren<'a> {
        MptBranchChildren {
            children: self.children.into_iter(),
        }
    }

    /// Returns the branch value slot as scalar bytes.
    #[must_use]
    pub const fn value(self) -> &'a [u8] {
        self.value
    }
}

/// Iterator over MPT branch child references.
#[derive(Clone, Debug)]
pub struct MptBranchChildren<'a> {
    children: core::array::IntoIter<MptNodeReference<'a>, MPT_BRANCH_CHILD_COUNT>,
}

impl<'a> Iterator for MptBranchChildren<'a> {
    type Item = Result<MptNodeReference<'a>, MptNodeDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.children.next().map(Ok)
    }
}

impl core::iter::FusedIterator for MptBranchChildren<'_> {}

/// Borrowed MPT extension node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptExtensionNode<'a> {
    /// Hex-prefix encoded extension path.
    pub path: MptCompactPath<'a>,
    /// Child reference reached after the extension path.
    pub child: MptNodeReference<'a>,
}

/// Borrowed MPT leaf node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptLeafNode<'a> {
    /// Hex-prefix encoded terminal path.
    pub path: MptCompactPath<'a>,
    /// Scalar trie value stored at the terminal path.
    pub value: &'a [u8],
}

/// Borrowed compact hex-prefix MPT path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptCompactPath<'a> {
    raw: &'a [u8],
    kind: MptCompactPathKind,
    odd_nibbles: bool,
}

impl<'a> MptCompactPath<'a> {
    /// Returns the raw compact-path bytes.
    #[must_use]
    pub const fn raw(self) -> &'a [u8] {
        self.raw
    }

    /// Returns true when this compact path marks a leaf node.
    #[must_use]
    pub const fn is_leaf(self) -> bool {
        matches!(self.kind, MptCompactPathKind::Leaf)
    }

    /// Returns true when the first path nibble is carried in the flag byte.
    #[must_use]
    pub const fn has_odd_nibbles(self) -> bool {
        self.odd_nibbles
    }

    /// Returns the decoded path nibble count.
    pub fn nibble_count(self) -> Result<usize, MptNodeDecodeError> {
        let doubled = self
            .raw
            .len()
            .checked_mul(2)
            .ok_or(MptNodeDecodeError::LengthOverflow)?;
        if self.odd_nibbles {
            doubled
                .checked_sub(1)
                .ok_or(MptNodeDecodeError::LengthOverflow)
        } else {
            doubled
                .checked_sub(2)
                .ok_or(MptNodeDecodeError::LengthOverflow)
        }
    }
}

/// Compact-path node kind encoded in the hex-prefix flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptCompactPathKind {
    /// Extension path.
    Extension,
    /// Leaf path.
    Leaf,
}

/// Borrowed MPT child reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptNodeReference<'a> {
    /// Empty branch child slot.
    Empty,
    /// Hashed child reference.
    Hash(B256),
    /// Inline child node represented as a borrowed RLP list.
    Inline(MptInlineNode<'a>),
}

/// Borrowed inline MPT child node reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptInlineNode<'a> {
    list: RlpList<'a>,
    depth_remaining: usize,
}

impl<'a> MptInlineNode<'a> {
    /// Decodes the inline node list into its syntactic node representation.
    ///
    /// Use [`Self::node_in_session`] when this reparse belongs to an untrusted
    /// operation governed by a shared decode session.
    pub fn node(self) -> Result<MptNode<'a>, MptNodeDecodeError> {
        decode_mpt_node_from_list(self.list, self.depth_remaining)
    }

    /// Decodes the inline node while charging its semantic reparse.
    pub fn node_in_session(
        self,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptNodeDecodeError> {
        session::decode_mpt_node_from_list_in_session(self.list, self.depth_remaining, session)
    }
}

/// Borrowed list of already shape-checked encoded proof nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptProofNodes<'a> {
    encoded_nodes: &'a [&'a [u8]],
}

impl<'a> MptProofNodes<'a> {
    /// Returns the encoded proof-node inputs that were checked.
    #[must_use]
    pub const fn encoded_nodes(self) -> &'a [&'a [u8]] {
        self.encoded_nodes
    }

    /// Returns the proof-node count.
    #[must_use]
    pub const fn len(self) -> usize {
        self.encoded_nodes.len()
    }

    /// Returns true when the proof-node list is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.encoded_nodes.is_empty()
    }
}

/// Decodes one canonical RLP MPT node under explicit limits.
pub fn decode_mpt_node<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    let mut accumulator = limits.accumulator();
    decode_mpt_node_with_accumulator(input, &mut accumulator)
}

/// Validates a borrowed list of encoded MPT proof nodes under cumulative
/// proof-node, byte, item, and nesting limits.
pub fn decode_mpt_proof_nodes<'a>(
    encoded_nodes: &'a [&'a [u8]],
    limits: DecodeLimits,
) -> Result<MptProofNodes<'a>, MptNodeDecodeError> {
    let mut accumulator = limits.accumulator();
    for node in encoded_nodes {
        let _ = decode_mpt_node_with_accumulator(node, &mut accumulator)?;
    }
    Ok(MptProofNodes { encoded_nodes })
}

pub(crate) fn decode_mpt_node_with_accumulator<'a>(
    input: &'a [u8],
    accumulator: &mut DecodeAccumulator,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    accumulator
        .account_proof_nodes(1)
        .map_err(|source| field_error(MptNodeField::ProofNode, source))?;
    accumulator
        .check_allocation(input.len())
        .map_err(|source| field_error(MptNodeField::ProofNode, source))?;
    let list = decode_rlp_list_partial(input, accumulator)
        .map_err(|source| field_error(MptNodeField::Node, source))?;
    require_exact_consumption(list.encoded_len(), input.len())
        .map_err(|source| field_error(MptNodeField::Node, source))?;
    decode_mpt_node_from_list(list, MPT_INLINE_REFERENCE_DEPTH_LIMIT)
}

fn decode_mpt_node_from_list<'a>(
    list: RlpList<'a>,
    depth_remaining: usize,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    match list.item_count() {
        MPT_BRANCH_NODE_FIELD_COUNT => decode_branch_node(list, depth_remaining),
        MPT_COMPACT_NODE_FIELD_COUNT => decode_compact_node(list, depth_remaining),
        found => Err(MptNodeDecodeError::WrongFieldCount { found }),
    }
}

fn decode_branch_node<'a>(
    list: RlpList<'a>,
    depth_remaining: usize,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    let mut items = list.items();
    let mut children = [MptNodeReference::Empty; MPT_BRANCH_CHILD_COUNT];
    for slot in &mut children {
        let item = items.next().ok_or(field_error(
            MptNodeField::BranchChild,
            DecodeError::Malformed,
        ))?;
        *slot = decode_node_reference_item(item, MptNodeField::BranchChild, true, depth_remaining)?;
    }
    let value = next_scalar(&mut items, MptNodeField::BranchValue)?.payload();
    require_canonical_branch(&children, value)?;
    Ok(MptNode::Branch(MptBranchNode { children, value }))
}

fn decode_compact_node<'a>(
    list: RlpList<'a>,
    depth_remaining: usize,
) -> Result<MptNode<'a>, MptNodeDecodeError> {
    let mut fields = list.items();
    let path = decode_compact_path(next_scalar(&mut fields, MptNodeField::CompactPath)?.payload())?;
    if path.is_leaf() {
        let value = next_scalar(&mut fields, MptNodeField::LeafValue)?.payload();
        if value.is_empty() {
            return Err(MptNodeDecodeError::EmptyLeafValue);
        }
        Ok(MptNode::Leaf(MptLeafNode { path, value }))
    } else {
        require_nonempty_extension_path(path)?;
        let child_item = fields.next().ok_or(field_error(
            MptNodeField::ExtensionChild,
            DecodeError::Malformed,
        ))?;
        let child = decode_node_reference_item(
            child_item,
            MptNodeField::ExtensionChild,
            false,
            depth_remaining,
        )?;
        Ok(MptNode::Extension(MptExtensionNode { path, child }))
    }
}

fn decode_compact_path(raw: &[u8]) -> Result<MptCompactPath<'_>, MptNodeDecodeError> {
    let first = *raw.first().ok_or(MptNodeDecodeError::EmptyCompactPath)?;
    let flag = first >> 4;
    let low = first & 0x0f;
    if flag > 3 {
        return Err(MptNodeDecodeError::InvalidCompactPathFlag { flag });
    }
    let odd_nibbles = (flag & 1) == 1;
    if !odd_nibbles && low != 0 {
        return Err(MptNodeDecodeError::InvalidCompactPathPadding { found: low });
    }
    let kind = if (flag & 2) == 2 {
        MptCompactPathKind::Leaf
    } else {
        MptCompactPathKind::Extension
    };
    Ok(MptCompactPath {
        raw,
        kind,
        odd_nibbles,
    })
}

fn decode_node_reference_item<'a>(
    item: Result<RlpItem<'a>, DecodeError>,
    field: MptNodeField,
    empty_allowed: bool,
    depth_remaining: usize,
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
            let decoded = decode_mpt_node_from_list(list, next_depth)?;
            require_canonical_extension_child(field, decoded)?;
            Ok(MptNodeReference::Inline(MptInlineNode {
                list,
                depth_remaining: next_depth,
            }))
        }
    }
}

fn require_nonempty_extension_path(path: MptCompactPath<'_>) -> Result<(), MptNodeDecodeError> {
    if path.nibble_count()? == 0 {
        return Err(MptNodeDecodeError::EmptyExtensionPath);
    }
    Ok(())
}

fn require_canonical_branch(
    children: &[MptNodeReference<'_>; MPT_BRANCH_CHILD_COUNT],
    value: &[u8],
) -> Result<(), MptNodeDecodeError> {
    let occupied = children
        .iter()
        .filter(|child| !matches!(child, MptNodeReference::Empty))
        .count()
        .saturating_add(usize::from(!value.is_empty()));
    if occupied < 2 {
        return Err(MptNodeDecodeError::DegenerateBranch { occupied });
    }
    Ok(())
}

fn require_canonical_extension_child(
    field: MptNodeField,
    node: MptNode<'_>,
) -> Result<(), MptNodeDecodeError> {
    if matches!(field, MptNodeField::ExtensionChild) && !matches!(node, MptNode::Branch(_)) {
        return Err(MptNodeDecodeError::NonCanonicalExtensionChild);
    }
    Ok(())
}

fn decode_scalar_reference<'a>(
    scalar: RlpScalar<'a>,
    field: MptNodeField,
    empty_allowed: bool,
) -> Result<MptNodeReference<'a>, MptNodeDecodeError> {
    let payload = scalar.payload();
    if payload.is_empty() {
        return if empty_allowed {
            Ok(MptNodeReference::Empty)
        } else {
            Err(MptNodeDecodeError::EmptyNodeReference { field })
        };
    }
    let found = payload.len();
    let bytes: [u8; MPT_HASH_REFERENCE_BYTES] = payload
        .try_into()
        .map_err(|_| MptNodeDecodeError::InvalidNodeReferenceLength { field, found })?;
    Ok(MptNodeReference::Hash(B256::from_bytes(bytes)))
}

fn next_scalar<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: MptNodeField,
) -> Result<RlpScalar<'a>, MptNodeDecodeError> {
    let item = fields
        .next()
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(field_error(field, DecodeError::UnexpectedList)),
    }
}

const fn field_error(field: MptNodeField, source: DecodeError) -> MptNodeDecodeError {
    MptNodeDecodeError::FieldDecode { field, source }
}

#[cfg(test)]
#[path = "mpt_tests.rs"]
mod tests;
