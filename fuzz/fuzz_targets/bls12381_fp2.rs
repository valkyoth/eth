#![no_main]
use eth_valkyoth_evm_core::{EvmBls12381Fp2 as Fp2, EvmCoreError};
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;
#[path = "../../crates/eth-valkyoth-evm-core/tests/support/bls12_fp2_oracle.rs"]
mod oracle;

fn check(data: &[u8]) -> Result<(), EvmCoreError> {
    let p = oracle::modulus()?;
    if let Ok(value) = Fp2::try_from_be_bytes(data) {
        assert_eq!(value.to_be_bytes().as_slice(), data);
        oracle::check(value, value.conjugate(), &p);
    }
    let mut bytes = [0; 192];
    for (out, byte) in bytes.iter_mut().zip(data) {
        *out = *byte;
    }
    let mut coefficients = bytes
        .as_chunks::<48>()
        .0
        .iter()
        .map(|part| BigUint::from_bytes_be(part) % &p);
    let mut coefficient = || {
        coefficients
            .next()
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)
    };
    let a = oracle::encode(&(coefficient()?, coefficient()?))?;
    let b = oracle::encode(&(coefficient()?, coefficient()?))?;
    oracle::check(a, b, &p);
    let square = a.square();
    oracle::check(square, b, &p);
    // Exercise the independent extension exponent as well as the norm criterion.
    assert_eq!(
        oracle::pow(&oracle::decode(a), &p, &p),
        oracle::decode(a.conjugate())
    );
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    assert!(check(data).is_ok());
});
