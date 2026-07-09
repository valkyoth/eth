use crate::{
    bn254_field::Fp,
    bn254_g2::{Fp2, XI_TO_P_MINUS_1_OVER_3},
};

const XI_TO_P_MINUS_1_OVER_6: Fp2 = Fp2 {
    c0: Fp::from_montgomery_limbs([
        0xaf9b_a696_3314_4907,
        0xca6b_1d73_87af_b78a,
        0x11bd_ed5e_f08a_2087,
        0x02f3_4d75_1a1f_3a7c,
    ]),
    c1: Fp::from_montgomery_limbs([
        0xa222_ae23_4c49_2d72,
        0xd00f_02a4_565d_e15b,
        0xdc2f_f3a2_53df_c926,
        0x10a7_5716_b389_9551,
    ]),
};

const XI_TO_2P_MINUS_2_OVER_3: Fp2 = Fp2 {
    c0: Fp::from_montgomery_limbs([
        0x7361_d77f_843a_be92,
        0xa5bb_2bd3_2734_11fb,
        0x9c94_1f31_4b3e_2399,
        0x15df_9cdd_bb9f_d3ec,
    ]),
    c1: Fp::from_montgomery_limbs([
        0x5ddd_fd15_4bd8_c949,
        0x62cb_29a5_a444_5b60,
        0x37bc_870a_0c7d_d2b9,
        0x2483_0a9d_3171_f0fd,
    ]),
};

const XI_TO_P_SQUARED_MINUS_1_OVER_6: Fp = Fp::from_montgomery_limbs([
    0xca8d_8005_00fa_1bf2,
    0xf0c5_d614_68b3_9769,
    0x0e20_1271_ad0d_4418,
    0x0429_0f65_bad8_56e6,
]);

const XI_TO_P_SQUARED_MINUS_1_OVER_3: Fp = Fp::from_montgomery_limbs([
    0x3350_c88e_13e8_0b9c,
    0x7dce_557c_db5e_56b9,
    0x6001_b4b8_b615_564a,
    0x2682_e617_0202_17e0,
]);

const XI_TO_2P_SQUARED_MINUS_2_OVER_3: Fp = Fp::from_montgomery_limbs([
    0x7193_0c11_d782_e155,
    0xa6bb_947c_ffbe_3323,
    0xaa30_3344_d474_1444,
    0x2c3b_3f0d_2659_4943,
]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Fp6 {
    pub(crate) c0: Fp2,
    pub(crate) c1: Fp2,
    pub(crate) c2: Fp2,
}

impl Fp6 {
    pub(crate) const ZERO: Self = Self {
        c0: Fp2::ZERO,
        c1: Fp2::ZERO,
        c2: Fp2::ZERO,
    };
    pub(crate) const ONE: Self = Self {
        c0: Fp2::ONE,
        c1: Fp2::ZERO,
        c2: Fp2::ZERO,
    };
    pub(crate) fn add(self, rhs: Self) -> Self {
        Self {
            c0: self.c0.add(rhs.c0),
            c1: self.c1.add(rhs.c1),
            c2: self.c2.add(rhs.c2),
        }
    }

    pub(crate) fn sub(self, rhs: Self) -> Self {
        Self {
            c0: self.c0.sub(rhs.c0),
            c1: self.c1.sub(rhs.c1),
            c2: self.c2.sub(rhs.c2),
        }
    }

    pub(crate) fn neg(self) -> Self {
        Self::ZERO.sub(self)
    }

    pub(crate) fn mul(self, rhs: Self) -> Self {
        let a0b0 = self.c0.mul(rhs.c0);
        let a0b1 = self.c0.mul(rhs.c1);
        let a0b2 = self.c0.mul(rhs.c2);
        let a1b0 = self.c1.mul(rhs.c0);
        let a1b1 = self.c1.mul(rhs.c1);
        let a1b2 = self.c1.mul(rhs.c2);
        let a2b0 = self.c2.mul(rhs.c0);
        let a2b1 = self.c2.mul(rhs.c1);
        let a2b2 = self.c2.mul(rhs.c2);
        Self {
            c0: a0b0.add(mul_fp2_by_nonresidue(a1b2.add(a2b1))),
            c1: a0b1.add(a1b0).add(mul_fp2_by_nonresidue(a2b2)),
            c2: a0b2.add(a1b1).add(a2b0),
        }
    }

    pub(crate) fn square(self) -> Self {
        self.mul(self)
    }

    pub(crate) fn invert(self) -> Option<Self> {
        let c0 = self
            .c0
            .square()
            .sub(mul_fp2_by_nonresidue(self.c1.mul(self.c2)));
        let c1 = mul_fp2_by_nonresidue(self.c2.square()).sub(self.c0.mul(self.c1));
        let c2 = self.c1.square().sub(self.c0.mul(self.c2));
        let norm = mul_fp2_by_nonresidue(self.c2.mul(c1).add(self.c1.mul(c2))).add(self.c0.mul(c0));
        let norm_inv = norm.invert()?;
        Some(Self {
            c0: c0.mul(norm_inv),
            c1: c1.mul(norm_inv),
            c2: c2.mul(norm_inv),
        })
    }

    pub(crate) fn mul_by_v(self) -> Self {
        Self {
            c0: mul_fp2_by_nonresidue(self.c2),
            c1: self.c0,
            c2: self.c1,
        }
    }

    pub(crate) fn mul_by_fp2(self, rhs: Fp2) -> Self {
        Self {
            c0: self.c0.mul(rhs),
            c1: self.c1.mul(rhs),
            c2: self.c2.mul(rhs),
        }
    }

    pub(crate) fn mul_by_fp(self, rhs: Fp) -> Self {
        Self {
            c0: self.c0.mul_by_fp(rhs),
            c1: self.c1.mul_by_fp(rhs),
            c2: self.c2.mul_by_fp(rhs),
        }
    }

    pub(crate) fn frobenius(self) -> Self {
        Self {
            c0: self.c0.conjugate(),
            c1: self.c1.conjugate().mul(XI_TO_P_MINUS_1_OVER_3),
            c2: self.c2.conjugate().mul(XI_TO_2P_MINUS_2_OVER_3),
        }
    }

    pub(crate) fn frobenius_p2(self) -> Self {
        Self {
            c0: self.c0,
            c1: self.c1.mul_by_fp(XI_TO_P_SQUARED_MINUS_1_OVER_3),
            c2: self.c2.mul_by_fp(XI_TO_2P_SQUARED_MINUS_2_OVER_3),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Fp12 {
    pub(crate) c0: Fp6,
    pub(crate) c1: Fp6,
}

impl Fp12 {
    #[cfg(test)]
    pub(crate) const ZERO: Self = Self {
        c0: Fp6::ZERO,
        c1: Fp6::ZERO,
    };
    pub(crate) const ONE: Self = Self {
        c0: Fp6::ONE,
        c1: Fp6::ZERO,
    };
    #[cfg(test)]
    pub(crate) fn add(self, rhs: Self) -> Self {
        Self {
            c0: self.c0.add(rhs.c0),
            c1: self.c1.add(rhs.c1),
        }
    }

    pub(crate) fn mul(self, rhs: Self) -> Self {
        let a0b0 = self.c0.mul(rhs.c0);
        let a1b1 = self.c1.mul(rhs.c1);
        Self {
            c0: a0b0.add(a1b1.mul_by_v()),
            c1: self.c0.mul(rhs.c1).add(self.c1.mul(rhs.c0)),
        }
    }

    #[cfg(test)]
    pub(crate) fn mul_by_fp6(self, rhs: Fp6) -> Self {
        Self {
            c0: self.c0.mul(rhs),
            c1: self.c1.mul(rhs),
        }
    }

    pub(crate) fn square(self) -> Self {
        self.mul(self)
    }

    pub(crate) fn invert(self) -> Option<Self> {
        let denominator = self.c0.square().sub(self.c1.square().mul_by_v());
        let denominator = denominator.invert()?;
        Some(Self {
            c0: self.c0.mul(denominator),
            c1: self.c1.neg().mul(denominator),
        })
    }

    pub(crate) fn conjugate(self) -> Self {
        Self {
            c0: self.c0,
            c1: self.c1.neg(),
        }
    }

    pub(crate) fn frobenius(self) -> Self {
        Self {
            c0: self.c0.frobenius(),
            c1: self.c1.frobenius().mul_by_fp2(XI_TO_P_MINUS_1_OVER_6),
        }
    }

    pub(crate) fn frobenius_p2(self) -> Self {
        Self {
            c0: self.c0.frobenius_p2(),
            c1: self
                .c1
                .frobenius_p2()
                .mul_by_fp(XI_TO_P_SQUARED_MINUS_1_OVER_6),
        }
    }

    pub(crate) fn frobenius_p6(self) -> Self {
        Self {
            c0: self.c0,
            c1: self.c1.neg(),
        }
    }

    pub(crate) fn pow_little_endian_limbs<const N: usize>(self, exponent: [u64; N]) -> Self {
        let mut result = Self::ONE;
        let mut base = self;
        for limb in exponent {
            let mut bits = limb;
            for _ in 0..64 {
                if bits & 1 == 1 {
                    result = result.mul(base);
                }
                base = base.square();
                bits >>= 1;
            }
        }
        result
    }
}

fn mul_fp2_by_nonresidue(value: Fp2) -> Fp2 {
    value.mul(Fp2::NINE_PLUS_I)
}
