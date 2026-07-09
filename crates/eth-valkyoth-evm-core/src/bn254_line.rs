use crate::{
    bn254::G1Point,
    bn254_field::Fp,
    bn254_g2::{Fp2, G2Point},
    bn254_tower::{Fp6, Fp12},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct G2LineCoefficients {
    pub(crate) x: Fp2,
    pub(crate) y: Fp2,
    pub(crate) c: Fp2,
}

impl G2LineCoefficients {
    pub(crate) const ZERO: Self = Self {
        x: Fp2::ZERO,
        y: Fp2::ZERO,
        c: Fp2::ZERO,
    };

    #[cfg(test)]
    pub(crate) fn evaluate_g2(self, point: G2Point) -> Fp2 {
        if point.infinity {
            return Fp2::ZERO;
        }
        self.x.mul(point.x).add(self.y.mul(point.y)).add(self.c)
    }

    pub(crate) fn evaluate_g1(self, point: G1Point) -> Fp12 {
        if point.infinity {
            return Fp12::ONE;
        }
        Fp12 {
            c0: Fp6 {
                c0: self.c,
                c1: self.x.mul(fp2_from_fp(point.x)),
                c2: self.y.mul(fp2_from_fp(point.y)),
            },
            c1: Fp6::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct G2LineStep {
    pub(crate) next: G2Point,
    pub(crate) line: G2LineCoefficients,
}

#[cfg(test)]
pub(crate) fn evaluate_line_foundation_at_g1(g1: G1Point, g2: G2Point) -> Fp12 {
    let doubled = g2_doubling_line(g2);
    let added = g2_addition_line(g2, doubled.next);
    let relation = doubled.line.evaluate_g2(g2).add(added.line.evaluate_g2(g2));
    doubled
        .line
        .evaluate_g1(g1)
        .add(added.line.evaluate_g1(g1))
        .add(fp12_from_fp2(relation))
}

pub(crate) fn g2_doubling_line(point: G2Point) -> G2LineStep {
    if point.infinity {
        return G2LineStep {
            next: point,
            line: G2LineCoefficients::ZERO,
        };
    }
    let Some(denominator) = point.y.double().invert() else {
        return G2LineStep {
            next: G2Point::INFINITY,
            line: vertical_line(point.x),
        };
    };
    let slope = point.x.square().add(point.x.square()).add(point.x.square());
    let slope = slope.mul(denominator);
    let x3 = slope.square().sub(point.x.double());
    let y3 = slope.mul(point.x.sub(x3)).sub(point.y);
    G2LineStep {
        next: G2Point {
            x: x3,
            y: y3,
            infinity: false,
        },
        line: slope_line(slope, point),
    }
}

pub(crate) fn g2_addition_line(left: G2Point, right: G2Point) -> G2LineStep {
    if left.infinity {
        return G2LineStep {
            next: right,
            line: G2LineCoefficients::ZERO,
        };
    }
    if right.infinity {
        return G2LineStep {
            next: left,
            line: G2LineCoefficients::ZERO,
        };
    }
    let delta_x = right.x.sub(left.x);
    if delta_x.is_zero() {
        if right.y == left.y {
            return g2_doubling_line(left);
        }
        return G2LineStep {
            next: G2Point::INFINITY,
            line: vertical_line(left.x),
        };
    }
    let Some(inverse_delta_x) = delta_x.invert() else {
        return G2LineStep {
            next: G2Point::INFINITY,
            line: vertical_line(left.x),
        };
    };
    let slope = right.y.sub(left.y).mul(inverse_delta_x);
    let x3 = slope.square().sub(left.x).sub(right.x);
    let y3 = slope.mul(left.x.sub(x3)).sub(left.y);
    G2LineStep {
        next: G2Point {
            x: x3,
            y: y3,
            infinity: false,
        },
        line: slope_line(slope, left),
    }
}

fn slope_line(slope: Fp2, point: G2Point) -> G2LineCoefficients {
    G2LineCoefficients {
        x: slope,
        y: Fp2::ONE.neg(),
        c: point.y.sub(slope.mul(point.x)),
    }
}

fn vertical_line(x: Fp2) -> G2LineCoefficients {
    G2LineCoefficients {
        x: Fp2::ONE,
        y: Fp2::ZERO,
        c: x.neg(),
    }
}

fn fp2_from_fp(value: Fp) -> Fp2 {
    Fp2 {
        c0: value,
        c1: Fp::ZERO,
    }
}

#[cfg(test)]
fn fp12_from_fp2(value: Fp2) -> Fp12 {
    Fp12 {
        c0: Fp6 {
            c0: value,
            c1: Fp2::ZERO,
            c2: Fp2::ZERO,
        },
        c1: Fp6::ZERO,
    }
}
