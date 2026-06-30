#![no_std]
#![forbid(unsafe_code)]
//! `no_std` Keccak-256 hashing boundary for Ethereum protocol code.
//!
//! Ethereum uses Keccak-256, not the finalized FIPS SHA3-256 variant, for
//! transaction hashes, header hashes, recovered sender addresses, and proof
//! roots. This crate intentionally defines only the boundary in `v0.9.3`.
//! Callers provide an implementation from hardware, platform APIs, WASM, or an
//! explicitly reviewed software crate.

#[cfg(feature = "std")]
extern crate std;

use eth_valkyoth_primitives::B256;

/// Keccak-256 digest domain used by Ethereum protocol hashing.
pub type Keccak256Digest = B256;

/// Incremental Keccak-256 hasher boundary.
///
/// Implementations must compute Ethereum Keccak-256, not FIPS SHA3-256. This
/// trait is intentionally minimal and allocation-free so it can be implemented
/// by hardware, platform, embedded, WASM, or reviewed software backends without
/// forcing a concrete hashing crate into the default `eth` dependency graph.
pub trait Keccak256 {
    /// Absorbs the next input chunk.
    fn update(&mut self, input: &[u8]);

    /// Finalizes the hash and returns the 32-byte digest.
    #[must_use]
    fn finalize(self) -> Keccak256Digest;
}

/// Hashes a single byte slice with a caller-provided Keccak-256 implementation.
#[must_use]
pub fn hash_one<H>(mut hasher: H, input: &[u8]) -> Keccak256Digest
where
    H: Keccak256,
{
    hasher.update(input);
    hasher.finalize()
}

/// Hashes ordered chunks with a caller-provided Keccak-256 implementation.
///
/// Chunk boundaries must not affect the digest for a correct implementation.
#[must_use]
pub fn hash_chunks<'a, H, I>(mut hasher: H, chunks: I) -> Keccak256Digest
where
    H: Keccak256,
    I: IntoIterator<Item = &'a [u8]>,
{
    for chunk in chunks {
        hasher.update(chunk);
    }
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TranscriptHasher {
        calls: u8,
        total_len: usize,
    }

    impl Keccak256 for TranscriptHasher {
        fn update(&mut self, input: &[u8]) {
            self.calls = self.calls.saturating_add(1);
            self.total_len = self.total_len.saturating_add(input.len());
        }

        fn finalize(self) -> Keccak256Digest {
            let mut bytes = [0_u8; 32];
            if let Some(first) = bytes.first_mut() {
                *first = self.calls;
            }
            if let Some(last) = bytes.last_mut() {
                *last = u8::try_from(self.total_len).unwrap_or(u8::MAX);
            }
            B256::from_bytes(bytes)
        }
    }

    #[test]
    fn hashes_one_slice_with_caller_hasher() {
        let digest = hash_one(TranscriptHasher::default(), b"abc");
        let expected = expected_transcript_digest(1, 3);

        assert_eq!(digest, expected);
    }

    #[test]
    fn hashes_chunks_in_order_with_caller_hasher() {
        let chunks: [&[u8]; 2] = [b"cat", b"dog"];
        let digest = hash_chunks(TranscriptHasher::default(), chunks);
        let expected = expected_transcript_digest(2, 6);

        assert_eq!(digest, expected);
    }

    fn expected_transcript_digest(calls: u8, total_len: usize) -> B256 {
        let mut bytes = [0_u8; 32];
        if let Some(first) = bytes.first_mut() {
            *first = calls;
        }
        if let Some(last) = bytes.last_mut() {
            *last = u8::try_from(total_len).unwrap_or(u8::MAX);
        }
        B256::from_bytes(bytes)
    }
}
