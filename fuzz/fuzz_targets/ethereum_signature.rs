#![no_main]

use eth_valkyoth_hash::Keccak256;
use eth_valkyoth_primitives::B256;
use eth_valkyoth_verify::{ETHEREUM_SIGNATURE_BYTES, EthereumSignature, recover_sender_from_digest};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut signature_bytes = [0_u8; ETHEREUM_SIGNATURE_BYTES];
    let signature_len = data.len().min(ETHEREUM_SIGNATURE_BYTES);
    if let Some(source) = data.get(..signature_len) {
        if let Some(target) = signature_bytes.get_mut(..signature_len) {
            target.copy_from_slice(source);
        }
    }

    let parsed = EthereumSignature::try_from_bytes(signature_bytes);
    let _ = EthereumSignature::try_from_parts_with_y_parity(
        signature_bytes
            .get(..32)
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .unwrap_or([0_u8; 32]),
        signature_bytes
            .get(32..64)
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .unwrap_or([0_u8; 32]),
        signature_bytes
            .get(64)
            .copied()
            .unwrap_or(u8::MAX),
    );

    if let Ok(signature) = parsed {
        let digest = digest_from_data(data);
        let _ = recover_sender_from_digest(digest, signature, FuzzKeccak::new());
    }
});

struct FuzzKeccak {
    digest: [u8; 32],
    cursor: usize,
}

impl FuzzKeccak {
    const fn new() -> Self {
        Self {
            digest: [0_u8; 32],
            cursor: 0,
        }
    }
}

impl Keccak256 for FuzzKeccak {
    fn update(&mut self, input: &[u8]) {
        for byte in input {
            let Some(slot) = self.digest.get_mut(self.cursor) else {
                self.cursor = 0;
                continue;
            };
            *slot ^= *byte;
            self.cursor = self.cursor.saturating_add(1);
        }
    }

    fn finalize(self) -> B256 {
        B256::from_bytes(self.digest)
    }
}

fn digest_from_data(data: &[u8]) -> B256 {
    let mut digest = [0_u8; 32];
    let Some(source) = data.get(ETHEREUM_SIGNATURE_BYTES..) else {
        return B256::from_bytes(digest);
    };
    let len = source.len().min(digest.len());
    if let Some(source) = source.get(..len) {
        if let Some(target) = digest.get_mut(..len) {
            target.copy_from_slice(source);
        }
    }
    B256::from_bytes(digest)
}
