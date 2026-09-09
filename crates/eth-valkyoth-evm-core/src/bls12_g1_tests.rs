use super::{EvmBls12381G1Affine as Affine, EvmBls12381G1Projective as Projective, Fp};
use crate::{EvmBls12381Fp as WireFp, EvmBls12381G1Point, EvmCoreError};

fn torsion() -> Result<Affine, EvmCoreError> {
    Affine::try_from_coordinates(WireFp::from_u64(0), WireFp::from_u64(2))
}

#[test]
fn identity_inverse_and_non_subgroup_point_are_supported() -> Result<(), EvmCoreError> {
    let p = torsion()?;
    let zero = Affine::infinity();
    assert!(!p.is_infinity());
    assert_eq!(p.double(), p.negate());
    assert!(p.double().add_point(p).is_infinity()); // Order three, not prime-subgroup membership.
    assert_eq!(p.add_point(zero), p);
    assert_eq!(zero.add_point(p), p);
    assert_eq!(p.add_point(p.negate()), zero);
    assert_eq!(zero.double(), zero);
    assert_eq!(zero.negate(), zero);
    assert_eq!(zero.to_be_bytes(), [0; 128]);
    assert_eq!(Affine::try_from_be_bytes(&[0; 128])?, zero);
    assert_eq!(Affine::try_from_be_bytes(&p.to_be_bytes())?, p);
    assert_eq!(Projective::infinity().negate().to_affine(), zero);
    assert_ne!(p.to_projective(), Projective::infinity());
    assert_eq!(core::mem::size_of::<Projective>(), 144);
    Ok(())
}

#[test]
fn geometric_equality_and_exceptional_cases_survive_rescaling() -> Result<(), EvmCoreError> {
    let p = torsion()?.to_projective();
    for scale in [1, 2, 3, 7, u64::MAX] {
        let z = Fp::from_u64(scale);
        let q = Projective {
            x: p.x.mul(z.square()),
            y: p.y.mul(z.square()).mul(z),
            z,
        };
        assert_eq!(p, q);
        assert_eq!(p.to_affine(), q.to_affine());
        assert_eq!(p.add_point(q), p.double());
        assert!(p.add_point(q.negate()).is_infinity());
        assert_eq!(q.double().to_affine(), p.double().to_affine());
        assert_ne!(q, q.negate());
    }
    Ok(())
}

#[test]
fn raw_wire_acceptance_does_not_imply_curve_validation() -> Result<(), EvmCoreError> {
    let mut bytes = [0; 128];
    if let Some(last) = bytes.last_mut() {
        *last = 1;
    }
    let wire = EvmBls12381G1Point::try_from_be_bytes(&bytes)?;
    assert_eq!(
        Affine::try_from_wire(wire),
        Err(EvmCoreError::PrecompilePointNotOnCurve)
    );
    assert_eq!(
        Affine::try_from_be_bytes(&bytes),
        Err(EvmCoreError::PrecompilePointNotOnCurve)
    );
    assert_eq!(
        Affine::try_from_coordinates(WireFp::from_u64(0), WireFp::from_u64(0)),
        Err(EvmCoreError::PrecompilePointNotOnCurve)
    );
    for len in [0, 64, 127, 129, 256] {
        let input = [0; 256];
        assert_eq!(
            Affine::try_from_be_bytes(
                input
                    .get(..len)
                    .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
            ),
            Err(EvmCoreError::PrecompileInvalidInputLength)
        );
    }
    for offset in [0, 64] {
        let mut bytes = torsion()?.to_be_bytes();
        if let Some(byte) = bytes.get_mut(offset) {
            *byte = 1;
        }
        assert_eq!(
            Affine::try_from_be_bytes(&bytes),
            Err(EvmCoreError::PrecompileFieldElementOutOfRange)
        );
        let mut bytes = [0; 128];
        let start = offset.saturating_add(16);
        let end = start.saturating_add(48);
        bytes
            .get_mut(start..end)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
            .copy_from_slice(&crate::bls12_wire::FP_MODULUS);
        assert_eq!(
            Affine::try_from_be_bytes(&bytes),
            Err(EvmCoreError::PrecompileFieldElementOutOfRange)
        );
    }
    Ok(())
}
