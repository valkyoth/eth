use super::{EvmBls12381G2Affine as Affine, EvmBls12381G2Projective as Projective, Fp2};
use crate::{EvmBls12381Fp2 as WireFp2, EvmBls12381G2Point, EvmCoreError};

fn point() -> Result<Affine, EvmCoreError> {
    let x = Fp2::from_u64(2);
    let y = x
        .square()
        .mul(x)
        .add(Fp2::from_coefficients(4, 4))
        .sqrt()
        .ok_or(EvmCoreError::PrecompilePointNotOnCurve)?;
    Affine::try_from_coordinates(x.to_wire(), y.to_wire())
}

#[test]
fn identities_and_geometric_equality_survive_nonreal_rescaling() -> Result<(), EvmCoreError> {
    let p = point()?.to_projective();
    let zero = Projective::infinity();
    assert_eq!(p.add_point(zero), p);
    assert_eq!(zero.add_point(p), p);
    assert_eq!(zero.double(), zero);
    assert_eq!(zero.negate(), zero);
    assert_eq!(p.add_point(p), p.double());
    assert!(p.add_point(p.negate()).is_infinity());
    assert_eq!(zero.to_affine().to_be_bytes(), [0; 256]);
    assert_eq!(Affine::try_from_be_bytes(&[0; 256])?, zero.to_affine());
    for (a, b) in [(1, 0), (0, 1), (2, 3), (7, u64::MAX)] {
        let z = Fp2::from_coefficients(a, b);
        let q = Projective {
            x: p.x.mul(z.square()),
            y: p.y.mul(z.square()).mul(z),
            z,
        };
        assert_eq!(p, q);
        assert_eq!(p.to_affine(), q.to_affine());
        assert_eq!(p.add_point(q), p.double());
        assert_eq!(q.double(), p.double());
        assert!(p.add_point(q.negate()).is_infinity());
        assert_ne!(q, q.negate());
        assert_ne!(q, zero);
    }
    assert_eq!(core::mem::size_of::<Projective>(), 288);
    Ok(())
}

#[test]
fn wire_domain_cannot_bypass_curve_or_field_validation() -> Result<(), EvmCoreError> {
    let zero = WireFp2::from_u64(0);
    assert_eq!(
        Affine::try_from_coordinates(zero, zero),
        Err(EvmCoreError::PrecompilePointNotOnCurve)
    );
    let mut bytes = [0; 256];
    bytes[255] = 1;
    let wire = EvmBls12381G2Point::try_from_be_bytes(&bytes)?;
    assert_eq!(
        Affine::try_from_wire(wire),
        Err(EvmCoreError::PrecompilePointNotOnCurve)
    );
    for offset in [0usize, 64, 128, 192] {
        let mut bytes = point()?.to_be_bytes();
        *bytes
            .get_mut(offset)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)? = 1;
        assert_eq!(
            Affine::try_from_be_bytes(&bytes),
            Err(EvmCoreError::PrecompileFieldElementOutOfRange)
        );
        let mut bytes = [0; 256];
        bytes
            .get_mut(offset + 16..offset + 64)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
            .copy_from_slice(&crate::bls12_wire::FP_MODULUS);
        assert_eq!(
            Affine::try_from_be_bytes(&bytes),
            Err(EvmCoreError::PrecompileFieldElementOutOfRange)
        );
    }
    for len in [0, 128, 255, 257, 512] {
        assert_eq!(
            Affine::try_from_be_bytes(
                [0; 512]
                    .get(..len)
                    .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
            ),
            Err(EvmCoreError::PrecompileInvalidInputLength)
        );
    }
    Ok(())
}
