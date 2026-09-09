//! Fixed-work public G1 benchmark. No gas or constant-time claim.

use eth_valkyoth_evm_core::{EvmBls12381Fp as Fp, EvmBls12381G1Affine as Affine, EvmCoreError};
use std::{hint::black_box, time::Instant};

fn measure(name: &str, mut operation: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..1000 {
        operation();
    }
    println!(
        "{name}: {} ns/op (1000 iterations)",
        start.elapsed().as_nanos().checked_div(1000).unwrap_or(0)
    );
}

fn main() -> Result<(), EvmCoreError> {
    let mut points = Vec::new();
    for value in 1..100 {
        let x = Fp::from_u64(value);
        if let Some(y) = x.square().mul_mod(x).add_mod(Fp::from_u64(4)).sqrt() {
            points.push(Affine::try_from_coordinates(x, y)?);
        }
    }
    let mut iter = points.into_iter();
    let a = iter.next().ok_or(EvmCoreError::PrecompilePointNotOnCurve)?;
    let b = iter.next().ok_or(EvmCoreError::PrecompilePointNotOnCurve)?;
    let ap = a.to_projective().double();
    let bp = b.to_projective().double();
    measure("projective_add", || {
        black_box(black_box(ap).add_point(black_box(bp)));
    });
    measure("projective_double", || {
        black_box(black_box(ap).double());
    });
    measure("normalize", || {
        black_box(black_box(ap).to_affine());
    });
    measure("affine_add", || {
        black_box(black_box(a).add_point(black_box(b)));
    });
    measure("on_curve_parse", || {
        black_box(Affine::try_from_be_bytes(black_box(&a.to_be_bytes()))).ok();
    });
    Ok(())
}
