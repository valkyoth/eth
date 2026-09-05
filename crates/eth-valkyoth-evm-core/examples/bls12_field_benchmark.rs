//! Reproducible fixed-work public field benchmark, not a timing-safety proof.

use eth_valkyoth_evm_core::EvmBls12381Fp as Fp;
use std::{hint::black_box, time::Instant};

fn measure(name: &str, iterations: u32, mut operation: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    println!(
        "{name}: {} ns/op ({iterations} iterations)",
        start
            .elapsed()
            .as_nanos()
            .checked_div(u128::from(iterations))
            .unwrap_or(0)
    );
}

fn main() {
    let a = Fp::from_wide_be_bytes([0xa5; 96]);
    let b = Fp::from_u64(u64::MAX);
    measure("add", 10000, || {
        black_box(black_box(a).add_mod(black_box(b)));
    });
    measure("sub", 10000, || {
        black_box(black_box(a).sub_mod(black_box(b)));
    });
    measure("mul", 10000, || {
        black_box(black_box(a).mul_mod(black_box(b)));
    });
    measure("square", 10000, || {
        black_box(black_box(a).square());
    });
    measure("reduce768", 1000, || {
        black_box(Fp::from_wide_be_bytes(black_box([0xff; 96])));
    });
    measure("invert", 100, || {
        black_box(black_box(a).invert());
    });
    measure("sqrt", 100, || {
        black_box(black_box(a).sqrt());
    });
}
