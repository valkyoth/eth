use eth_valkyoth_codec::{
    DecodeError, DecodeLimits, DecodeSession, DecodeSessionPolicy, encode_rlp_integer,
};
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;

use crate::mpt::{
    MPT_MAX_INLINE_REFERENCE_BYTES, MptCompactPath, MptNode, MptNodeDecodeError, MptNodeReference,
    decode_mpt_node_body_in_session,
};

mod error;
mod plan;
mod preflight;
mod root;

use plan::plan_remaining_work;
pub(crate) use preflight::{preflight_proof, proof_resource_error};

pub use error::{MptProofVerificationError, MptProofVerificationErrorCategory};
pub use root::{
    MptProofRoot, ReceiptTrieRoot, TransactionTrieRoot, VerifiedReceiptInclusion,
    VerifiedTransactionInclusion,
};

const MAX_RLP_U64_BYTES: usize = 9;
/// Hard cap on transaction and receipt proof traversal depth.
///
/// This is independent from [`DecodeLimits::max_proof_nodes`] so deployments
/// cannot accidentally configure native call-stack growth into proof walking.
pub const MAX_PROOF_WALK_DEPTH: usize = 128;

/// Verifies that `encoded_transaction` is included at `transaction_index`.
///
/// The trie key is `rlp(transaction_index)`, as used by Ethereum transaction
/// tries. The value is compared byte-for-byte with `encoded_transaction`; this
/// function does not decode or validate the transaction envelope itself.
pub fn verify_transaction_inclusion<H>(
    root: TransactionTrieRoot,
    transaction_index: u64,
    encoded_transaction: &[u8],
    proof_nodes: &[&[u8]],
    limits: DecodeLimits,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedTransactionInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let mut session = compatibility_session(limits)?;
    verify_transaction_inclusion_in_session(
        root,
        transaction_index,
        encoded_transaction,
        proof_nodes,
        &mut session,
        new_hasher,
    )
}

/// Verifies transaction inclusion through one shared decode/work session.
pub fn verify_transaction_inclusion_in_session<H>(
    root: TransactionTrieRoot,
    transaction_index: u64,
    encoded_transaction: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedTransactionInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    verify_indexed_inclusion_in_session(
        root.into(),
        transaction_index,
        encoded_transaction,
        proof_nodes,
        session,
        new_hasher,
    )?;
    Ok(VerifiedTransactionInclusion::new(transaction_index, root))
}

/// Verifies that `encoded_receipt` is included at `transaction_index`.
///
/// The trie key is `rlp(transaction_index)`, as used by Ethereum receipt
/// tries. The value is compared byte-for-byte with `encoded_receipt`; this
/// function does not decode or validate receipt fields.
pub fn verify_receipt_inclusion<H>(
    root: ReceiptTrieRoot,
    transaction_index: u64,
    encoded_receipt: &[u8],
    proof_nodes: &[&[u8]],
    limits: DecodeLimits,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedReceiptInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let mut session = compatibility_session(limits)?;
    verify_receipt_inclusion_in_session(
        root,
        transaction_index,
        encoded_receipt,
        proof_nodes,
        &mut session,
        new_hasher,
    )
}

/// Verifies receipt inclusion through one shared decode/work session.
pub fn verify_receipt_inclusion_in_session<H>(
    root: ReceiptTrieRoot,
    transaction_index: u64,
    encoded_receipt: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedReceiptInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    verify_indexed_inclusion_in_session(
        root.into(),
        transaction_index,
        encoded_receipt,
        proof_nodes,
        session,
        new_hasher,
    )?;
    Ok(VerifiedReceiptInclusion::new(transaction_index, root))
}

fn verify_indexed_inclusion_in_session<H>(
    root: MptProofRoot,
    index: u64,
    value: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    new_hasher: impl FnMut() -> H,
) -> Result<(), MptProofVerificationError>
where
    H: Keccak256,
{
    let mut key = [0_u8; MAX_RLP_U64_BYTES];
    let key_len = encode_index_key(index, &mut key)?;
    let key = key
        .get(..key_len)
        .ok_or(MptProofVerificationError::KeyEncode(
            DecodeError::OffsetOutOfBounds,
        ))?;
    verify_key_inclusion_in_session(root, key, value, proof_nodes, session, new_hasher)
}

pub(crate) fn verify_key_inclusion_in_session<H>(
    root: MptProofRoot,
    key: &[u8],
    value: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    new_hasher: impl FnMut() -> H,
) -> Result<(), MptProofVerificationError>
where
    H: Keccak256,
{
    preflight_proof(proof_nodes, value, 0, 0, session)?;
    check_preflighted_key_inclusion_capacity(key, value, proof_nodes, session)?;
    verify_preflighted_key_inclusion(root, key, value, proof_nodes, session, new_hasher)
}

pub(crate) fn check_preflighted_key_inclusion_capacity(
    key: &[u8],
    value: &[u8],
    proof_nodes: &[&[u8]],
    session: &DecodeSession,
) -> Result<(), MptProofVerificationError> {
    let planned = plan_remaining_work(key, value, proof_nodes, session)?;
    session
        .check_remaining_capacity(planned)
        .map_err(proof_resource_error)
}

pub(crate) fn verify_preflighted_key_inclusion<H>(
    root: MptProofRoot,
    key: &[u8],
    value: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    mut new_hasher: impl FnMut() -> H,
) -> Result<(), MptProofVerificationError>
where
    H: Keccak256,
{
    let mut cursor = ProofCursor::new(root, proof_nodes, &mut new_hasher);
    let first = cursor.next_hashed_node(session)?;
    walk_to_value(first, key, value, &mut cursor, session)?;
    if cursor.is_consumed() {
        Ok(())
    } else {
        Err(MptProofVerificationError::TrailingProofNodes)
    }
}

fn walk_to_value<'a, H>(
    mut node: MptNode<'a>,
    key: &[u8],
    expected_value: &[u8],
    cursor: &mut ProofCursor<'a, '_, H>,
    session: &mut DecodeSession,
) -> Result<(), MptProofVerificationError>
where
    H: Keccak256,
{
    let mut key_nibble_offset = 0usize;
    let mut depth = 0usize;

    loop {
        depth = depth
            .checked_add(1)
            .ok_or(MptProofVerificationError::ProofTooDeep)?;
        if depth > MAX_PROOF_WALK_DEPTH {
            return Err(MptProofVerificationError::ProofTooDeep);
        }

        let reference = match node {
            MptNode::Branch(branch) => {
                if key_nibble_offset == key_nibble_len(key) {
                    return compare_value(branch.value(), expected_value, session);
                }
                let child_index = key_nibble(key, key_nibble_offset)?;
                key_nibble_offset = key_nibble_offset.saturating_add(1);
                branch
                    .children()
                    .nth(usize::from(child_index))
                    .ok_or(MptProofVerificationError::Absent)?
                    .map_err(MptProofVerificationError::MalformedNode)?
            }
            MptNode::Extension(extension) => {
                let consumed = match_compact_path(extension.path, key, key_nibble_offset, session)?;
                key_nibble_offset = key_nibble_offset.saturating_add(consumed);
                extension.child
            }
            MptNode::Leaf(leaf) => {
                let consumed = match_compact_path(leaf.path, key, key_nibble_offset, session)?;
                if key_nibble_offset.saturating_add(consumed) != key_nibble_len(key) {
                    return Err(MptProofVerificationError::Absent);
                }
                return compare_value(leaf.value, expected_value, session);
            }
        };

        node = match reference {
            MptNodeReference::Empty => return Err(MptProofVerificationError::Absent),
            MptNodeReference::Hash(expected) => match node {
                MptNode::Extension(_) => cursor.next_extension_child(expected, session)?,
                MptNode::Branch(_) | MptNode::Leaf(_) => {
                    cursor.next_child_node(expected, session)?
                }
            },
            MptNodeReference::Inline(inline) => inline
                .node_in_session(session)
                .map_err(MptProofVerificationError::MalformedNode)?,
        };
    }
}

struct ProofCursor<'a, 'h, H> {
    expected_root: MptProofRoot,
    nodes: &'a [&'a [u8]],
    index: usize,
    new_hasher: &'h mut dyn FnMut() -> H,
}

impl<'a, 'h, H> ProofCursor<'a, 'h, H>
where
    H: Keccak256,
{
    fn new(
        expected_root: MptProofRoot,
        nodes: &'a [&'a [u8]],
        new_hasher: &'h mut impl FnMut() -> H,
    ) -> Self {
        Self {
            expected_root,
            nodes,
            index: 0,
            new_hasher,
        }
    }

    fn next_hashed_node(
        &mut self,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        let root = self.expected_root.to_b256();
        self.next_node_matching(root, false, false, session)
    }

    fn next_child_node(
        &mut self,
        expected: B256,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        self.next_node_matching(expected, true, false, session)
    }

    fn next_extension_child(
        &mut self,
        expected: B256,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        self.next_node_matching(expected, true, true, session)
    }

    fn is_consumed(&self) -> bool {
        self.index == self.nodes.len()
    }

    fn next_node_matching(
        &mut self,
        expected: B256,
        is_child: bool,
        require_branch: bool,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        let encoded = *self
            .nodes
            .get(self.index)
            .ok_or(MptProofVerificationError::MissingProofNode)?;
        if is_child && encoded.len() < MPT_MAX_INLINE_REFERENCE_BYTES {
            return Err(MptProofVerificationError::MalformedNode(
                MptNodeDecodeError::HashedNodeTooShort {
                    found: encoded.len(),
                },
            ));
        }
        let node = decode_mpt_node_body_in_session(encoded, session)
            .map_err(MptProofVerificationError::MalformedNode)?;
        if require_branch && !matches!(node, MptNode::Branch(_)) {
            return Err(MptProofVerificationError::MalformedNode(
                MptNodeDecodeError::NonCanonicalExtensionChild,
            ));
        }
        session
            .account_hashes(1, encoded.len())
            .map_err(proof_resource_error)?;
        let digest = hash_one((self.new_hasher)(), encoded);
        if digest != expected {
            return Err(MptProofVerificationError::WrongRoot);
        }
        self.index = self.index.saturating_add(1);
        Ok(node)
    }
}

fn encode_index_key(index: u64, output: &mut [u8]) -> Result<usize, MptProofVerificationError> {
    let bytes = index.to_be_bytes();
    let payload = if index == 0 {
        &[][..]
    } else {
        let first = bytes
            .iter()
            .position(|byte| *byte != 0)
            .ok_or(MptProofVerificationError::KeyEncode(DecodeError::Malformed))?;
        bytes
            .get(first..)
            .ok_or(MptProofVerificationError::KeyEncode(
                DecodeError::OffsetOutOfBounds,
            ))?
    };
    encode_rlp_integer(payload, output).map_err(MptProofVerificationError::KeyEncode)
}

fn match_compact_path(
    path: MptCompactPath<'_>,
    key: &[u8],
    key_nibble_offset: usize,
    session: &mut DecodeSession,
) -> Result<usize, MptProofVerificationError> {
    if !path.is_leaf() && key_nibble_offset == key_nibble_len(key) {
        return Err(MptProofVerificationError::Absent);
    }
    let count = path
        .nibble_count()
        .map_err(MptProofVerificationError::MalformedNode)?;
    session
        .account_nibbles(count)
        .map_err(proof_resource_error)?;
    if key_nibble_offset.saturating_add(count) > key_nibble_len(key) {
        return Err(MptProofVerificationError::Absent);
    }
    for index in 0..count {
        let expected = compact_path_nibble(path, index)?;
        let actual = key_nibble(key, key_nibble_offset.saturating_add(index))?;
        if expected != actual {
            return Err(MptProofVerificationError::Absent);
        }
    }
    Ok(count)
}

fn compact_path_nibble(
    path: MptCompactPath<'_>,
    path_nibble_index: usize,
) -> Result<u8, MptProofVerificationError> {
    let raw = path.raw();
    if path.has_odd_nibbles() {
        if path_nibble_index == 0 {
            return raw.first().map(|byte| byte & 0x0f).ok_or(
                MptProofVerificationError::MalformedNode(MptNodeDecodeError::EmptyCompactPath),
            );
        }
        byte_nibble(raw, path_nibble_index.saturating_add(1))
    } else {
        byte_nibble(raw, path_nibble_index.saturating_add(2))
    }
}

fn key_nibble(key: &[u8], nibble_index: usize) -> Result<u8, MptProofVerificationError> {
    byte_nibble(key, nibble_index)
}

fn byte_nibble(bytes: &[u8], nibble_index: usize) -> Result<u8, MptProofVerificationError> {
    let byte = *bytes
        .get(nibble_index / 2)
        .ok_or(MptProofVerificationError::Absent)?;
    if nibble_index.is_multiple_of(2) {
        Ok(byte >> 4)
    } else {
        Ok(byte & 0x0f)
    }
}

fn key_nibble_len(key: &[u8]) -> usize {
    key.len().saturating_mul(2)
}

fn compare_value(
    found: &[u8],
    expected: &[u8],
    session: &mut DecodeSession,
) -> Result<(), MptProofVerificationError> {
    let compared = found
        .len()
        .checked_add(expected.len())
        .ok_or_else(|| proof_resource_error(DecodeError::ValueBytesExceeded))?;
    session
        .account_value_bytes(compared)
        .map_err(proof_resource_error)?;
    if found == expected {
        Ok(())
    } else {
        Err(MptProofVerificationError::ValueMismatch)
    }
}

pub(crate) fn compatibility_session(
    limits: DecodeLimits,
) -> Result<DecodeSession, MptProofVerificationError> {
    let policy = DecodeSessionPolicy::compatibility_policy(limits).map_err(proof_resource_error)?;
    DecodeSession::new(policy).map_err(proof_resource_error)
}

#[cfg(test)]
#[path = "mpt_proof_budget_tests.rs"]
mod budget_tests;

#[cfg(test)]
#[path = "mpt_proof_tests.rs"]
mod tests;
