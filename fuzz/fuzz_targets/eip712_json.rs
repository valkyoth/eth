#![no_main]

use eth_valkyoth_hash::{Keccak256, Keccak256Digest};
use eth_valkyoth_primitives::B256;
use eth_valkyoth_verify::{Eip712JsonLimits, eip712_json_typed_data_signing_digest};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = core::str::from_utf8(data) else {
        return;
    };

    let mut scratch = [0_u8; 1024];
    let _ = eip712_json_typed_data_signing_digest::<FuzzKeccak>(
        input,
        Eip712JsonLimits::DEFAULT,
        &mut scratch,
    );
});

#[derive(Default)]
struct FuzzKeccak {
    digest: [u8; 32],
    cursor: usize,
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

    fn finalize(self) -> Keccak256Digest {
        B256::from_bytes(self.digest)
    }
}
