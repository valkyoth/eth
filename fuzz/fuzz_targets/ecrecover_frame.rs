#![no_main]

use eth_valkyoth_evm_core::{
    EVM_ECRECOVER_PUBLIC_KEY_BYTES, EvmEcRecoverBackend, EvmEcRecoverSignature,
    EvmPrecompileKeccak256, execute_ecrecover,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut output = [0_u8; 32];
    let result = execute_ecrecover(data, &mut output, FuzzBackend, FuzzKeccak);
    if let Ok(len) = result {
        assert!(len == 0 || len == output.len());
    }
});

struct FuzzBackend;

impl EvmEcRecoverBackend for FuzzBackend {
    fn recover_uncompressed_public_key(
        &mut self,
        _digest: [u8; 32],
        signature: EvmEcRecoverSignature,
    ) -> Option<[u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES]> {
        let mut public_key = [0_u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES];
        let r = signature.r();
        let s = signature.s();
        public_key[..32].copy_from_slice(&r);
        public_key[32..].copy_from_slice(&s);
        Some(public_key)
    }
}

struct FuzzKeccak;

impl EvmPrecompileKeccak256 for FuzzKeccak {
    fn keccak256(&mut self, input: &[u8]) -> [u8; 32] {
        let mut digest = [0_u8; 32];
        for (index, byte) in input.iter().enumerate() {
            if let Some(slot) = digest.get_mut(index % 32) {
                *slot ^= byte.wrapping_add(index as u8);
            }
        }
        digest
    }
}
