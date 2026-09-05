//! Independent public-input arithmetic oracle; never a runtime dependency.

use eth_valkyoth_evm_core::{EvmBls12381Fp as Fp, EvmCoreError};
use num_bigint::BigUint;

fn modulus() -> Result<BigUint, EvmCoreError> {
    // EIP-2537's base-field modulus, independently specified from the kernel.
    BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16)
        .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)
}

fn field(value: &BigUint) -> Result<Fp, EvmCoreError> {
    let raw = value.to_bytes_be();
    let mut encoded = [0; 64];
    let start = 64usize
        .checked_sub(raw.len())
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    encoded
        .get_mut(start..)
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?
        .copy_from_slice(&raw);
    Fp::try_from_be_bytes(&encoded)
}

fn integer(value: Fp) -> BigUint {
    BigUint::from_bytes_be(value.value_bytes())
}

fn sample(state: &mut u64) -> [u8; 96] {
    let mut bytes = [0; 96];
    for chunk in bytes.chunks_exact_mut(8) {
        // Reproducible, noncryptographic test generator, not production entropy.
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        chunk.copy_from_slice(&state.to_be_bytes());
    }
    bytes
}

#[test]
fn arithmetic_matches_biguint_at_every_limb_boundary() -> Result<(), EvmCoreError> {
    let p = modulus()?;
    let mut values = vec![BigUint::from(0u8), BigUint::from(1u8), &p - 1u8, &p - 2u8];
    for bit in [63usize, 64, 65, 127, 128, 191, 192, 255, 256, 319, 320, 380] {
        let power = BigUint::from(1u8) << bit;
        values.extend([&power - 1u8, power.clone(), &power + 1u8, &p - power]);
    }
    for a in &values {
        let fa = field(a)?;
        assert_eq!(integer(fa), *a);
        assert_eq!(integer(fa.square()), (a * a) % &p);
        assert_eq!(integer(fa.negate()), (&p - a) % &p);
        for b in &values {
            let fb = field(b)?;
            assert_eq!(integer(fa.add_mod(fb)), (a + b) % &p);
            assert_eq!(integer(fa.sub_mod(fb)), (a + &p - b) % &p);
            assert_eq!(integer(fa.mul_mod(fb)), (a * b) % &p);
        }
    }
    Ok(())
}

#[test]
fn reduction_and_operations_match_independent_random_residues() -> Result<(), EvmCoreError> {
    let p = modulus()?;
    let mut state = 1;
    for _ in 0..256 {
        let wide = sample(&mut state);
        let a = BigUint::from_bytes_be(&wide) % &p;
        let b = BigUint::from_bytes_be(&sample(&mut state)) % &p;
        let fa = Fp::from_wide_be_bytes(wide);
        let fb = field(&b)?;
        assert_eq!(integer(fa), a);
        assert_eq!(integer(fa.mul_mod(fb)), (&a * &b) % &p);
        assert_eq!(integer(fa.add_mod(fb)), (&a + &b) % &p);
        assert_eq!(integer(fa.sub_mod(fb)), (&a + &p - &b) % &p);
    }
    for bit in 0..768 {
        let mut wide = [0; 96];
        if let Some(out) = wide.get_mut(95 - bit / 8) {
            *out = 1 << (bit % 8);
        }
        assert_eq!(
            integer(Fp::from_wide_be_bytes(wide)),
            (BigUint::from(1u8) << bit) % &p
        );
    }
    for byte in [0, 1, 0x55, 0xaa, 0xff] {
        let wide = [byte; 96];
        assert_eq!(
            integer(Fp::from_wide_be_bytes(wide)),
            BigUint::from_bytes_be(&wide) % &p
        );
    }
    Ok(())
}

#[test]
fn inverses_and_roots_match_modpow_and_reject_nonsquares() -> Result<(), EvmCoreError> {
    let p = modulus()?;
    let mut state = 79;
    let zero = Fp::from_u64(0);
    assert_eq!(zero.invert(), None);
    assert_eq!(zero.sqrt(), Some(zero));
    assert_eq!(field(&(&p - 1u8))?.sqrt(), None);
    for _ in 0..64 {
        let a = BigUint::from_bytes_be(&sample(&mut state)) % (&p - 1u8) + 1u8;
        let fa = field(&a)?;
        let inverse = fa
            .invert()
            .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
        assert_eq!(integer(inverse), a.modpow(&(&p - 2u8), &p));
        assert_eq!(fa.mul_mod(inverse), Fp::from_u64(1));
        let root = fa
            .square()
            .sqrt()
            .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
        assert_eq!(integer(root), a.clone().min(&p - &a));
        let candidate = a.modpow(&((&p + 1u8) >> 2), &p);
        if (&candidate * &candidate) % &p == a {
            assert_eq!(
                fa.sqrt().map(integer),
                Some(candidate.clone().min(&p - candidate))
            );
        } else {
            assert_eq!(fa.sqrt(), None);
        }
    }
    Ok(())
}

#[test]
fn wire_decode_does_not_reduce_and_u64_conversion_is_exact() -> Result<(), EvmCoreError> {
    let p = modulus()?;
    assert_eq!(
        field(&p),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
    assert_eq!(
        field(&(&p + 1u8)),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
    for value in [0, 1, 2, u64::MAX, u64::MAX - 1] {
        let value = Fp::from_u64(value);
        assert_eq!(Fp::try_from_be_bytes(&value.to_be_bytes())?, value);
    }
    assert_eq!(integer(Fp::from_u64(u64::MAX)), BigUint::from(u64::MAX));
    assert_eq!(core::mem::size_of::<Fp>(), 48);
    for len in [0, 48, 63, 65, 128] {
        assert_eq!(
            Fp::try_from_be_bytes(&vec![0; len]),
            Err(EvmCoreError::PrecompileInvalidInputLength)
        );
    }
    let mut bad = [0; 64];
    bad[0] = 1;
    assert_eq!(
        Fp::try_from_be_bytes(&bad),
        Err(EvmCoreError::PrecompileFieldElementOutOfRange)
    );
    Ok(())
}
