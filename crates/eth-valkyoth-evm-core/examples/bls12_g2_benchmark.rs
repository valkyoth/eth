//! Fixed-work public-point smoke, not gas calibration or side-channel evidence.
use eth_valkyoth_evm_core::{
    EvmBls12381Fp as Fp, EvmBls12381Fp2 as Fp2, EvmBls12381G2Affine as G2, EvmCoreError,
};
use std::{hint::black_box, time::Instant};

fn main() -> Result<(), EvmCoreError> {
    let x = Fp2::from_u64(2);
    let y = x
        .square()
        .mul_mod(x)
        .add_mod(Fp2::from_coefficients(Fp::from_u64(4), Fp::from_u64(4)))
        .sqrt()
        .ok_or(EvmCoreError::PrecompilePointNotOnCurve)?;
    let point = G2::try_from_coordinates(x, y)?;
    let other = point.double();
    let expected = point.add_point(other);
    let start = Instant::now();
    for _ in 0..1000 {
        let result = black_box(point).add_point(black_box(other));
        assert_eq!(result, expected);
        black_box(result);
    }
    let elapsed = start.elapsed();
    println!(
        "g2 affine addition: {} ns/call; 1000 calls; {:?} total",
        elapsed.as_nanos().checked_div(1000).unwrap_or(0),
        elapsed
    );
    assert!(
        elapsed.as_secs() < 5,
        "fixed-work smoke exceeded five-second host ceiling"
    );
    Ok(())
}
