use core::{iter::FusedIterator, slice::ChunksExact};

use crate::{
    EvmBls12381Fp, EvmBls12381Fp2, EvmBls12381G1Point, EvmBls12381G2Point, EvmBls12381Scalar,
    EvmCoreError, EvmPrecompileKind, advanced_precompile, precompile::validate_input_policy,
};

/// Byte length of one EIP-2537 G1 MSM item.
pub const EVM_BLS12381_G1_MSM_ITEM_BYTES: usize = 160;
/// Byte length of one EIP-2537 G2 MSM item.
pub const EVM_BLS12381_G2_MSM_ITEM_BYTES: usize = 288;
/// Byte length of one EIP-2537 pairing item.
pub const EVM_BLS12381_PAIRING_ITEM_BYTES: usize = 384;

const G1_POINT_BYTES: usize = crate::EVM_BLS12381_G1_POINT_BYTES;
const G2_POINT_BYTES: usize = crate::EVM_BLS12381_G2_POINT_BYTES;

/// Parsed EIP-2537 G1 addition frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G1AddInput {
    /// First canonical wire encoding, without a curve-validity claim.
    pub left: EvmBls12381G1Point,
    /// Second canonical wire encoding, without a curve-validity claim.
    pub right: EvmBls12381G1Point,
}

/// Parsed EIP-2537 G2 addition frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G2AddInput {
    /// First canonical wire encoding, without a curve-validity claim.
    pub left: EvmBls12381G2Point,
    /// Second canonical wire encoding, without a curve-validity claim.
    pub right: EvmBls12381G2Point,
}

/// One parsed EIP-2537 G1 MSM item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G1MsmItem {
    /// Canonical wire coordinates, without a curve-validity claim.
    pub point: EvmBls12381G1Point,
    /// Unrestricted 256-bit multiplication scalar.
    pub scalar: EvmBls12381Scalar,
}

/// One parsed EIP-2537 G2 MSM item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G2MsmItem {
    /// Canonical wire coordinates, without a curve-validity claim.
    pub point: EvmBls12381G2Point,
    /// Unrestricted 256-bit multiplication scalar.
    pub scalar: EvmBls12381Scalar,
}

/// One parsed EIP-2537 pairing item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381PairingItem {
    /// Canonical G1 wire coordinates, without a curve-validity claim.
    pub g1: EvmBls12381G1Point,
    /// Canonical G2 wire coordinates, without a curve-validity claim.
    pub g2: EvmBls12381G2Point,
}

/// Eagerly validated borrowed EIP-2537 G1 MSM frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G1MsmInput<'a>(&'a [u8]);

impl<'a> EvmBls12381G1MsmInput<'a> {
    /// Returns the positive item count.
    #[must_use]
    pub fn len(self) -> usize {
        self.0.len() / EVM_BLS12381_G1_MSM_ITEM_BYTES
    }

    /// Returns false for every successfully parsed non-empty frame.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    /// Returns a parser over every already-validated item.
    #[must_use]
    pub fn items(self) -> EvmBls12381G1MsmItems<'a> {
        EvmBls12381G1MsmItems(self.0.chunks_exact(EVM_BLS12381_G1_MSM_ITEM_BYTES))
    }
}

/// Iterator over a validated G1 MSM frame.
pub struct EvmBls12381G1MsmItems<'a>(ChunksExact<'a, u8>);

impl Iterator for EvmBls12381G1MsmItems<'_> {
    type Item = Result<EvmBls12381G1MsmItem, EvmCoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(parse_g1_msm_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for EvmBls12381G1MsmItems<'_> {}
impl FusedIterator for EvmBls12381G1MsmItems<'_> {}

/// Eagerly validated borrowed EIP-2537 G2 MSM frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G2MsmInput<'a>(&'a [u8]);

impl<'a> EvmBls12381G2MsmInput<'a> {
    /// Returns the positive item count.
    #[must_use]
    pub fn len(self) -> usize {
        self.0.len() / EVM_BLS12381_G2_MSM_ITEM_BYTES
    }

    /// Returns false for every successfully parsed non-empty frame.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    /// Returns a parser over every already-validated item.
    #[must_use]
    pub fn items(self) -> EvmBls12381G2MsmItems<'a> {
        EvmBls12381G2MsmItems(self.0.chunks_exact(EVM_BLS12381_G2_MSM_ITEM_BYTES))
    }
}

/// Iterator over a validated G2 MSM frame.
pub struct EvmBls12381G2MsmItems<'a>(ChunksExact<'a, u8>);

impl Iterator for EvmBls12381G2MsmItems<'_> {
    type Item = Result<EvmBls12381G2MsmItem, EvmCoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(parse_g2_msm_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for EvmBls12381G2MsmItems<'_> {}
impl FusedIterator for EvmBls12381G2MsmItems<'_> {}

/// Eagerly validated borrowed EIP-2537 pairing frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381PairingInput<'a>(&'a [u8]);

impl<'a> EvmBls12381PairingInput<'a> {
    /// Returns the positive item count.
    #[must_use]
    pub fn len(self) -> usize {
        self.0.len() / EVM_BLS12381_PAIRING_ITEM_BYTES
    }

    /// Returns false for every successfully parsed non-empty frame.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    /// Returns a parser over every already-validated item.
    #[must_use]
    pub fn items(self) -> EvmBls12381PairingItems<'a> {
        EvmBls12381PairingItems(self.0.chunks_exact(EVM_BLS12381_PAIRING_ITEM_BYTES))
    }
}

/// Iterator over a validated pairing frame.
pub struct EvmBls12381PairingItems<'a>(ChunksExact<'a, u8>);

impl Iterator for EvmBls12381PairingItems<'_> {
    type Item = Result<EvmBls12381PairingItem, EvmCoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(parse_pairing_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for EvmBls12381PairingItems<'_> {}
impl FusedIterator for EvmBls12381PairingItems<'_> {}

/// Parses the exact two-point G1 addition frame without claiming curve validity.
pub fn parse_bls12381_g1_add(input: &[u8]) -> Result<EvmBls12381G1AddInput, EvmCoreError> {
    validate(input, EvmPrecompileKind::Bls12G1Add)?;
    Ok(EvmBls12381G1AddInput {
        left: EvmBls12381G1Point::try_from_be_bytes(slice(input, 0, G1_POINT_BYTES)?)?,
        right: EvmBls12381G1Point::try_from_be_bytes(slice(
            input,
            G1_POINT_BYTES,
            G1_POINT_BYTES,
        )?)?,
    })
}

/// Parses a non-empty G1 MSM frame without claiming point curve validity.
pub fn parse_bls12381_g1_msm(input: &[u8]) -> Result<EvmBls12381G1MsmInput<'_>, EvmCoreError> {
    validate(input, EvmPrecompileKind::Bls12G1Msm)?;
    for item in input.chunks_exact(EVM_BLS12381_G1_MSM_ITEM_BYTES) {
        let _ = parse_g1_msm_item(item)?;
    }
    Ok(EvmBls12381G1MsmInput(input))
}

/// Parses the exact two-point G2 addition frame without claiming curve validity.
pub fn parse_bls12381_g2_add(input: &[u8]) -> Result<EvmBls12381G2AddInput, EvmCoreError> {
    validate(input, EvmPrecompileKind::Bls12G2Add)?;
    Ok(EvmBls12381G2AddInput {
        left: EvmBls12381G2Point::try_from_be_bytes(slice(input, 0, G2_POINT_BYTES)?)?,
        right: EvmBls12381G2Point::try_from_be_bytes(slice(
            input,
            G2_POINT_BYTES,
            G2_POINT_BYTES,
        )?)?,
    })
}

/// Parses a non-empty G2 MSM frame without claiming point curve validity.
pub fn parse_bls12381_g2_msm(input: &[u8]) -> Result<EvmBls12381G2MsmInput<'_>, EvmCoreError> {
    validate(input, EvmPrecompileKind::Bls12G2Msm)?;
    for item in input.chunks_exact(EVM_BLS12381_G2_MSM_ITEM_BYTES) {
        let _ = parse_g2_msm_item(item)?;
    }
    Ok(EvmBls12381G2MsmInput(input))
}

/// Parses a non-empty pairing frame without claiming point curve validity.
pub fn parse_bls12381_pairing(input: &[u8]) -> Result<EvmBls12381PairingInput<'_>, EvmCoreError> {
    validate(input, EvmPrecompileKind::Bls12PairingCheck)?;
    for item in input.chunks_exact(EVM_BLS12381_PAIRING_ITEM_BYTES) {
        let _ = parse_pairing_item(item)?;
    }
    Ok(EvmBls12381PairingInput(input))
}

/// Parses the exact map-to-G1 base-field frame.
pub fn parse_bls12381_map_fp_to_g1(input: &[u8]) -> Result<EvmBls12381Fp, EvmCoreError> {
    validate(input, EvmPrecompileKind::Bls12MapFpToG1)?;
    EvmBls12381Fp::try_from_be_bytes(input)
}

/// Parses the exact map-to-G2 extension-field frame.
pub fn parse_bls12381_map_fp2_to_g2(input: &[u8]) -> Result<EvmBls12381Fp2, EvmCoreError> {
    validate(input, EvmPrecompileKind::Bls12MapFp2ToG2)?;
    EvmBls12381Fp2::try_from_be_bytes(input)
}

fn parse_g1_msm_item(input: &[u8]) -> Result<EvmBls12381G1MsmItem, EvmCoreError> {
    Ok(EvmBls12381G1MsmItem {
        point: EvmBls12381G1Point::try_from_be_bytes(slice(input, 0, G1_POINT_BYTES)?)?,
        scalar: EvmBls12381Scalar::try_from_be_bytes(slice(
            input,
            G1_POINT_BYTES,
            crate::EVM_BLS12381_SCALAR_BYTES,
        )?)?,
    })
}

fn parse_g2_msm_item(input: &[u8]) -> Result<EvmBls12381G2MsmItem, EvmCoreError> {
    Ok(EvmBls12381G2MsmItem {
        point: EvmBls12381G2Point::try_from_be_bytes(slice(input, 0, G2_POINT_BYTES)?)?,
        scalar: EvmBls12381Scalar::try_from_be_bytes(slice(
            input,
            G2_POINT_BYTES,
            crate::EVM_BLS12381_SCALAR_BYTES,
        )?)?,
    })
}

fn parse_pairing_item(input: &[u8]) -> Result<EvmBls12381PairingItem, EvmCoreError> {
    Ok(EvmBls12381PairingItem {
        g1: EvmBls12381G1Point::try_from_be_bytes(slice(input, 0, G1_POINT_BYTES)?)?,
        g2: EvmBls12381G2Point::try_from_be_bytes(slice(input, G1_POINT_BYTES, G2_POINT_BYTES)?)?,
    })
}

fn validate(input: &[u8], kind: EvmPrecompileKind) -> Result<(), EvmCoreError> {
    validate_input_policy(advanced_precompile::input_policy(kind), input.len())
}

fn slice(input: &[u8], offset: usize, len: usize) -> Result<&[u8], EvmCoreError> {
    let end = offset
        .checked_add(len)
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    input
        .get(offset..end)
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)
}
