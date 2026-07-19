use eth_valkyoth_codec::{DecodeError, DecodeSession};

use crate::mpt::{MPT_MAX_INLINE_REFERENCE_BYTES, MptNode, MptNodeReference};
use crate::mpt::{MptNodeDecodeError, MptNodeField, decode_mpt_node_body_in_session};

use super::MptProofVerificationError;

pub(crate) fn preflight_proof(
    proof_nodes: &[&[u8]],
    expected_value: &[u8],
    additional_hashes: usize,
    additional_hash_bytes: usize,
    session: &mut DecodeSession,
) -> Result<(), MptProofVerificationError> {
    if proof_nodes.is_empty() {
        return Err(MptProofVerificationError::MissingProofNode);
    }

    session
        .check_input_len(expected_value.len())
        .map_err(proof_resource_error)?;
    session
        .account_value_bytes(expected_value.len())
        .map_err(proof_resource_error)?;
    session
        .account_proof_nodes(proof_nodes.len())
        .map_err(proof_resource_error)?;

    let mut total_hash_bytes = 0usize;
    for encoded in proof_nodes {
        session
            .check_input_len(encoded.len())
            .map_err(proof_resource_error)?;
        total_hash_bytes = total_hash_bytes
            .checked_add(encoded.len())
            .ok_or_else(|| proof_resource_error(DecodeError::HashBytesExceeded))?;
    }
    let total_hashes = proof_nodes
        .len()
        .checked_add(additional_hashes)
        .ok_or_else(|| proof_resource_error(DecodeError::HashCountExceeded))?;
    let total_hash_bytes = total_hash_bytes
        .checked_add(additional_hash_bytes)
        .ok_or_else(|| proof_resource_error(DecodeError::HashBytesExceeded))?;
    session
        .check_hash_capacity(total_hashes, total_hash_bytes)
        .map_err(proof_resource_error)?;

    // Every supplied node is syntactically and locally canonically admitted
    // before the first attacker-controlled node byte reaches Keccak.
    let mut require_branch = false;
    for (index, encoded) in proof_nodes.iter().copied().enumerate() {
        if index > 0 && encoded.len() < MPT_MAX_INLINE_REFERENCE_BYTES {
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
        require_branch = matches!(
            node,
            MptNode::Extension(extension)
                if matches!(extension.child, MptNodeReference::Hash(_))
        );
    }
    if require_branch {
        return Err(MptProofVerificationError::MissingProofNode);
    }
    Ok(())
}

pub(crate) const fn proof_resource_error(source: DecodeError) -> MptProofVerificationError {
    MptProofVerificationError::MalformedNode(MptNodeDecodeError::FieldDecode {
        field: MptNodeField::ProofNode,
        source,
    })
}
