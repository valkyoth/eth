//! Test-only affine BigUint oracle, independent of production field/Jacobian code.

use eth_valkyoth_evm_core::EvmCoreError;
use num_bigint::BigUint;
use std::ops::{Add, Mul, Rem, Shr, Sub};

pub type Point = Option<(BigUint, BigUint)>;

pub fn modulus() -> Result<BigUint, EvmCoreError> {
    number(
        "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab",
    )
}

fn number(hex: &str) -> Result<BigUint, EvmCoreError> {
    BigUint::parse_bytes(hex.as_bytes(), 16).ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)
}

pub fn generator() -> Result<Point, EvmCoreError> {
    // EIP-2537 H1, not produced by the implementation under test.
    Ok(Some((
        number(
            "17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
        )?,
        number(
            "08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1",
        )?,
    )))
}

pub fn from_x(x: BigUint) -> Result<Point, EvmCoreError> {
    let p = modulus()?;
    // Fixed nonzero EIP modulus; explicit BigUint remainder and shift APIs.
    let x = x.rem(&p);
    let rhs = (&x).mul(&x).mul(&x).add(4u8).rem(&p);
    let y = rhs.modpow(&(&p).add(1u8).shr(2usize), &p);
    Ok(if (&y).mul(&y).rem(&p) == rhs {
        Some((x, y))
    } else {
        None
    })
}

pub fn add(a: &Point, b: &Point) -> Result<Point, EvmCoreError> {
    let p = modulus()?;
    let (Some((x1, y1)), Some((x2, y2))) = (a, b) else {
        return Ok(if a.is_none() { b.clone() } else { a.clone() });
    };
    let (numerator, denominator) = if x1 == x2 {
        if y1.add(y2).rem(&p) == BigUint::from(0u8) {
            return Ok(None);
        }
        (x1.mul(x1).mul(3u8).rem(&p), y1.mul(2u8).rem(&p))
    } else {
        (y2.add(&p).sub(y1).rem(&p), x2.add(&p).sub(x1).rem(&p))
    };
    let slope = numerator
        .mul(denominator.modpow(&(&p).sub(2u8), &p))
        .rem(&p);
    let x = (&slope).mul(&slope).add(&p).add(&p).sub(x1).sub(x2).rem(&p);
    let y = slope.mul(x1.add(&p).sub(&x)).add(&p).sub(y1).rem(&p);
    Ok(Some((x, y)))
}

pub fn encode(point: &Point) -> Result<[u8; 128], EvmCoreError> {
    let mut output = [0; 128];
    if let Some((x, y)) = point {
        for (chunk, value) in output.as_chunks_mut::<64>().0.iter_mut().zip([x, y]) {
            let bytes = value.to_bytes_be();
            let start = 64usize
                .checked_sub(bytes.len())
                .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
            chunk
                .get_mut(start..)
                .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
                .copy_from_slice(&bytes);
        }
    }
    Ok(output)
}

pub fn decode(input: &[u8]) -> Result<Point, EvmCoreError> {
    if input.len() != 128 {
        return Err(EvmCoreError::PrecompileInvalidInputLength);
    }
    let (left, right) = input.split_at(64);
    let p = modulus()?;
    let x = BigUint::from_bytes_be(left);
    let y = BigUint::from_bytes_be(right);
    if x >= p || y >= p {
        return Err(EvmCoreError::PrecompileFieldElementOutOfRange);
    }
    if x == BigUint::from(0u8) && y == BigUint::from(0u8) {
        return Ok(None);
    }
    if (&y).mul(&y).rem(&p) != (&x).mul(&x).mul(&x).add(4u8).rem(&p) {
        return Err(EvmCoreError::PrecompilePointNotOnCurve);
    }
    Ok(Some((x, y)))
}
