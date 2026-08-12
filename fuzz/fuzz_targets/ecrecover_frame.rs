#![no_main]

use eth_valkyoth_evm_core::{
    EVM_ECRECOVER_PUBLIC_KEY_BYTES, EvmEcRecover, EvmEcRecoverBackend, EvmEcRecoverSignature,
    EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKeccak256, EvmPrecompileKind,
    EvmPrecompileRegistry, EvmPrecompileStatus,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::FRONTIER)
        .and_then(|registry| registry.descriptor(EvmPrecompileKind::EcRecover))
        .expect("Frontier ECRECOVER descriptor exists");
    let Ok(quote) = descriptor.quote::<EvmEcRecover>(data) else {
        return;
    };
    let mut output = [0_u8; 32];
    let mut gas = EvmGasMeter::try_new(EvmGas::new(3_000)).expect("ECRECOVER gas is valid");
    let Ok(outcome) = quote.authorize_and_execute_ecrecover(
        &mut gas,
        &mut output,
        FuzzBackend,
        FuzzKeccak,
    ) else {
        return;
    };
    if outcome.status() == EvmPrecompileStatus::Success {
        assert!(outcome.output_len() == 0 || outcome.output_len() == output.len());
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
