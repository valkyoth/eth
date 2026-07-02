#![no_main]

use eth_valkyoth_hash::Keccak256;
use eth_valkyoth_primitives::{Address, B256, Nonce};
use eth_valkyoth_protocol::{SetCodeAuthorization, SetCodeAuthorizationChainId, SignatureYParity};
use eth_valkyoth_verify::{
    set_code_authorization_signing_hash, validate_set_code_authorization_signature,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let authorization = authorization_from_data(data);
    let mut scratch = [0_u8; 160];
    let scratch_len = usize::from(data.first().copied().unwrap_or_default()).min(scratch.len());
    let Some(scratch) = scratch.get_mut(..scratch_len) else {
        return;
    };
    let _ = set_code_authorization_signing_hash(authorization, scratch, FuzzKeccak::new());
    let _ = validate_set_code_authorization_signature(
        authorization,
        None,
        scratch,
        FuzzKeccak::new(),
        FuzzKeccak::new(),
    );
});

fn authorization_from_data(data: &[u8]) -> SetCodeAuthorization {
    SetCodeAuthorization {
        chain_id: SetCodeAuthorizationChainId::from_be_bytes(bytes32(data, 0)),
        address: Address::from_bytes(address20(data, 32)),
        nonce: Nonce::new(nonce_from_data(data, 52)),
        y_parity: y_parity_from_data(data, 60),
        r: bytes32(data, 61),
        s: bytes32(data, 93),
    }
}

fn bytes32(data: &[u8], offset: usize) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    copy_from(data, offset, &mut bytes);
    bytes
}

fn address20(data: &[u8], offset: usize) -> [u8; 20] {
    let mut bytes = [0_u8; 20];
    copy_from(data, offset, &mut bytes);
    bytes
}

fn nonce_from_data(data: &[u8], offset: usize) -> u64 {
    let mut bytes = [0_u8; 8];
    copy_from(data, offset, &mut bytes);
    u64::from_be_bytes(bytes)
}

fn y_parity_from_data(data: &[u8], offset: usize) -> SignatureYParity {
    let value = data.get(offset).copied().unwrap_or_default() & 1;
    SignatureYParity::try_new(u64::from(value)).unwrap_or(SignatureYParity::Even)
}

fn copy_from(data: &[u8], offset: usize, target: &mut [u8]) {
    let Some(source) = data.get(offset..) else {
        return;
    };
    let len = source.len().min(target.len());
    if let Some(source) = source.get(..len)
        && let Some(target) = target.get_mut(..len)
    {
        target.copy_from_slice(source);
    }
}

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
