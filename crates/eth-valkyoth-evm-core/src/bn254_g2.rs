use crate::{EvmCoreError, bn254_field::Fp};

const BN254_GROUP_ORDER: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00, 0x00, 0x01,
];

const BN254_TWIST_B: Fp2 = Fp2 {
    c0: Fp::from_montgomery_limbs([
        0x3bf9_38e3_77b8_02a8,
        0x020b_1b27_3633_535d,
        0x26b7_edf0_4975_5260,
        0x2514_c632_4384_a86d,
    ]),
    c1: Fp::from_montgomery_limbs([
        0x38e7_eccc_d1dc_ff67,
        0x65f0_b37d_93ce_0d3e,
        0xd749_d0dd_22ac_00aa,
        0x0141_b9ce_4a68_8d4d,
    ]),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Fp2 {
    pub(crate) c0: Fp,
    pub(crate) c1: Fp,
}

impl Fp2 {
    pub(crate) const ZERO: Self = Self {
        c0: Fp::ZERO,
        c1: Fp::ZERO,
    };
    pub(crate) const ONE: Self = Self {
        c0: Fp::ONE,
        c1: Fp::ZERO,
    };
    pub(crate) const NINE_PLUS_I: Self = Self {
        c0: Fp::from_montgomery_limbs([
            0xf606_47ce_410d_7ff7,
            0x2f3d_6f4d_d31b_d011,
            0x2943_337e_3940_c6d1,
            0x1d95_98e8_a7e3_9857,
        ]),
        c1: Fp::ONE,
    };

    pub(crate) fn is_zero(self) -> bool {
        self.c0.is_zero() && self.c1.is_zero()
    }

    pub(crate) fn add(self, rhs: Self) -> Self {
        Self {
            c0: self.c0.add(rhs.c0),
            c1: self.c1.add(rhs.c1),
        }
    }

    pub(crate) fn sub(self, rhs: Self) -> Self {
        Self {
            c0: self.c0.sub(rhs.c0),
            c1: self.c1.sub(rhs.c1),
        }
    }

    pub(crate) fn double(self) -> Self {
        self.add(self)
    }

    pub(crate) fn mul(self, rhs: Self) -> Self {
        let ac = self.c0.mul(rhs.c0);
        let bd = self.c1.mul(rhs.c1);
        let ad = self.c0.mul(rhs.c1);
        let bc = self.c1.mul(rhs.c0);
        Self {
            c0: ac.sub(bd),
            c1: ad.add(bc),
        }
    }

    pub(crate) fn square(self) -> Self {
        let c0c1 = self.c0.mul(self.c1);
        Self {
            c0: self.c0.square().sub(self.c1.square()),
            c1: c0c1.double(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct G2Point {
    x: Fp2,
    y: Fp2,
    infinity: bool,
}

impl G2Point {
    const INFINITY: Self = Self {
        x: Fp2::ZERO,
        y: Fp2::ZERO,
        infinity: true,
    };

    fn new(x: Fp2, y: Fp2) -> Result<Self, EvmCoreError> {
        if x.is_zero() && y.is_zero() {
            return Ok(Self::INFINITY);
        }
        let point = Self {
            x,
            y,
            infinity: false,
        };
        if !point.is_on_curve() {
            return Err(EvmCoreError::PrecompilePointNotOnCurve);
        }
        if !point.is_in_subgroup() {
            return Err(EvmCoreError::PrecompilePointNotInSubgroup);
        }
        Ok(point)
    }

    fn is_on_curve(self) -> bool {
        self.infinity || self.y.square() == self.x.square().mul(self.x).add(BN254_TWIST_B)
    }

    fn is_in_subgroup(self) -> bool {
        // This is intentionally ordinary variable-time arithmetic over public
        // calldata and the fixed public BN254 group order. Do not reuse it for
        // secret scalar multiplication.
        ProjectiveG2Point::from_affine(self)
            .mul_scalar(BN254_GROUP_ORDER)
            .is_infinity()
    }
}

#[derive(Clone, Copy)]
struct ProjectiveG2Point {
    x: Fp2,
    y: Fp2,
    z: Fp2,
}

impl ProjectiveG2Point {
    const INFINITY: Self = Self {
        x: Fp2::ZERO,
        y: Fp2 {
            c0: Fp::ONE,
            c1: Fp::ZERO,
        },
        z: Fp2::ZERO,
    };

    fn from_affine(point: G2Point) -> Self {
        if point.infinity {
            Self::INFINITY
        } else {
            Self {
                x: point.x,
                y: point.y,
                z: Fp2 {
                    c0: Fp::ONE,
                    c1: Fp::ZERO,
                },
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

    fn add_mixed(self, rhs: G2Point) -> Self {
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

    fn mul_scalar(self, scalar: [u8; 32]) -> Self {
        let affine = G2Point {
            x: self.x,
            y: self.y,
            infinity: self.is_infinity(),
        };
        let mut acc = Self::INFINITY;
        for bit in 0..256 {
            acc = acc.double();
            if scalar_bit(&scalar, bit) {
                acc = acc.add_mixed(affine);
            }
        }
        acc
    }
}

pub(crate) fn read_g2_point(input: &[u8], offset: usize) -> Result<G2Point, EvmCoreError> {
    let x = read_fp2(input, offset)?;
    let y = read_fp2(input, offset.saturating_add(64))?;
    G2Point::new(x, y)
}

fn read_fp2(input: &[u8], offset: usize) -> Result<Fp2, EvmCoreError> {
    let imaginary = read_fp(input, offset)?;
    let real = read_fp(input, offset.saturating_add(32))?;
    Ok(Fp2 {
        c0: real,
        c1: imaginary,
    })
}

fn read_fp(input: &[u8], offset: usize) -> Result<Fp, EvmCoreError> {
    Fp::from_be_bytes(read_word(input, offset))
        .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)
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
