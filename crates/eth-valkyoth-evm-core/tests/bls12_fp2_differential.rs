//! Extension-field differential, canonicality, and deterministic-root regressions.
use eth_valkyoth_evm_core::{EvmBls12381Fp as Fp, EvmBls12381Fp2 as Fp2, EvmCoreError};
use num_bigint::BigUint;
#[path = "support/bls12_fp2_oracle.rs"]
mod oracle;

fn pair(a: u64, b: u64) -> Fp2 {
    Fp2::from_coefficients(Fp::from_u64(a), Fp::from_u64(b))
}

#[test]
fn eip2537_generator_coordinates_fix_extension_conventions() -> Result<(), EvmCoreError> {
    // Official public H2 coordinates, EIPs revision 582684e2d7d372c09f45777be8ea603e485e9e9d.
    // A field-equation vector only: no G2 point validation or subgroup API is admitted.
    let parse = |hex: &[u8]| {
        BigUint::parse_bytes(hex, 16).ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)
    };
    let x = oracle::encode(&(
        parse(b"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8")?,
        parse(b"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e")?,
    ))?;
    let y = oracle::encode(&(
        parse(b"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801")?,
        parse(b"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be")?,
    ))?;
    let rhs = x.square().mul_mod(x).add_mod(pair(4, 4));
    assert_eq!(y.square(), rhs);
    let root = rhs
        .sqrt()
        .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
    assert!(root == y || root == y.negate());
    oracle::check(x, y, &oracle::modulus()?);
    Ok(())
}

#[test]
fn extension_conventions_and_small_exact_vectors() -> Result<(), EvmCoreError> {
    let p = oracle::modulus()?;
    let zero = pair(0, 0);
    let one = pair(1, 0);
    let v = pair(0, 1);
    assert_eq!(v.square(), one.negate());
    assert_eq!(v.invert(), Some(v.negate()));
    assert_eq!(zero.invert(), None);
    assert_eq!(zero.sqrt(), Some(zero));
    assert_eq!(one.negate().sqrt(), Some(v));
    assert_eq!(pair(1, 1).sqrt(), None);
    assert_eq!(
        pair(3, 4).square(),
        oracle::encode(&(&p - 7u8, BigUint::from(24u8)))?
    );
    assert_eq!(pair(3, 4).square().sqrt(), Some(pair(3, 4)));
    assert_eq!(
        pair(3, 4).mul_mod(pair(5, 6)),
        oracle::encode(&(&p - 9u8, BigUint::from(38u8)))?
    );
    // Both signs of the norm root, real and imaginary cases, and zero.
    for a in 0..12 {
        for b in 0..12 {
            let value = pair(a, b);
            oracle::check(value, pair(b, a), &p);
            let root = value
                .square()
                .sqrt()
                .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
            assert!(root == value || root == value.negate());
        }
    }
    Ok(())
}

#[test]
fn full_width_limb_boundaries_inverses_and_frobenius() -> Result<(), EvmCoreError> {
    let p = oracle::modulus()?;
    let mut values = vec![BigUint::from(0u8), BigUint::from(1u8), &p - 1u8, &p - 2u8];
    for bit in [63usize, 64, 127, 128, 191, 192, 255, 256, 319, 320, 380] {
        let value = BigUint::from(1u8) << bit;
        values.extend([&value - 1u8, value.clone(), &value + 1u8]);
    }
    for (a, b) in values.iter().zip(values.iter().rev()) {
        let raw = (a.clone(), b.clone());
        let value = oracle::encode(&raw)?;
        oracle::check(value, value.conjugate(), &p);
        assert_eq!(oracle::pow(&raw, &p, &p), oracle::decode(value.conjugate()));
        if let Some(inverse) = value.invert() {
            assert_eq!(
                oracle::decode(inverse),
                oracle::pow(&raw, &(&p * &p - 2u8), &p)
            );
        }
        let root = value
            .square()
            .sqrt()
            .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
        assert!(root == value || root == value.negate());
    }
    Ok(())
}

#[test]
fn deterministic_full_width_residues_and_chained_operations() -> Result<(), EvmCoreError> {
    let p = oracle::modulus()?;
    let mut state = 91u64;
    let mut accumulator = pair(1, 1);
    let mut expected = oracle::decode(accumulator);
    for _ in 0..128 {
        let mut raw = [0; 96];
        for chunk in raw.chunks_exact_mut(8) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            chunk.copy_from_slice(&state.to_be_bytes());
        }
        let (a, b) = raw.split_at(48);
        let raw = (
            BigUint::from_bytes_be(a) % &p,
            BigUint::from_bytes_be(b) % &p,
        );
        let value = oracle::encode(&raw)?;
        oracle::check(value, accumulator, &p);
        accumulator = accumulator.mul_mod(value);
        expected = oracle::mul(&expected, &raw, &p);
        assert_eq!(oracle::decode(accumulator), expected);
        assert_eq!(
            Fp2::try_from_be_bytes(&accumulator.to_be_bytes())?,
            accumulator
        );
    }
    Ok(())
}

#[test]
fn canonical_encoding_preserves_order_and_rejects_each_bad_coefficient() -> Result<(), EvmCoreError>
{
    let p = oracle::modulus()?;
    let value = pair(3, 4);
    assert_eq!(value.c0(), Fp::from_u64(3));
    assert_eq!(value.c1(), Fp::from_u64(4));
    assert_eq!(Fp2::try_from_be_bytes(&value.to_be_bytes())?, value);
    assert_ne!(value.to_be_bytes(), pair(4, 3).to_be_bytes());
    for offset in [0usize, 64] {
        let mut bytes = value.to_be_bytes();
        *bytes
            .get_mut(offset)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)? = 1;
        assert_eq!(
            Fp2::try_from_be_bytes(&bytes),
            Err(EvmCoreError::PrecompileFieldElementOutOfRange)
        );
        let mut bytes = value.to_be_bytes();
        bytes
            .get_mut(offset..offset + 64)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
            .fill(0);
        bytes
            .get_mut(offset + 16..offset + 64)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
            .copy_from_slice(&p.to_bytes_be());
        assert_eq!(
            Fp2::try_from_be_bytes(&bytes),
            Err(EvmCoreError::PrecompileFieldElementOutOfRange)
        );
    }
    for len in [0, 64, 127, 129, 256] {
        assert_eq!(
            Fp2::try_from_be_bytes(&vec![0; len]),
            Err(EvmCoreError::PrecompileInvalidInputLength)
        );
    }
    assert_eq!(core::mem::size_of::<Fp2>(), 96);
    Ok(())
}
