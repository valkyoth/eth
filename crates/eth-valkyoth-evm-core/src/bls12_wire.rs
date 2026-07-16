use crate::EvmCoreError;

/// Byte length of one EIP-2537 base-field encoding.
pub const EVM_BLS12381_FP_BYTES: usize = 64;
/// Byte length of one canonical BLS12-381 scalar-field encoding.
pub const EVM_BLS12381_FR_BYTES: usize = 32;
/// Byte length of one EIP-2537 multiplication scalar encoding.
pub const EVM_BLS12381_SCALAR_BYTES: usize = 32;
/// Byte length of one EIP-2537 quadratic-extension field encoding.
pub const EVM_BLS12381_FP2_BYTES: usize = 128;
/// Byte length of one uncompressed EIP-2537 G1 point encoding.
pub const EVM_BLS12381_G1_POINT_BYTES: usize = 128;
/// Byte length of one uncompressed EIP-2537 G2 point encoding.
pub const EVM_BLS12381_G2_POINT_BYTES: usize = 256;

const FP_VALUE_BYTES: usize = 48;
const FP_PADDING_BYTES: usize = EVM_BLS12381_FP_BYTES - FP_VALUE_BYTES;

const FP_MODULUS: [u8; FP_VALUE_BYTES] = [
    0x1a, 0x01, 0x11, 0xea, 0x39, 0x7f, 0xe6, 0x9a, 0x4b, 0x1b, 0xa7, 0xb6, 0x43, 0x4b, 0xac, 0xd7,
    0x64, 0x77, 0x4b, 0x84, 0xf3, 0x85, 0x12, 0xbf, 0x67, 0x30, 0xd2, 0xa0, 0xf6, 0xb0, 0xf6, 0x24,
    0x1e, 0xab, 0xff, 0xfe, 0xb1, 0x53, 0xff, 0xff, 0xb9, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xaa, 0xab,
];

const FR_MODULUS: [u8; EVM_BLS12381_FR_BYTES] = [
    0x73, 0xed, 0xa7, 0x53, 0x29, 0x9d, 0x7d, 0x48, 0x33, 0x39, 0xd8, 0x08, 0x09, 0xa1, 0xd8, 0x05,
    0x53, 0xbd, 0xa4, 0x02, 0xff, 0xfe, 0x5b, 0xfe, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x01,
];

/// Canonical BLS12-381 base-field value decoded from EIP-2537 wire bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381Fp([u8; FP_VALUE_BYTES]);

impl EvmBls12381Fp {
    /// Decodes one canonical 64-byte EIP-2537 base-field element.
    pub fn try_from_be_bytes(input: &[u8]) -> Result<Self, EvmCoreError> {
        let encoded: &[u8; EVM_BLS12381_FP_BYTES] = input
            .try_into()
            .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?;
        if encoded[..FP_PADDING_BYTES].iter().any(|byte| *byte != 0) {
            return Err(EvmCoreError::PrecompileFieldElementOutOfRange);
        }
        let value: [u8; FP_VALUE_BYTES] = encoded[FP_PADDING_BYTES..]
            .try_into()
            .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?;
        if value >= FP_MODULUS {
            return Err(EvmCoreError::PrecompileFieldElementOutOfRange);
        }
        Ok(Self(value))
    }

    /// Returns the canonical 48-byte integer value without wire padding.
    #[must_use]
    pub const fn value_bytes(&self) -> &[u8; FP_VALUE_BYTES] {
        &self.0
    }

    /// Returns the canonical 64-byte EIP-2537 wire encoding.
    #[must_use]
    pub fn to_be_bytes(self) -> [u8; EVM_BLS12381_FP_BYTES] {
        let mut output = [0u8; EVM_BLS12381_FP_BYTES];
        output[FP_PADDING_BYTES..].copy_from_slice(&self.0);
        output
    }

    /// Returns whether this field value is zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0.iter().all(|byte| *byte == 0)
    }
}

/// Canonical scalar-field value strictly below the BLS12-381 subgroup order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381Fr([u8; EVM_BLS12381_FR_BYTES]);

impl EvmBls12381Fr {
    /// Decodes a canonical scalar-field value.
    pub fn try_from_be_bytes(input: &[u8]) -> Result<Self, EvmCoreError> {
        let value: [u8; EVM_BLS12381_FR_BYTES] = input
            .try_into()
            .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?;
        if value >= FR_MODULUS {
            return Err(EvmCoreError::PrecompileFieldElementOutOfRange);
        }
        Ok(Self(value))
    }

    /// Returns the canonical scalar-field bytes.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; EVM_BLS12381_FR_BYTES] {
        self.0
    }
}

/// Full-width EIP-2537 multiplication scalar.
///
/// Unlike [`EvmBls12381Fr`], this wire domain deliberately accepts every
/// 256-bit value because EIP-2537 does not restrict MSM scalars to the subgroup
/// order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381Scalar([u8; EVM_BLS12381_SCALAR_BYTES]);

impl EvmBls12381Scalar {
    /// Decodes any exact 32-byte big-endian multiplication scalar.
    pub fn try_from_be_bytes(input: &[u8]) -> Result<Self, EvmCoreError> {
        input
            .try_into()
            .map(Self)
            .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)
    }

    /// Returns the full-width scalar bytes.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; EVM_BLS12381_SCALAR_BYTES] {
        self.0
    }
}

/// Canonical EIP-2537 Fp2 value in `c0 || c1` coefficient order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381Fp2 {
    c0: EvmBls12381Fp,
    c1: EvmBls12381Fp,
}

impl EvmBls12381Fp2 {
    /// Decodes one exact EIP-2537 `c0 || c1` field encoding.
    pub fn try_from_be_bytes(input: &[u8]) -> Result<Self, EvmCoreError> {
        require_len(input, EVM_BLS12381_FP2_BYTES)?;
        let c0 = EvmBls12381Fp::try_from_be_bytes(slice(input, 0, EVM_BLS12381_FP_BYTES)?)?;
        let c1 = EvmBls12381Fp::try_from_be_bytes(slice(
            input,
            EVM_BLS12381_FP_BYTES,
            EVM_BLS12381_FP_BYTES,
        )?)?;
        Ok(Self { c0, c1 })
    }

    /// Returns the constant coefficient.
    #[must_use]
    pub const fn c0(self) -> EvmBls12381Fp {
        self.c0
    }

    /// Returns the linear coefficient.
    #[must_use]
    pub const fn c1(self) -> EvmBls12381Fp {
        self.c1
    }

    /// Returns the exact `c0 || c1` wire encoding.
    #[must_use]
    pub fn to_be_bytes(self) -> [u8; EVM_BLS12381_FP2_BYTES] {
        let mut output = [0u8; EVM_BLS12381_FP2_BYTES];
        output[..EVM_BLS12381_FP_BYTES].copy_from_slice(&self.c0.to_be_bytes());
        output[EVM_BLS12381_FP_BYTES..].copy_from_slice(&self.c1.to_be_bytes());
        output
    }

    /// Returns whether both coefficients are zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.c0.is_zero() && self.c1.is_zero()
    }
}

/// Canonically encoded, not-yet-curve-validated EIP-2537 G1 point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G1Point {
    x: EvmBls12381Fp,
    y: EvmBls12381Fp,
    infinity: bool,
}

impl EvmBls12381G1Point {
    /// Decodes canonical coordinates and the all-zero infinity encoding.
    pub fn try_from_be_bytes(input: &[u8]) -> Result<Self, EvmCoreError> {
        require_len(input, EVM_BLS12381_G1_POINT_BYTES)?;
        let x = EvmBls12381Fp::try_from_be_bytes(slice(input, 0, EVM_BLS12381_FP_BYTES)?)?;
        let y = EvmBls12381Fp::try_from_be_bytes(slice(
            input,
            EVM_BLS12381_FP_BYTES,
            EVM_BLS12381_FP_BYTES,
        )?)?;
        Ok(Self {
            x,
            y,
            infinity: x.is_zero() && y.is_zero(),
        })
    }

    /// Returns whether this is the unique all-zero infinity encoding.
    #[must_use]
    pub const fn is_infinity(self) -> bool {
        self.infinity
    }

    /// Returns coordinates for a non-infinity encoding.
    #[must_use]
    pub const fn coordinates(self) -> Option<(EvmBls12381Fp, EvmBls12381Fp)> {
        if self.infinity {
            None
        } else {
            Some((self.x, self.y))
        }
    }

    /// Returns the canonical uncompressed point encoding.
    #[must_use]
    pub fn to_be_bytes(self) -> [u8; EVM_BLS12381_G1_POINT_BYTES] {
        let mut output = [0u8; EVM_BLS12381_G1_POINT_BYTES];
        if !self.infinity {
            output[..EVM_BLS12381_FP_BYTES].copy_from_slice(&self.x.to_be_bytes());
            output[EVM_BLS12381_FP_BYTES..].copy_from_slice(&self.y.to_be_bytes());
        }
        output
    }
}

/// Canonically encoded, not-yet-curve-validated EIP-2537 G2 point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G2Point {
    x: EvmBls12381Fp2,
    y: EvmBls12381Fp2,
    infinity: bool,
}

impl EvmBls12381G2Point {
    /// Decodes canonical Fp2 coordinates and the all-zero infinity encoding.
    pub fn try_from_be_bytes(input: &[u8]) -> Result<Self, EvmCoreError> {
        require_len(input, EVM_BLS12381_G2_POINT_BYTES)?;
        let x = EvmBls12381Fp2::try_from_be_bytes(slice(input, 0, EVM_BLS12381_FP2_BYTES)?)?;
        let y = EvmBls12381Fp2::try_from_be_bytes(slice(
            input,
            EVM_BLS12381_FP2_BYTES,
            EVM_BLS12381_FP2_BYTES,
        )?)?;
        Ok(Self {
            x,
            y,
            infinity: x.is_zero() && y.is_zero(),
        })
    }

    /// Returns whether this is the unique all-zero infinity encoding.
    #[must_use]
    pub const fn is_infinity(self) -> bool {
        self.infinity
    }

    /// Returns coordinates for a non-infinity encoding.
    #[must_use]
    pub const fn coordinates(self) -> Option<(EvmBls12381Fp2, EvmBls12381Fp2)> {
        if self.infinity {
            None
        } else {
            Some((self.x, self.y))
        }
    }

    /// Returns the canonical uncompressed point encoding.
    #[must_use]
    pub fn to_be_bytes(self) -> [u8; EVM_BLS12381_G2_POINT_BYTES] {
        let mut output = [0u8; EVM_BLS12381_G2_POINT_BYTES];
        if !self.infinity {
            output[..EVM_BLS12381_FP2_BYTES].copy_from_slice(&self.x.to_be_bytes());
            output[EVM_BLS12381_FP2_BYTES..].copy_from_slice(&self.y.to_be_bytes());
        }
        output
    }
}

fn require_len(input: &[u8], expected: usize) -> Result<(), EvmCoreError> {
    if input.len() == expected {
        Ok(())
    } else {
        Err(EvmCoreError::PrecompileInvalidInputLength)
    }
}

fn slice(input: &[u8], offset: usize, len: usize) -> Result<&[u8], EvmCoreError> {
    let end = offset
        .checked_add(len)
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    input
        .get(offset..end)
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)
}
