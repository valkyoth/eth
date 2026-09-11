//! Test-only affine polynomial oracle. No production field or point arithmetic.
use eth_valkyoth_evm_core::EvmCoreError;
use num_bigint::BigUint;
use std::ops::{Add, Mul, Rem, Shr, Sub};

#[allow(dead_code)]
#[path = "bls12_fp2_oracle.rs"]
mod field;
use field::{Pair, modulus, mul};

pub type Point = Option<(Pair, Pair)>;

fn zero() -> Pair {
    (BigUint::from(0u8), BigUint::from(0u8))
}
fn small(a: u8, b: u8) -> Pair {
    (BigUint::from(a), BigUint::from(b))
}
fn add(a: &Pair, b: &Pair, p: &BigUint) -> Pair {
    ((&a.0).add(&b.0).rem(p), (&a.1).add(&b.1).rem(p))
}
fn sub(a: &Pair, b: &Pair, p: &BigUint) -> Pair {
    (
        (&a.0).add(p).sub(&b.0).rem(p),
        (&a.1).add(p).sub(&b.1).rem(p),
    )
}
fn inverse(a: &Pair, p: &BigUint) -> Pair {
    let norm = (&a.0).mul(&a.0).add((&a.1).mul(&a.1)).rem(p);
    assert_ne!(norm, BigUint::from(0u8));
    let inverse = norm.modpow(&p.sub(2u8), p);
    ((&a.0).mul(&inverse).rem(p), p.sub(&a.1).mul(inverse).rem(p))
}

pub fn generator() -> Result<Point, EvmCoreError> {
    let n = |s: &[u8]| {
        BigUint::parse_bytes(s, 16).ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)
    };
    Ok(Some((
        (n(b"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8")?,
         n(b"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e")?),
        (n(b"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801")?,
         n(b"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be")?),
    )))
}

fn sqrt_fp(a: &BigUint, p: &BigUint) -> Option<BigUint> {
    let root = a.modpow(&p.add(1u8).shr(2usize), p);
    ((&root).mul(&root).rem(p) == *a).then_some(root)
}

fn sqrt(a: &Pair, p: &BigUint) -> Option<Pair> {
    if a.1 == BigUint::from(0u8) {
        return sqrt_fp(&a.0, p)
            .map(|r| (r, BigUint::from(0u8)))
            .or_else(|| sqrt_fp(&p.sub(&a.0).rem(p), p).map(|r| (BigUint::from(0u8), r)));
    }
    let norm = (&a.0).mul(&a.0).add((&a.1).mul(&a.1)).rem(p);
    let norm_root = sqrt_fp(&norm, p)?;
    let half = p.add(1u8).shr(1usize);
    for signed in [norm_root.clone(), p.sub(&norm_root)] {
        let delta = (&a.0).add(signed).mul(&half).rem(p);
        if let Some(x) = sqrt_fp(&delta, p) {
            if x == BigUint::from(0u8) {
                continue;
            }
            let y = (&a.1).mul((&x).mul(2u8).modpow(&p.sub(2u8), p)).rem(p);
            let result = (x, y);
            if mul(&result, &result, p) == *a {
                return Some(result);
            }
        }
    }
    None
}

pub fn from_x(a: &[u8], b: &[u8]) -> Result<Point, EvmCoreError> {
    let p = modulus()?;
    let x = (
        BigUint::from_bytes_be(a).rem(&p),
        BigUint::from_bytes_be(b).rem(&p),
    );
    let rhs = add(&mul(&mul(&x, &x, &p), &x, &p), &small(4, 4), &p);
    Ok(sqrt(&rhs, &p).map(|y| (x, y)))
}

pub fn add_point(a: &Point, b: &Point) -> Result<Point, EvmCoreError> {
    let p = modulus()?;
    let (Some((x1, y1)), Some((x2, y2))) = (a, b) else {
        return Ok(if a.is_none() { b.clone() } else { a.clone() });
    };
    let (numerator, denominator) = if x1 == x2 {
        if add(y1, y2, &p) == zero() {
            return Ok(None);
        }
        (
            mul(&small(3, 0), &mul(x1, x1, &p), &p),
            mul(&small(2, 0), y1, &p),
        )
    } else {
        (sub(y2, y1, &p), sub(x2, x1, &p))
    };
    let slope = mul(&numerator, &inverse(&denominator, &p), &p);
    let x = sub(&sub(&mul(&slope, &slope, &p), x1, &p), x2, &p);
    let y = sub(&mul(&slope, &sub(x1, &x, &p), &p), y1, &p);
    Ok(Some((x, y)))
}

pub fn encode(point: &Point) -> Result<[u8; 256], EvmCoreError> {
    let mut output = [0; 256];
    if let Some((x, y)) = point {
        let (left, right) = output.split_at_mut(128);
        left.copy_from_slice(&field::encode(x)?.to_be_bytes());
        right.copy_from_slice(&field::encode(y)?.to_be_bytes());
    }
    Ok(output)
}

pub fn decode(input: &[u8]) -> Result<Point, EvmCoreError> {
    if input.len() != 256 {
        return Err(EvmCoreError::PrecompileInvalidInputLength);
    }
    let mut values = input
        .as_chunks::<64>()
        .0
        .iter()
        .map(|v| BigUint::from_bytes_be(v));
    let mut next = || {
        values
            .next()
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)
    };
    let x = (next()?, next()?);
    let y = (next()?, next()?);
    let p = modulus()?;
    if [&x.0, &x.1, &y.0, &y.1].into_iter().any(|v| v >= &p) {
        return Err(EvmCoreError::PrecompileFieldElementOutOfRange);
    }
    if x == zero() && y == zero() {
        return Ok(None);
    }
    if mul(&y, &y, &p) != add(&mul(&mul(&x, &x, &p), &x, &p), &small(4, 4), &p) {
        return Err(EvmCoreError::PrecompilePointNotOnCurve);
    }
    Ok(Some((x, y)))
}
