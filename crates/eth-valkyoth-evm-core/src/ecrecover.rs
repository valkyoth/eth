use core::cmp::Ordering;

use crate::{EVM_PRECOMPILE_INPUT_LIMIT, EvmCoreError, EvmPrecompileKind, EvmPrecompilePlan};

/// Canonical byte length of the ECRECOVER input frame.
pub const EVM_ECRECOVER_INPUT_BYTES: usize = 128;
/// Byte length of the uncompressed public key payload, without SEC1 prefix.
pub const EVM_ECRECOVER_PUBLIC_KEY_BYTES: usize = 64;
const EVM_ECRECOVER_OUTPUT_BYTES: usize = 32;
const WORD_BYTES: usize = 32;
const V_OFFSET: usize = 32;
const R_OFFSET: usize = 64;
const S_OFFSET: usize = 96;
const ADDRESS_OFFSET: usize = 12;

const SECP256K1_ORDER_BE: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
];

/// ECRECOVER signature accepted by the native precompile boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmEcRecoverSignature {
    r: [u8; 32],
    s: [u8; 32],
    y_parity: u8,
}

impl EvmEcRecoverSignature {
    /// Returns the raw `r` scalar.
    #[must_use]
    pub const fn r(self) -> [u8; 32] {
        self.r
    }

    /// Returns the raw `s` scalar.
    #[must_use]
    pub const fn s(self) -> [u8; 32] {
        self.s
    }

    /// Returns the normalized Ethereum y parity, `0` or `1`.
    #[must_use]
    pub const fn y_parity(self) -> u8 {
        self.y_parity
    }
}

/// Caller-provided ECRECOVER secp256k1 backend boundary.
pub trait EvmEcRecoverBackend {
    /// Recovers a 64-byte uncompressed public key payload from `digest` and
    /// `signature`. Invalid signatures must return `None`.
    fn recover_uncompressed_public_key(
        &mut self,
        digest: [u8; 32],
        signature: EvmEcRecoverSignature,
    ) -> Option<[u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES]>;
}

impl<T> EvmEcRecoverBackend for &mut T
where
    T: EvmEcRecoverBackend + ?Sized,
{
    fn recover_uncompressed_public_key(
        &mut self,
        digest: [u8; 32],
        signature: EvmEcRecoverSignature,
    ) -> Option<[u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES]> {
        (**self).recover_uncompressed_public_key(digest, signature)
    }
}

/// Caller-provided Keccak-256 boundary for ECRECOVER address derivation.
pub trait EvmPrecompileKeccak256 {
    /// Hashes one byte slice with Ethereum Keccak-256.
    fn keccak256(&mut self, input: &[u8]) -> [u8; 32];
}

impl<T> EvmPrecompileKeccak256 for &mut T
where
    T: EvmPrecompileKeccak256 + ?Sized,
{
    fn keccak256(&mut self, input: &[u8]) -> [u8; 32] {
        (**self).keccak256(input)
    }
}

impl EvmPrecompilePlan {
    /// Executes ECRECOVER with caller-provided secp256k1 and Keccak backends.
    pub fn execute_ecrecover<B, H>(
        self,
        input: &[u8],
        output: &mut [u8],
        backend: B,
        hasher: H,
    ) -> Result<usize, EvmCoreError>
    where
        B: EvmEcRecoverBackend,
        H: EvmPrecompileKeccak256,
    {
        if self.descriptor().kind != EvmPrecompileKind::EcRecover {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_ecrecover(input, output, backend, hasher)
    }
}

/// Executes the ECRECOVER precompile.
pub fn execute_ecrecover<B, H>(
    input: &[u8],
    output: &mut [u8],
    mut backend: B,
    mut hasher: H,
) -> Result<usize, EvmCoreError>
where
    B: EvmEcRecoverBackend,
    H: EvmPrecompileKeccak256,
{
    validate_buffers(input, output)?;
    let frame = canonical_frame(input);
    let digest = word(&frame, 0);
    let Some(signature) = signature_from_frame(&frame) else {
        return Ok(0);
    };
    let Some(public_key) = backend.recover_uncompressed_public_key(digest, signature) else {
        return Ok(0);
    };
    write_address_output(&mut hasher, &public_key, output)?;
    Ok(EVM_ECRECOVER_OUTPUT_BYTES)
}

fn validate_buffers(input: &[u8], output: &[u8]) -> Result<(), EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    output
        .get(..EVM_ECRECOVER_OUTPUT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    Ok(())
}

fn canonical_frame(input: &[u8]) -> [u8; EVM_ECRECOVER_INPUT_BYTES] {
    let mut frame = [0u8; EVM_ECRECOVER_INPUT_BYTES];
    let len = input.len().min(EVM_ECRECOVER_INPUT_BYTES);
    if let (Some(target), Some(source)) = (frame.get_mut(..len), input.get(..len)) {
        target.copy_from_slice(source);
    }
    frame
}

fn signature_from_frame(frame: &[u8; EVM_ECRECOVER_INPUT_BYTES]) -> Option<EvmEcRecoverSignature> {
    let v = word(frame, V_OFFSET);
    let y_parity = normalized_y_parity(v)?;
    let r = word(frame, R_OFFSET);
    let s = word(frame, S_OFFSET);
    if scalar_is_valid(r) && scalar_is_valid(s) {
        return Some(EvmEcRecoverSignature { r, s, y_parity });
    }
    None
}

fn normalized_y_parity(v: [u8; 32]) -> Option<u8> {
    let prefix_is_zero = v
        .get(..31)
        .is_some_and(|prefix| prefix.iter().all(|byte| *byte == 0));
    if !prefix_is_zero {
        return None;
    }
    match v.get(31).copied() {
        Some(27) => Some(0),
        Some(28) => Some(1),
        _ => None,
    }
}

fn scalar_is_valid(value: [u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0) && cmp_be(value, SECP256K1_ORDER_BE) == Ordering::Less
}

fn cmp_be(left: [u8; 32], right: [u8; 32]) -> Ordering {
    let mut ordering = Ordering::Equal;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        if ordering == Ordering::Equal {
            ordering = left_byte.cmp(right_byte);
        }
    }
    ordering
}

fn write_address_output<H>(
    hasher: &mut H,
    public_key: &[u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES],
    output: &mut [u8],
) -> Result<(), EvmCoreError>
where
    H: EvmPrecompileKeccak256,
{
    let digest = hasher.keccak256(public_key);
    let target = output
        .get_mut(..EVM_ECRECOVER_OUTPUT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    target.fill(0);
    if let (Some(address), Some(suffix)) = (
        digest.get(ADDRESS_OFFSET..),
        target.get_mut(ADDRESS_OFFSET..),
    ) {
        suffix.copy_from_slice(address);
    }
    Ok(())
}

fn word<const N: usize>(input: &[u8; N], offset: usize) -> [u8; 32] {
    let mut output = [0u8; 32];
    if let Some(source) = input.get(offset..offset.saturating_add(WORD_BYTES)) {
        output.copy_from_slice(source);
    }
    output
}
