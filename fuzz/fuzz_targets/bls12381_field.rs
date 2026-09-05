#![no_main]

use eth_valkyoth_evm_core::EvmBls12381Fp as Fp;
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;

fn integer(value: Fp) -> BigUint {
    BigUint::from_bytes_be(value.value_bytes())
}

fuzz_target!(|data: &[u8]| {
    let mut wide = [0; 96];
    for (out, byte) in wide.iter_mut().zip(data) {
        *out = *byte;
    }
    let a = Fp::from_wide_be_bytes(wide);
    let p = BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).expect("fixed public modulus");
    let expected_a = BigUint::from_bytes_be(&wide) % &p;
    assert_eq!(integer(a), expected_a);
    wide.reverse();
    let b = Fp::from_wide_be_bytes(wide);
    let expected_b = BigUint::from_bytes_be(&wide) % &p;
    assert_eq!(integer(b), expected_b);
    assert_eq!(integer(a.mul_mod(b)), (&expected_a * &expected_b) % &p);
    assert_eq!(integer(a.add_mod(b)), (&expected_a + &expected_b) % &p);
    assert_eq!(integer(a.sub_mod(b)), (&expected_a + &p - &expected_b) % &p);
    assert_eq!(a.add_mod(b).sub_mod(b), a);
    assert_eq!(a.mul_mod(b), b.mul_mod(a));
    assert_eq!(a.add_mod(a), a.mul_mod(Fp::from_u64(2)));
    assert_eq!(a.square(), a.mul_mod(a));
    assert!(a.add_mod(a.negate()).is_zero());
    assert_eq!(Fp::try_from_be_bytes(&a.to_be_bytes()), Ok(a));
    if let Some(inverse) = a.invert() {
        assert_eq!(a.mul_mod(inverse), Fp::from_u64(1));
        assert_eq!(integer(inverse), expected_a.modpow(&(&p - 2u8), &p));
    } else {
        assert!(a.is_zero());
    }
    let candidate = expected_a.modpow(&((&p + 1u8) >> 2), &p);
    let expected_root = if (&candidate * &candidate) % &p == expected_a {
        Some(candidate.clone().min(&p - candidate))
    } else {
        None
    };
    assert_eq!(a.sqrt().map(integer), expected_root);
    let square = a.square();
    assert!(square.sqrt().is_some_and(|root| root.square() == square));
});
