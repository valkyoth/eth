use eth_valkyoth_hash::TinyKeccak256;
use eth_valkyoth_primitives::B256;

use crate::{
    ETHEREUM_PUBLIC_KEY_BYTES, EthereumSignature, RecoverableSecp256k1, SIGNING_DIGEST_BYTES,
    VerifyError,
};

pub(crate) type RealKeccak = TinyKeccak256;

pub(crate) struct TestSecp256k1Backend;

impl RecoverableSecp256k1 for TestSecp256k1Backend {
    fn recover_uncompressed_public_key(
        &mut self,
        signing_digest: B256,
        signature: EthereumSignature,
    ) -> Result<[u8; ETHEREUM_PUBLIC_KEY_BYTES], VerifyError> {
        let secp256k1_signature =
            k256::ecdsa::Signature::from_scalars(signature.r(), signature.s())
                .map_err(|_| VerifyError::InvalidSignature)?;
        if secp256k1_signature.normalize_s() != secp256k1_signature {
            return Err(VerifyError::InvalidSignature);
        }
        let recovery_id = k256::ecdsa::RecoveryId::try_from(signature.y_parity().get())
            .map_err(|_| VerifyError::InvalidSignature)?;
        let digest_bytes = <[u8; SIGNING_DIGEST_BYTES]>::from(signing_digest);
        let verifying_key = k256::ecdsa::VerifyingKey::recover_from_prehash(
            &digest_bytes,
            &secp256k1_signature,
            recovery_id,
        )
        .map_err(|_| VerifyError::InvalidSignature)?;
        let encoded = verifying_key.to_sec1_point(false);
        encoded
            .as_bytes()
            .get(1..)
            .and_then(|value| <[u8; ETHEREUM_PUBLIC_KEY_BYTES]>::try_from(value).ok())
            .ok_or(VerifyError::InvalidSignature)
    }
}
