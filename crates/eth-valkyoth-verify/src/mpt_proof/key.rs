use eth_valkyoth_codec::{DecodeError, encode_rlp_integer};

use super::MptProofVerificationError;

pub(super) fn encode_index_key(
    index: u64,
    output: &mut [u8],
) -> Result<usize, MptProofVerificationError> {
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

pub(super) fn key_nibble_len(key: &[u8]) -> usize {
    key.len().saturating_mul(2)
}
