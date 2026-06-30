#![no_std]
#![forbid(unsafe_code)]
//! `no_std` Keccak-256 hashing boundary for Ethereum protocol code.
//!
//! Ethereum uses Keccak-256, not the finalized FIPS SHA3-256 variant, for
//! transaction hashes, header hashes, recovered sender addresses, and proof
//! roots. This crate intentionally defines only the boundary and conformance
//! helpers in `v0.10.0`.
//! Callers provide an implementation from hardware, platform APIs, WASM, or an
//! explicitly reviewed software crate.

#[cfg(feature = "std")]
extern crate std;

use core::fmt;

use eth_valkyoth_primitives::B256;

/// Keccak-256 digest domain used by Ethereum protocol hashing.
pub type Keccak256Digest = B256;

/// Ethereum Keccak-256 of the empty byte string.
///
/// SHA3-256 of `b""` has a different digest. Backend authors should use this
/// known-answer value to reject FIPS SHA3-256 implementations accidentally
/// wired into Ethereum hashing paths.
pub const KECCAK256_EMPTY: [u8; 32] = [
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
];

/// Keccak backend conformance failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Keccak256ConformanceError;

impl fmt::Display for Keccak256ConformanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Keccak-256 backend failed empty-input conformance check")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Keccak256ConformanceError {}

/// Incremental Keccak-256 hasher boundary.
///
/// Implementations must compute Ethereum Keccak-256, not FIPS SHA3-256. This
/// trait is intentionally minimal and allocation-free so it can be implemented
/// by hardware, platform, embedded, WASM, or reviewed software backends without
/// forcing a concrete hashing crate into the default `eth` dependency graph.
pub trait Keccak256 {
    /// Required empty-input digest for Ethereum Keccak-256.
    const EMPTY_DIGEST: [u8; 32] = KECCAK256_EMPTY;

    /// Absorbs the next input chunk.
    fn update(&mut self, input: &[u8]);

    /// Finalizes the hash and returns the 32-byte digest.
    #[must_use]
    fn finalize(self) -> Keccak256Digest;
}

/// Verifies a default hasher against the empty-input Keccak-256 KAT.
///
/// This catches the most common backend admission error: wiring FIPS SHA3-256
/// where Ethereum Keccak-256 is required.
pub fn verify_empty_digest<H>() -> Result<(), Keccak256ConformanceError>
where
    H: Default + Keccak256,
{
    verify_empty_digest_with(H::default())
}

/// Verifies a hasher instance against the empty-input Keccak-256 KAT.
///
/// Use this for hardware-backed, platform-backed, or otherwise configured
/// hashers that cannot implement [`Default`].
pub fn verify_empty_digest_with<H>(hasher: H) -> Result<(), Keccak256ConformanceError>
where
    H: Keccak256,
{
    let digest = hash_one(hasher, b"");
    if <[u8; 32]>::from(digest) == KECCAK256_EMPTY {
        return Ok(());
    }
    Err(Keccak256ConformanceError)
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
    use core::fmt::Write;

    #[derive(Default)]
    struct TranscriptHasher {
        calls: u8,
        total_len: usize,
    }

    struct FixedHasher {
        digest: B256,
    }

    struct FixedBuffer {
        bytes: [u8; 64],
        len: usize,
    }

    impl FixedBuffer {
        const fn new() -> Self {
            Self {
                bytes: [0_u8; 64],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            self.bytes
                .get(..self.len)
                .and_then(|bytes| core::str::from_utf8(bytes).ok())
                .unwrap_or("")
        }
    }

    impl Write for FixedBuffer {
        fn write_str(&mut self, input: &str) -> core::fmt::Result {
            let end = self.len.saturating_add(input.len());
            let Some(slot) = self.bytes.get_mut(self.len..end) else {
                return Err(core::fmt::Error);
            };
            slot.copy_from_slice(input.as_bytes());
            self.len = end;
            Ok(())
        }
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

    impl Keccak256 for FixedHasher {
        fn update(&mut self, _input: &[u8]) {}

        fn finalize(self) -> Keccak256Digest {
            self.digest
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

    #[test]
    fn empty_digest_vector_is_available_to_backend_tests() {
        assert_eq!(
            KECCAK256_EMPTY,
            [
                0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7,
                0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04,
                0x5d, 0x85, 0xa4, 0x70,
            ]
        );
    }

    #[test]
    fn empty_digest_helper_accepts_non_default_hasher_instances() {
        let hasher = FixedHasher {
            digest: B256::from_bytes(KECCAK256_EMPTY),
        };

        assert_eq!(verify_empty_digest_with(hasher), Ok(()));
    }

    #[test]
    fn empty_digest_helper_rejects_wrong_empty_digest() {
        let hasher = FixedHasher {
            digest: B256::from_bytes([0_u8; 32]),
        };

        assert_eq!(
            verify_empty_digest_with(hasher),
            Err(Keccak256ConformanceError)
        );
    }

    #[test]
    fn conformance_error_has_stable_display_text() {
        let mut buffer = FixedBuffer::new();
        let written = write!(&mut buffer, "{}", Keccak256ConformanceError);

        assert!(written.is_ok());
        assert_eq!(
            buffer.as_str(),
            "Keccak-256 backend failed empty-input conformance check"
        );
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
