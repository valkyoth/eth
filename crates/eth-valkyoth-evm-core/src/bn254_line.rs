use crate::{
    bn254::G1Point,
    bn254_g2::{Fp2, G2Point},
    bn254_tower::{Fp6, Fp12},
};

#[cfg(test)]
use crate::bn254_field::Fp;

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct G2LineCoefficients {
    pub(crate) x: Fp2,
    pub(crate) y: Fp2,
    pub(crate) c: Fp2,
}

#[cfg(test)]
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

    pub(crate) fn evaluate_g1_fp6(self, point: G1Point) -> Fp6 {
        if point.infinity {
            return Fp6::ONE;
        }
        Fp6 {
            c0: self.c,
            c1: self.x.mul(fp2_from_fp(point.x)),
            c2: self.y.mul(fp2_from_fp(point.y)),
        }
    }

    #[cfg(test)]
    pub(crate) fn evaluate_g1(self, point: G1Point) -> Fp12 {
        Fp12 {
            c0: self.evaluate_g1_fp6(point),
            c1: Fp6::ZERO,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct G2LineStep {
    pub(crate) next: G2Point,
    pub(crate) line: G2LineCoefficients,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProjectiveG2LineCoefficients {
    a: Fp2,
    b: Fp2,
    c: Fp2,
}

impl ProjectiveG2LineCoefficients {
    const ZERO: Self = Self {
        a: Fp2::ZERO,
        b: Fp2::ZERO,
        c: Fp2::ZERO,
    };

    pub(crate) fn multiply_accumulator(self, acc: Fp12) -> Fp12 {
        if self == Self::ZERO {
            return acc;
        }
        let a2 = Fp6 {
            c0: self.b,
            c1: self.a,
            c2: Fp2::ZERO,
        }
        .mul(acc.c1);
        let t3 = acc.c0.mul_by_fp2(self.c);
        let t2 = Fp6 {
            c0: self.b.add(self.c),
            c1: self.a,
            c2: Fp2::ZERO,
        };
        let x = acc.c1.add(acc.c0).mul(t2).sub(a2).sub(t3);
        let y = t3.add(a2.mul_by_v());
        Fp12 { c0: y, c1: x }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProjectiveG2LineState {
    x: Fp2,
    y: Fp2,
    z: Fp2,
    t: Fp2,
}

impl ProjectiveG2LineState {
    pub(crate) fn from_affine(point: G2Point) -> Self {
        if point.infinity {
            return Self {
                x: Fp2::ZERO,
                y: Fp2::ONE,
                z: Fp2::ZERO,
                t: Fp2::ZERO,
            };
        }
        Self {
            x: point.x,
            y: point.y,
            z: Fp2::ONE,
            t: Fp2::ONE,
        }
    }

    pub(crate) fn doubling_line(self, g1: G1Point) -> ProjectiveG2LineStep {
        if self.z.is_zero() || self.y.is_zero() {
            return ProjectiveG2LineStep {
                next: self,
                line: ProjectiveG2LineCoefficients::ZERO,
            };
        }
        let a = self.x.square();
        let b = self.y.square();
        let c = b.square();
        let d = self.x.add(b).square().sub(a).sub(c).double();
        let e = a.double().add(a);
        let g = e.square();
        let x3 = g.sub(d.double());
        let z3 = self.y.add(self.z).square().sub(b).sub(self.t);
        let y3 = e.mul(d.sub(x3)).sub(c.double().double().double());
        let t3 = z3.square();
        let b_line = e.mul(self.t).double().neg().mul_by_fp(g1.x);
        let a_line = self
            .x
            .add(e)
            .square()
            .sub(a)
            .sub(g)
            .sub(b.double().double());
        let c_line = z3.mul(self.t).double().mul_by_fp(g1.y);
        ProjectiveG2LineStep {
            next: Self {
                x: x3,
                y: y3,
                z: z3,
                t: t3,
            },
            line: ProjectiveG2LineCoefficients {
                a: a_line,
                b: b_line,
                c: c_line,
            },
        }
    }

    pub(crate) fn addition_line(
        self,
        point: G2Point,
        point_y_squared: Fp2,
        g1: G1Point,
    ) -> ProjectiveG2LineStep {
        if self.z.is_zero() || point.infinity {
            return ProjectiveG2LineStep {
                next: self,
                line: ProjectiveG2LineCoefficients::ZERO,
            };
        }
        let b = point.x.mul(self.t);
        let d = point
            .y
            .add(self.z)
            .square()
            .sub(point_y_squared)
            .sub(self.t)
            .mul(self.t);
        let h = b.sub(self.x);
        let i = h.square();
        let e = i.double().double();
        let j = h.mul(e);
        let l1 = d.sub(self.y).sub(self.y);
        let v = self.x.mul(e);
        let x3 = l1.square().sub(j).sub(v.double());
        let z3 = self.z.add(h).square().sub(self.t).sub(i);
        let y3 = l1.mul(v.sub(x3)).sub(self.y.mul(j).double());
        let t3 = z3.square();
        let a_line = l1
            .mul(point.x)
            .double()
            .sub(point.y.add(z3).square().sub(point_y_squared).sub(t3));
        let c_line = z3.mul_by_fp(g1.y).double();
        let b_line = l1.neg().mul_by_fp(g1.x).double();
        ProjectiveG2LineStep {
            next: Self {
                x: x3,
                y: y3,
                z: z3,
                t: t3,
            },
            line: ProjectiveG2LineCoefficients {
                a: a_line,
                b: b_line,
                c: c_line,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProjectiveG2LineStep {
    pub(crate) next: ProjectiveG2LineState,
    pub(crate) line: ProjectiveG2LineCoefficients,
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn slope_line(slope: Fp2, point: G2Point) -> G2LineCoefficients {
    G2LineCoefficients {
        x: slope,
        y: Fp2::ONE.neg(),
        c: point.y.sub(slope.mul(point.x)),
    }
}

#[cfg(test)]
fn vertical_line(x: Fp2) -> G2LineCoefficients {
    G2LineCoefficients {
        x: Fp2::ONE,
        y: Fp2::ZERO,
        c: x.neg(),
    }
}

#[cfg(test)]
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
