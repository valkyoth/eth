//! Independent polynomial/BigUint oracle. Test-only; no Montgomery arithmetic.

use eth_valkyoth_evm_core::{EvmBls12381Fp as Fp, EvmBls12381Fp2 as Fp2, EvmCoreError};
use num_bigint::BigUint;
use std::ops::{Add, Mul, Rem, Shr, Sub};

pub type Pair = (BigUint, BigUint);

pub fn modulus() -> Result<BigUint, EvmCoreError> {
    BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16)
        .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)
}

pub fn decode(value: Fp2) -> Pair {
    (
        BigUint::from_bytes_be(value.c0().value_bytes()),
        BigUint::from_bytes_be(value.c1().value_bytes()),
    )
}

pub fn encode(value: &Pair) -> Result<Fp2, EvmCoreError> {
    fn coefficient(value: &BigUint) -> Result<Fp, EvmCoreError> {
        let bytes = value.to_bytes_be();
        let mut output = [0; 64];
        let offset = output
            .len()
            .checked_sub(bytes.len())
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
        output
            .get_mut(offset..)
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
            .copy_from_slice(&bytes);
        Fp::try_from_be_bytes(&output)
    }
    Ok(Fp2::from_coefficients(
        coefficient(&value.0)?,
        coefficient(&value.1)?,
    ))
}

pub fn mul(a: &Pair, b: &Pair, p: &BigUint) -> Pair {
    (
        (&a.0).mul(&b.0).add(p.mul(p)).sub((&a.1).mul(&b.1)).rem(p),
        (&a.0).mul(&b.1).add((&a.1).mul(&b.0)).rem(p),
    )
}

pub fn pow(a: &Pair, exponent: &BigUint, p: &BigUint) -> Pair {
    let mut result = (BigUint::from(1u8), BigUint::from(0u8));
    for byte in exponent.to_bytes_be() {
        for bit in (0..8).rev() {
            result = mul(&result, &result, p);
            if byte & (1 << bit) != 0 {
                result = mul(&result, a, p);
            }
        }
    }
    result
}

pub fn is_square(a: &Pair, p: &BigUint) -> bool {
    // Euler's criterion in Fp on the norm, not the production root formula.
    let norm = (&a.0).mul(&a.0).add((&a.1).mul(&a.1)).rem(p);
    norm == BigUint::from(0u8) || norm.modpow(&p.sub(1u8).shr(1usize), p) == BigUint::from(1u8)
}

pub fn check(a: Fp2, b: Fp2, p: &BigUint) {
    let aa = decode(a);
    let bb = decode(b);
    assert_eq!(
        decode(a.add_mod(b)),
        ((&aa.0).add(&bb.0).rem(p), (&aa.1).add(&bb.1).rem(p))
    );
    assert_eq!(
        decode(a.sub_mod(b)),
        (
            (&aa.0).add(p).sub(&bb.0).rem(p),
            (&aa.1).add(p).sub(&bb.1).rem(p)
        )
    );
    assert_eq!(decode(a.mul_mod(b)), mul(&aa, &bb, p));
    assert_eq!(decode(a.square()), mul(&aa, &aa, p));
    assert_eq!(
        decode(a.negate()),
        (p.sub(&aa.0).rem(p), p.sub(&aa.1).rem(p))
    );
    assert_eq!(decode(a.conjugate()), (aa.0.clone(), p.sub(&aa.1).rem(p)));
    let inverse = a.invert();
    assert_eq!(inverse.is_none(), a.is_zero());
    if let Some(inverse) = inverse {
        assert_eq!(
            mul(&aa, &decode(inverse), p),
            (BigUint::from(1u8), BigUint::from(0u8))
        );
    }
    let root = a.sqrt();
    assert_eq!(root.is_some(), is_square(&aa, p));
    if let Some(root) = root {
        assert_eq!(mul(&decode(root), &decode(root), p), aa);
        assert!(root.to_be_bytes() <= root.negate().to_be_bytes());
    }
}
