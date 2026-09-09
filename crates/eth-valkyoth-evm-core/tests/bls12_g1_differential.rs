//! Independent affine oracle plus exceptional and full-width group-law checks.

#[path = "support/bls12_g1_oracle.rs"]
mod oracle;

use eth_valkyoth_evm_core::{EvmBls12381G1Affine as Affine, EvmCoreError};
use num_bigint::BigUint;

fn actual(p: &oracle::Point) -> Result<Affine, EvmCoreError> {
    let encoded = oracle::encode(p)?;
    assert_eq!(oracle::decode(&encoded)?, *p);
    Affine::try_from_be_bytes(&encoded)
}

#[test]
fn generator_multiples_match_independent_affine_formula() -> Result<(), EvmCoreError> {
    let generator = oracle::generator()?;
    let g = actual(&generator)?;
    let mut expected = None;
    let mut sum = Affine::infinity().to_projective();
    for _ in 0..40 {
        assert_eq!(sum.to_affine().to_be_bytes(), oracle::encode(&expected)?);
        assert_eq!(
            sum.double().to_affine().to_be_bytes(),
            oracle::encode(&oracle::add(&expected, &expected)?)?
        );
        expected = oracle::add(&expected, &generator)?;
        sum = sum.add_point(g.to_projective());
    }
    Ok(())
}

#[test]
fn full_curve_samples_and_nonunit_projective_additions_match_oracle() -> Result<(), EvmCoreError> {
    let mut points = vec![
        None,
        oracle::generator()?,
        oracle::from_x(BigUint::from(0u8))?,
    ];
    let mut state = 19u64;
    for _ in 0..40 {
        let mut bytes = [0; 48];
        for chunk in bytes.chunks_exact_mut(8) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            chunk.copy_from_slice(&state.to_be_bytes());
        }
        if let Some(point) = oracle::from_x(BigUint::from_bytes_be(&bytes))? {
            points.push(Some(point));
        }
    }
    assert!(points.len() >= 12);
    for pair in points.windows(2) {
        let [left, right] = pair else {
            continue;
        };
        let a = actual(left)?;
        let b = actual(right)?;
        let expected = oracle::add(left, right)?;
        assert_eq!(a.add_point(b).to_be_bytes(), oracle::encode(&expected)?);
        assert_eq!(b.add_point(a), a.add_point(b));
        let aa = oracle::add(left, left)?;
        let bb = oracle::add(right, right)?;
        let ap = a.to_projective().double();
        let bp = b.to_projective().double();
        assert_eq!(
            ap.add_point(bp).to_affine().to_be_bytes(),
            oracle::encode(&oracle::add(&aa, &bb)?)?
        );
        assert_eq!(ap, actual(&aa)?.to_projective());
        assert_eq!(ap.add_point(actual(&aa)?.to_projective()), ap.double());
        assert!(
            ap.add_point(actual(&aa)?.negate().to_projective())
                .is_infinity()
        );
        assert_eq!(ap.add_point(ap), ap.double());
        assert!(ap.add_point(ap.negate()).is_infinity());
        let c = actual(&oracle::generator()?)?.to_projective();
        assert_eq!(ap.add_point(bp).add_point(c), ap.add_point(bp.add_point(c)));
        assert_eq!(
            Affine::try_from_be_bytes(&ap.add_point(bp).to_affine().to_be_bytes())?,
            ap.add_point(bp).to_affine()
        );
    }
    Ok(())
}

#[test]
fn invalid_curve_samples_are_rejected_by_independent_equation() -> Result<(), EvmCoreError> {
    let p = oracle::modulus()?;
    for x in 0..12u64 {
        for y in 0..12u64 {
            let xb = BigUint::from(x);
            let yb = BigUint::from(y);
            let valid = (x == 0 && y == 0) || (&yb * &yb) % &p == (&xb * &xb * &xb + 4u8) % &p;
            assert_eq!(
                Affine::try_from_be_bytes(&oracle::encode(&Some((xb, yb)))?).is_ok(),
                valid
            );
        }
    }
    Ok(())
}
