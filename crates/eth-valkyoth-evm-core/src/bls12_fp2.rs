//! Fixed-width public-input arithmetic in Fp[v]/(v^2 + 1).

use crate::{EvmBls12381Fp, EvmBls12381Fp2, bls12_field::Fp};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct Fp2 {
    c0: Fp,
    c1: Fp,
}

impl Fp2 {
    pub(crate) fn from_wire(value: EvmBls12381Fp2) -> Self {
        Self {
            c0: Fp::from_wire(value.c0()),
            c1: Fp::from_wire(value.c1()),
        }
    }

    pub(crate) fn to_wire(self) -> EvmBls12381Fp2 {
        EvmBls12381Fp2::from_coefficients(
            EvmBls12381Fp::from_field(self.c0),
            EvmBls12381Fp::from_field(self.c1),
        )
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

    pub(crate) fn negate(self) -> Self {
        let zero = Fp::from_u64(0);
        Self {
            c0: zero.sub(self.c0),
            c1: zero.sub(self.c1),
        }
    }

    pub(crate) fn conjugate(self) -> Self {
        Self {
            c0: self.c0,
            c1: Fp::from_u64(0).sub(self.c1),
        }
    }

    pub(crate) fn mul(self, rhs: Self) -> Self {
        let ac = self.c0.mul(rhs.c0);
        let bd = self.c1.mul(rhs.c1);
        Self {
            c0: ac.sub(bd),
            c1: self.c0.add(self.c1).mul(rhs.c0.add(rhs.c1)).sub(ac).sub(bd),
        }
    }

    pub(crate) fn square(self) -> Self {
        let ab = self.c0.mul(self.c1);
        Self {
            c0: self.c0.add(self.c1).mul(self.c0.sub(self.c1)),
            c1: ab.add(ab),
        }
    }

    fn norm(self) -> Fp {
        self.c0.square().add(self.c1.square())
    }

    pub(crate) fn invert(self) -> Option<Self> {
        // -1 is a nonsquare in Fp, so the norm is zero iff both coefficients are.
        let inverse = self.norm().invert()?;
        let conjugate = self.conjugate();
        Some(Self {
            c0: conjugate.c0.mul(inverse),
            c1: conjugate.c1.mul(inverse),
        })
    }

    pub(crate) fn sqrt(self) -> Option<Self> {
        let zero = Fp::from_u64(0);
        let root = if self.c1 == zero {
            // Includes zero; a base-field nonsquare has a purely imaginary root.
            if let Some(real) = self.c0.sqrt() {
                Self { c0: real, c1: zero }
            } else {
                Self {
                    c0: zero,
                    c1: zero.sub(self.c0).sqrt()?,
                }
            }
        } else {
            // For (x+y*v)^2=a+b*v, x^2=(a +/- sqrt(a^2+b^2))/2.
            // Nonzero b guarantees that a successful x is nonzero.
            let norm_root = self.norm().sqrt()?;
            let half = Fp::from_u64(2).invert()?;
            let x = self
                .c0
                .add(norm_root)
                .mul(half)
                .sqrt()
                .or_else(|| self.c0.sub(norm_root).mul(half).sqrt())?;
            let y = self.c1.mul(x.add(x).invert()?);
            Self { c0: x, c1: y }
        };
        // Check the complete extension-field equation before exposing any root.
        if root.square() != self {
            return None;
        }
        let negative = root.negate();
        Some(
            if root.to_wire().to_be_bytes() <= negative.to_wire().to_be_bytes() {
                root
            } else {
                negative
            },
        )
    }
}
