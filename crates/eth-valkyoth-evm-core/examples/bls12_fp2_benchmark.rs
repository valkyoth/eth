//! Fixed-work public-input Fp2 smoke; not portable gas or side-channel evidence.
use eth_valkyoth_evm_core::{EvmBls12381Fp as Fp, EvmBls12381Fp2 as Fp2, EvmCoreError};
use std::{hint::black_box, time::Instant};

fn main() -> Result<(), EvmCoreError> {
    let value = Fp2::from_coefficients(
        Fp::from_wide_be_bytes([0x55; 96]),
        Fp::from_wide_be_bytes([0xaa; 96]),
    );
    let square = value.square();
    let start = Instant::now();
    for _ in 0..1000 {
        let inverse = black_box(value)
            .invert()
            .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
        let root = black_box(square)
            .sqrt()
            .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)?;
        assert_eq!(value.mul_mod(inverse), Fp2::from_u64(1));
        assert_eq!(root.square(), square);
        black_box((inverse, root));
    }
    let elapsed = start.elapsed();
    println!(
        "fp2 inverse+sqrt: {} ns/pair; 1000 pairs; {:?} total",
        elapsed.as_nanos().checked_div(1000).unwrap_or(0),
        elapsed
    );
    assert!(
        elapsed.as_secs() < 5,
        "fixed-work smoke exceeded five-second host ceiling"
    );
    Ok(())
}
