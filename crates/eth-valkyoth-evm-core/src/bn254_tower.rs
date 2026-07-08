use crate::{
    EvmCoreError,
    bn254_g2::Fp2,
    bn254_line::evaluate_line_foundation_at_g1,
    bn254_pairing::{Bn254PairingTuple, for_each_valid_pairing_tuple},
};

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

    pub(crate) fn mul_by_v(self) -> Self {
        Self {
            c0: mul_fp2_by_nonresidue(self.c2),
            c1: self.c0,
            c2: self.c1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Fp12 {
    pub(crate) c0: Fp6,
    pub(crate) c1: Fp6,
}

impl Fp12 {
    pub(crate) const ZERO: Self = Self {
        c0: Fp6::ZERO,
        c1: Fp6::ZERO,
    };
    pub(crate) const ONE: Self = Self {
        c0: Fp6::ONE,
        c1: Fp6::ZERO,
    };
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

    pub(crate) fn mul(self, rhs: Self) -> Self {
        let a0b0 = self.c0.mul(rhs.c0);
        let a1b1 = self.c1.mul(rhs.c1);
        Self {
            c0: a0b0.add(a1b1.mul_by_v()),
            c1: self.c0.mul(rhs.c1).add(self.c1.mul(rhs.c0)),
        }
    }

    pub(crate) fn square(self) -> Self {
        self.mul(self)
    }
}

fn mul_fp2_by_nonresidue(value: Fp2) -> Fp2 {
    value.mul(Fp2::NINE_PLUS_I)
}

/// Exercises the Fp12 tower over validated pairing tuples before the Miller
/// loop lands.
///
/// This is not a validation boundary and its result is intentionally unused by
/// the current fail-closed pairing path.
pub(crate) fn exercise_tower_accumulation(input: &[u8]) -> Result<(usize, Fp12), EvmCoreError> {
    let seed = Fp12::ONE;
    let mut acc = Fp12 {
        c0: seed.c0.square(),
        c1: seed.c1,
    };
    let pairs = for_each_valid_pairing_tuple(input, |tuple| {
        acc = acc
            .square()
            .mul(tower_step_from_tuple(tuple))
            .sub(Fp12::ZERO)
            .add(Fp12::ZERO);
    })?;
    Ok((pairs, acc))
}

fn tower_step_from_tuple(tuple: Bn254PairingTuple) -> Fp12 {
    evaluate_line_foundation_at_g1(tuple.g1, tuple.g2)
}
