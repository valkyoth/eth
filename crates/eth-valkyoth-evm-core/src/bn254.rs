use crate::{EVM_PRECOMPILE_INPUT_LIMIT, EvmCoreError, EvmPrecompileKind, EvmPrecompilePlan};

use crate::bn254_field::Fp;

/// Byte length of one BN254 affine point in the EIP-196 encoding.
pub const EVM_BN254_POINT_BYTES: usize = 64;
/// Byte length of the BN254 addition precompile input frame.
pub const EVM_BN254_ADD_INPUT_BYTES: usize = 128;
/// Byte length of the BN254 multiplication precompile input frame.
pub const EVM_BN254_MUL_INPUT_BYTES: usize = 96;

/// Executes the EIP-196 BN254 point-addition precompile.
///
/// # Security
///
/// This function implements public EVM precompile arithmetic. It is not
/// constant-time and must not be reused for secret-dependent private-key
/// operations.
pub fn execute_bn254_add(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    let target = output
        .get_mut(..EVM_BN254_POINT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    let left = read_point(input, 0)?;
    let right = read_point(input, EVM_BN254_POINT_BYTES)?;
    write_point(left.add(right), target);
    Ok(EVM_BN254_POINT_BYTES)
}

/// Executes the EIP-196 BN254 scalar-multiplication precompile.
///
/// # Security
///
/// This function implements public EVM precompile arithmetic. It is not
/// constant-time and must not be reused for secret-dependent private-key
/// operations.
pub fn execute_bn254_mul(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    let target = output
        .get_mut(..EVM_BN254_POINT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    let point = read_point(input, 0)?;
    let scalar = read_word(input, EVM_BN254_POINT_BYTES);
    write_point(point.mul_scalar(scalar), target);
    Ok(EVM_BN254_POINT_BYTES)
}

impl EvmPrecompilePlan {
    /// Executes the EIP-196 BN254 point-addition precompile into `output`.
    pub fn execute_bn254_add(self, input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Bn254Add {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_bn254_add(input, output)
    }

    /// Executes the EIP-196 BN254 scalar-multiplication precompile into `output`.
    pub fn execute_bn254_mul(self, input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Bn254Mul {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_bn254_mul(input, output)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Point {
    x: Fp,
    y: Fp,
    infinity: bool,
}

impl Point {
    const INFINITY: Self = Self {
        x: Fp::ZERO,
        y: Fp::ZERO,
        infinity: true,
    };

    fn new(x: Fp, y: Fp) -> Result<Self, EvmCoreError> {
        if x.is_zero() && y.is_zero() {
            return Ok(Self::INFINITY);
        }
        let point = Self {
            x,
            y,
            infinity: false,
        };
        if point.is_on_curve() {
            Ok(point)
        } else {
            Err(EvmCoreError::PrecompilePointNotOnCurve)
        }
    }

    fn is_on_curve(self) -> bool {
        self.infinity || self.y.square() == self.x.square().mul(self.x).add(Fp::THREE)
    }

    fn add(self, rhs: Self) -> Self {
        ProjectivePoint::from_affine(self)
            .add_mixed(rhs)
            .to_affine()
    }

    fn mul_scalar(self, scalar: [u8; 32]) -> Self {
        let mut acc = ProjectivePoint::INFINITY;
        for bit in 0..256 {
            acc = acc.double();
            if scalar_bit(&scalar, bit) {
                acc = acc.add_mixed(self);
            }
        }
        acc.to_affine()
    }
}

#[derive(Clone, Copy)]
struct ProjectivePoint {
    x: Fp,
    y: Fp,
    z: Fp,
}

impl ProjectivePoint {
    const INFINITY: Self = Self {
        x: Fp::ZERO,
        y: Fp::ONE,
        z: Fp::ZERO,
    };

    fn from_affine(point: Point) -> Self {
        if point.infinity {
            Self::INFINITY
        } else {
            Self {
                x: point.x,
                y: point.y,
                z: Fp::ONE,
            }
        }
    }

    fn is_infinity(self) -> bool {
        self.z.is_zero()
    }

    fn double(self) -> Self {
        if self.is_infinity() || self.y.is_zero() {
            return Self::INFINITY;
        }
        let a = self.x.square();
        let b = self.y.square();
        let c = b.square();
        let x_plus_b = self.x.add(b);
        let d = x_plus_b.square().sub(a).sub(c).double();
        let e = a.add(a).add(a);
        let f = e.square();
        let x3 = f.sub(d.double());
        let y3 = e.mul(d.sub(x3)).sub(c.double().double().double());
        let z3 = self.y.mul(self.z).double();
        Self {
            x: x3,
            y: y3,
            z: z3,
        }
    }

    fn add_mixed(self, rhs: Point) -> Self {
        if rhs.infinity {
            return self;
        }
        if self.is_infinity() {
            return Self::from_affine(rhs);
        }
        let z1z1 = self.z.square();
        let u2 = rhs.x.mul(z1z1);
        let s2 = rhs.y.mul(self.z).mul(z1z1);
        let h = u2.sub(self.x);
        if h.is_zero() {
            if s2 == self.y {
                return self.double();
            }
            return Self::INFINITY;
        }
        let hh = h.square();
        let i = hh.double().double();
        let j = h.mul(i);
        let r = s2.sub(self.y).double();
        let v = self.x.mul(i);
        let x3 = r.square().sub(j).sub(v.double());
        let y3 = r.mul(v.sub(x3)).sub(self.y.mul(j).double());
        let z3 = self.z.add(h).square().sub(z1z1).sub(hh);
        Self {
            x: x3,
            y: y3,
            z: z3,
        }
    }

    fn to_affine(self) -> Point {
        if self.is_infinity() {
            return Point::INFINITY;
        }
        let Some(z_inv) = self.z.invert() else {
            return Point::INFINITY;
        };
        let z2 = z_inv.square();
        let z3 = z2.mul(z_inv);
        Point {
            x: self.x.mul(z2),
            y: self.y.mul(z3),
            infinity: false,
        }
    }
}

fn read_point(input: &[u8], offset: usize) -> Result<Point, EvmCoreError> {
    let x = Fp::from_be_bytes(read_word(input, offset))
        .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
    let y = Fp::from_be_bytes(read_word(input, offset.saturating_add(32)))
        .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
    Point::new(x, y)
}

pub(crate) fn validate_g1_point(input: &[u8], offset: usize) -> Result<(), EvmCoreError> {
    read_point(input, offset).map(|_| ())
}

fn write_point(point: Point, output: &mut [u8]) {
    if point.infinity {
        output.fill(0);
        return;
    }
    let (x, y) = output.split_at_mut(32);
    x.copy_from_slice(&point.x.to_be_bytes());
    y.copy_from_slice(&point.y.to_be_bytes());
}

fn read_word(input: &[u8], offset: usize) -> [u8; 32] {
    let mut output = [0u8; 32];
    if let Some(available) = input.get(offset..) {
        let len = available.len().min(32);
        if let (Some(source), Some(target)) = (available.get(..len), output.get_mut(..len)) {
            target.copy_from_slice(source);
        }
    }
    output
}

fn scalar_bit(scalar: &[u8; 32], bit_from_high: usize) -> bool {
    let byte = bit_from_high / 8;
    let offset = 7usize.saturating_sub(bit_from_high % 8);
    scalar
        .get(byte)
        .copied()
        .map(|value| ((value >> offset) & 1) == 1)
        .unwrap_or(false)
}
