extern crate std;

use std::vec::Vec;

use crate::{
    EVM_BLS12381_FP_BYTES, EVM_BLS12381_FP2_BYTES, EVM_BLS12381_FR_BYTES,
    EVM_BLS12381_G1_MSM_ITEM_BYTES, EVM_BLS12381_G1_POINT_BYTES, EVM_BLS12381_G2_MSM_ITEM_BYTES,
    EVM_BLS12381_G2_POINT_BYTES, EVM_BLS12381_PAIRING_ITEM_BYTES, EVM_BLS12381_SCALAR_BYTES,
    EvmBls12381Fp, EvmBls12381Fp2, EvmBls12381Fr, EvmBls12381G1Point, EvmBls12381G2Point,
    EvmBls12381Scalar, EvmCoreError, parse_bls12381_g1_add, parse_bls12381_g1_msm,
    parse_bls12381_g2_add, parse_bls12381_g2_msm, parse_bls12381_map_fp_to_g1,
    parse_bls12381_map_fp2_to_g2, parse_bls12381_pairing,
};

const FP_MODULUS: [u8; 48] = [
    0x1a, 0x01, 0x11, 0xea, 0x39, 0x7f, 0xe6, 0x9a, 0x4b, 0x1b, 0xa7, 0xb6, 0x43, 0x4b, 0xac, 0xd7,
    0x64, 0x77, 0x4b, 0x84, 0xf3, 0x85, 0x12, 0xbf, 0x67, 0x30, 0xd2, 0xa0, 0xf6, 0xb0, 0xf6, 0x24,
    0x1e, 0xab, 0xff, 0xfe, 0xb1, 0x53, 0xff, 0xff, 0xb9, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xaa, 0xab,
];

const FR_MODULUS: [u8; EVM_BLS12381_FR_BYTES] = [
    0x73, 0xed, 0xa7, 0x53, 0x29, 0x9d, 0x7d, 0x48, 0x33, 0x39, 0xd8, 0x08, 0x09, 0xa1, 0xd8, 0x05,
    0x53, 0xbd, 0xa4, 0x02, 0xff, 0xfe, 0x5b, 0xfe, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x01,
];

#[test]
fn fp_accepts_zero_and_modulus_minus_one() -> Result<(), EvmCoreError> {
    let zero = [0u8; EVM_BLS12381_FP_BYTES];
    let decoded = EvmBls12381Fp::try_from_be_bytes(&zero)?;
    assert!(decoded.is_zero());
    assert_eq!(decoded.to_be_bytes(), zero);

    let max = padded_fp(decrement(FP_MODULUS));
    let decoded = EvmBls12381Fp::try_from_be_bytes(&max)?;
    assert!(!decoded.is_zero());
    assert_eq!(decoded.value_bytes(), &max[16..]);
    assert_eq!(decoded.to_be_bytes(), max);
    Ok(())
}

#[test]
fn fp_rejects_modulus_padding_and_wrong_lengths() {
    let modulus = padded_fp(FP_MODULUS);
    assert_eq!(
        EvmBls12381Fp::try_from_be_bytes(&modulus),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );

    let mut bad_padding = [0u8; EVM_BLS12381_FP_BYTES];
    bad_padding[15] = 1;
    assert_eq!(
        EvmBls12381Fp::try_from_be_bytes(&bad_padding),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
    assert_eq!(
        EvmBls12381Fp::try_from_be_bytes(&bad_padding[..63]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
}

#[test]
fn fr_and_wire_scalar_keep_distinct_ranges() -> Result<(), EvmCoreError> {
    let q_minus_one = decrement(FR_MODULUS);
    assert_eq!(
        EvmBls12381Fr::try_from_be_bytes(&q_minus_one)?.to_be_bytes(),
        q_minus_one
    );
    assert_eq!(
        EvmBls12381Fr::try_from_be_bytes(&FR_MODULUS),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );

    let q = EvmBls12381Scalar::try_from_be_bytes(&FR_MODULUS)?;
    assert_eq!(q.to_be_bytes(), FR_MODULUS);
    let max = [u8::MAX; EVM_BLS12381_SCALAR_BYTES];
    assert_eq!(
        EvmBls12381Scalar::try_from_be_bytes(&max)?.to_be_bytes(),
        max
    );
    Ok(())
}

#[test]
fn fp2_preserves_c0_then_c1_order() -> Result<(), EvmCoreError> {
    let mut encoded = [0u8; EVM_BLS12381_FP2_BYTES];
    encoded[63] = 1;
    encoded[127] = 2;
    let value = EvmBls12381Fp2::try_from_be_bytes(&encoded)?;

    assert_eq!(value.c0().to_be_bytes()[63], 1);
    assert_eq!(value.c1().to_be_bytes()[63], 2);
    assert_eq!(value.to_be_bytes(), encoded);
    Ok(())
}

#[test]
fn point_encodings_admit_only_the_all_zero_infinity_form() -> Result<(), EvmCoreError> {
    let g1_zero = [0u8; EVM_BLS12381_G1_POINT_BYTES];
    let g1 = EvmBls12381G1Point::try_from_be_bytes(&g1_zero)?;
    assert!(g1.is_infinity());
    assert_eq!(g1.coordinates(), None);
    assert_eq!(g1.to_be_bytes(), g1_zero);

    let mut g1_nonzero = g1_zero;
    g1_nonzero[127] = 1;
    let g1 = EvmBls12381G1Point::try_from_be_bytes(&g1_nonzero)?;
    assert!(!g1.is_infinity());
    assert!(g1.coordinates().is_some());
    assert_eq!(g1.to_be_bytes(), g1_nonzero);

    let g2_zero = [0u8; EVM_BLS12381_G2_POINT_BYTES];
    let g2 = EvmBls12381G2Point::try_from_be_bytes(&g2_zero)?;
    assert!(g2.is_infinity());
    assert_eq!(g2.coordinates(), None);
    assert_eq!(g2.to_be_bytes(), g2_zero);

    let mut alternate = g2_zero;
    alternate[0] = 1;
    assert_eq!(
        EvmBls12381G2Point::try_from_be_bytes(&alternate),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
    Ok(())
}

#[test]
fn fixed_frames_decode_every_coordinate() -> Result<(), EvmCoreError> {
    let mut g1_add = [0u8; EVM_BLS12381_G1_POINT_BYTES * 2];
    g1_add[127] = 1;
    let parsed = parse_bls12381_g1_add(&g1_add)?;
    assert!(!parsed.left.is_infinity());
    assert!(parsed.right.is_infinity());

    let mut g2_add = [0u8; EVM_BLS12381_G2_POINT_BYTES * 2];
    g2_add[255] = 1;
    let parsed = parse_bls12381_g2_add(&g2_add)?;
    assert!(!parsed.left.is_infinity());
    assert!(parsed.right.is_infinity());

    let mut fp = [0u8; EVM_BLS12381_FP_BYTES];
    fp[63] = 7;
    assert_eq!(parse_bls12381_map_fp_to_g1(&fp)?.to_be_bytes(), fp);

    let mut fp2 = [0u8; EVM_BLS12381_FP2_BYTES];
    fp2[63] = 7;
    fp2[127] = 9;
    assert_eq!(parse_bls12381_map_fp2_to_g2(&fp2)?.to_be_bytes(), fp2);
    Ok(())
}

#[test]
fn variable_frames_accept_full_width_scalars_and_iterate_exactly() -> Result<(), EvmCoreError> {
    let mut g1_msm = [0u8; EVM_BLS12381_G1_MSM_ITEM_BYTES * 2];
    g1_msm[127] = 1;
    g1_msm[128..160].fill(u8::MAX);
    let parsed = parse_bls12381_g1_msm(&g1_msm)?;
    assert_eq!(parsed.len(), 2);
    let mut items = parsed.items();
    assert_eq!(items.len(), 2);
    let first = items
        .next()
        .transpose()?
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    assert!(!first.point.is_infinity());
    assert_eq!(first.scalar.to_be_bytes(), [u8::MAX; 32]);
    assert!(items.next().transpose()?.is_some());
    assert_eq!(items.next(), None);
    assert_eq!(items.next(), None);

    let mut g2_msm = [0u8; EVM_BLS12381_G2_MSM_ITEM_BYTES];
    g2_msm[256..].copy_from_slice(&FR_MODULUS);
    let item = parse_bls12381_g2_msm(&g2_msm)?
        .items()
        .next()
        .transpose()?
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    assert!(item.point.is_infinity());
    assert_eq!(item.scalar.to_be_bytes(), FR_MODULUS);

    let pairing = [0u8; EVM_BLS12381_PAIRING_ITEM_BYTES * 2];
    let parsed = parse_bls12381_pairing(&pairing)?;
    assert_eq!(parsed.len(), 2);
    assert!(parsed.items().all(|item| {
        item.map(|pair| pair.g1.is_infinity() && pair.g2.is_infinity())
            .unwrap_or(false)
    }));
    Ok(())
}

#[test]
fn variable_frames_reject_empty_partial_and_late_malformed_items() {
    assert_eq!(
        parse_bls12381_g1_msm(&[]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        parse_bls12381_g2_msm(&[0u8; 287]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        parse_bls12381_pairing(&[0u8; 383]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );

    let mut late_bad_g1 = Vec::from([0u8; EVM_BLS12381_G1_MSM_ITEM_BYTES * 2]);
    if let Some(byte) = late_bad_g1.get_mut(EVM_BLS12381_G1_MSM_ITEM_BYTES) {
        *byte = 1;
    }
    assert_eq!(
        parse_bls12381_g1_msm(&late_bad_g1),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );

    let mut late_bad_g2 = Vec::from([0u8; EVM_BLS12381_G2_MSM_ITEM_BYTES * 2]);
    if let Some(byte) = late_bad_g2.get_mut(EVM_BLS12381_G2_MSM_ITEM_BYTES) {
        *byte = 1;
    }
    assert_eq!(
        parse_bls12381_g2_msm(&late_bad_g2),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );

    let mut late_bad_pairing = Vec::from([0u8; EVM_BLS12381_PAIRING_ITEM_BYTES * 2]);
    if let Some(byte) = late_bad_pairing.get_mut(EVM_BLS12381_PAIRING_ITEM_BYTES) {
        *byte = 1;
    }
    assert_eq!(
        parse_bls12381_pairing(&late_bad_pairing),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
}

#[test]
fn every_wire_domain_rejects_adjacent_lengths() {
    assert_eq!(
        EvmBls12381Fr::try_from_be_bytes(&[0u8; 31]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        EvmBls12381Scalar::try_from_be_bytes(&[0u8; 33]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        EvmBls12381Fp2::try_from_be_bytes(&[0u8; 127]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        EvmBls12381G1Point::try_from_be_bytes(&[0u8; 129]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    assert_eq!(
        EvmBls12381G2Point::try_from_be_bytes(&[0u8; 255]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
}

fn padded_fp(value: [u8; 48]) -> [u8; EVM_BLS12381_FP_BYTES] {
    let mut output = [0u8; EVM_BLS12381_FP_BYTES];
    output[16..].copy_from_slice(&value);
    output
}

fn decrement<const N: usize>(mut value: [u8; N]) -> [u8; N] {
    for byte in value.iter_mut().rev() {
        if *byte != 0 {
            *byte = byte.saturating_sub(1);
            break;
        }
        *byte = u8::MAX;
    }
    value
}
