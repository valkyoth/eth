use crate::{
    bn254_g2::Fp2,
    bn254_tower::{Fp6, Fp12},
};

#[test]
fn fp6_extension_defines_v_cubed_as_fp2_nonresidue() {
    let v = fp6(Fp2::ZERO, Fp2::ONE, Fp2::ZERO);
    let v2 = v.square();
    let v3 = v2.mul(v);
    let expected = fp6(Fp2::NINE_PLUS_I, Fp2::ZERO, Fp2::ZERO);
    assert_eq!(v3, expected);
}

#[test]
fn fp12_extension_defines_w_squared_as_v() {
    let v = fp6(Fp2::ZERO, Fp2::ONE, Fp2::ZERO);
    let w = fp12(Fp6::ZERO, Fp6::ONE);
    assert_eq!(w.square(), fp12(v, Fp6::ZERO));
}

#[test]
fn fp6_and_fp12_identity_and_zero_are_stable() {
    let x = sample_fp6();
    assert_eq!(x.mul(Fp6::ONE), x);
    assert_eq!(Fp6::ONE.mul(x), x);
    assert_eq!(x.mul(Fp6::ZERO), Fp6::ZERO);

    let v = fp6(Fp2::ZERO, Fp2::ONE, Fp2::ZERO);
    let y = fp12(x, v.add(Fp6::ONE));
    assert_eq!(y.mul(Fp12::ONE), y);
    assert_eq!(Fp12::ONE.mul(y), y);
    assert_eq!(y.mul(Fp12::ZERO), Fp12::ZERO);
}

#[test]
fn fp6_and_fp12_square_match_self_multiplication() {
    let x = sample_fp6();
    assert_eq!(x.square(), x.mul(x));

    let v = fp6(Fp2::ZERO, Fp2::ONE, Fp2::ZERO);
    let y = fp12(x, v.add(Fp6::ONE));
    assert_eq!(y.square(), y.mul(y));
}

#[test]
fn fp12_multiplication_distributes_over_addition() {
    let v = fp6(Fp2::ZERO, Fp2::ONE, Fp2::ZERO);
    let x = fp12(sample_fp6(), v);
    let y = fp12(Fp6::ONE.add(v), sample_fp6());
    let z = fp12(v.square(), Fp6::ONE);
    assert_eq!(x.mul(y.add(z)), x.mul(y).add(x.mul(z)));
}

#[test]
fn fp6_and_fp12_inversion_round_trip_to_one() {
    let x = sample_fp6();
    assert_eq!(x.invert().map(|x_inv| x.mul(x_inv)), Some(Fp6::ONE));

    let v = fp6(Fp2::ZERO, Fp2::ONE, Fp2::ZERO);
    let y = fp12(x, v.add(Fp6::ONE));
    assert_eq!(y.invert().map(|y_inv| y.mul(y_inv)), Some(Fp12::ONE));
}

#[test]
fn fp12_frobenius_maps_cycle_back_to_identity() {
    let v = fp6(Fp2::ZERO, Fp2::ONE, Fp2::ZERO);
    let x = fp12(sample_fp6(), v.add(Fp6::ONE));

    let mut cycled = x;
    for _ in 0..12 {
        cycled = cycled.frobenius();
    }
    assert_eq!(cycled, x);
    assert_eq!(x.frobenius_p2(), x.frobenius().frobenius());
    assert_eq!(
        x.frobenius_p6(),
        x.frobenius_p2().frobenius_p2().frobenius_p2()
    );
}

fn sample_fp6() -> Fp6 {
    let two = Fp2::ONE.double();
    let three = Fp2 {
        c0: crate::bn254_field::Fp::THREE,
        c1: crate::bn254_field::Fp::ZERO,
    };
    let mixed = Fp2::NINE_PLUS_I.add(Fp2::ONE);
    fp6(two, three, mixed)
}

fn fp12(c0: Fp6, c1: Fp6) -> Fp12 {
    Fp12 { c0, c1 }
}

fn fp6(c0: Fp2, c1: Fp2, c2: Fp2) -> Fp6 {
    Fp6 { c0, c1, c2 }
}
