use eth_valkyoth_codec::DecodeSession;
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;

use crate::mpt::{
    MPT_MAX_INLINE_REFERENCE_BYTES, MptNode, MptNodeDecodeError, decode_mpt_node_body_in_session,
};
use crate::mpt_resolver::MptNodeResolver;

use super::{
    MptProofRoot, MptProofValue, MptProofVerificationError, compare_expected_value,
    proof_resource_error, walk_to_value,
};

pub(super) trait ProofNodeCursor<'a> {
    fn next_hashed_node(
        &mut self,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError>;

    fn next_child_node(
        &mut self,
        expected: B256,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError>;

    fn next_extension_child(
        &mut self,
        expected: B256,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError>;
}

pub(super) struct ProofCursor<'a, 'h, H> {
    expected_root: MptProofRoot,
    nodes: &'a [&'a [u8]],
    index: usize,
    new_hasher: &'h mut dyn FnMut() -> H,
}

impl<'a, 'h, H> ProofCursor<'a, 'h, H>
where
    H: Keccak256,
{
    pub(super) fn new(
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

    pub(super) fn is_consumed(&self) -> bool {
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
        if hash_one((self.new_hasher)(), encoded) != expected {
            return Err(MptProofVerificationError::WrongRoot);
        }
        self.index = self.index.saturating_add(1);
        Ok(node)
    }
}

impl<'a, H> ProofNodeCursor<'a> for ProofCursor<'a, '_, H>
where
    H: Keccak256,
{
    fn next_hashed_node(
        &mut self,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        self.next_node_matching(self.expected_root.to_b256(), false, false, session)
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
}

pub(crate) fn verify_key_in_resolver<'a>(
    resolver: &'a MptNodeResolver<'a>,
    key: &[u8],
    expected_value: &[u8],
    session: &mut DecodeSession,
) -> Result<(), MptProofVerificationError> {
    session
        .check_input_len(expected_value.len())
        .map_err(proof_resource_error)?;
    let mut cursor = ResolverCursor { resolver };
    let first = cursor.next_hashed_node(session)?;
    match walk_to_value(first, key, &mut cursor, session)? {
        MptProofValue::Present(found) => {
            compare_expected_value(expected_value, session).and_then(|()| {
                if found == expected_value {
                    Ok(())
                } else {
                    Err(MptProofVerificationError::ValueMismatch)
                }
            })
        }
        MptProofValue::Absent => Err(MptProofVerificationError::Absent),
    }
}

struct ResolverCursor<'a> {
    resolver: &'a MptNodeResolver<'a>,
}

impl<'a> ResolverCursor<'a> {
    fn decode_matching(
        &self,
        expected: B256,
        require_branch: bool,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        let encoded = self
            .resolver
            .resolve(expected)
            .ok_or(MptProofVerificationError::MissingProofNode)?;
        let node = decode_mpt_node_body_in_session(encoded, session)
            .map_err(MptProofVerificationError::MalformedNode)?;
        if require_branch && !matches!(node, MptNode::Branch(_)) {
            return Err(MptProofVerificationError::MalformedNode(
                MptNodeDecodeError::NonCanonicalExtensionChild,
            ));
        }
        Ok(node)
    }
}

impl<'a> ProofNodeCursor<'a> for ResolverCursor<'a> {
    fn next_hashed_node(
        &mut self,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        self.decode_matching(self.resolver.anchor().root().to_b256(), false, session)
    }

    fn next_child_node(
        &mut self,
        expected: B256,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        self.decode_matching(expected, false, session)
    }

    fn next_extension_child(
        &mut self,
        expected: B256,
        session: &mut DecodeSession,
    ) -> Result<MptNode<'a>, MptProofVerificationError> {
        self.decode_matching(expected, true, session)
    }
}
