//! Complete charged public-input path; host smoke evidence, not a portable gas calibration.
use eth_valkyoth_evm_core::{
    EvmBls12G1Add, EvmBls12381Fp as Fp, EvmBls12381G1Affine as Point, EvmCoreError, EvmFork,
    EvmGas, EvmGasMeter, EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus,
};
use std::{hint::black_box, time::Instant};

fn main() -> Result<(), EvmCoreError> {
    let mut point = None;
    for x in 1..100 {
        let x = Fp::from_u64(x);
        if let Some(y) = x.square().mul_mod(x).add_mod(Fp::from_u64(4)).sqrt() {
            point = Some(Point::try_from_coordinates(x, y)?);
            break;
        }
    }
    let point = point.ok_or(EvmCoreError::PrecompilePointNotOnCurve)?;
    let mut input = [0; 256];
    input[..128].copy_from_slice(&point.to_be_bytes());
    input[128..].copy_from_slice(&point.double().to_be_bytes());
    let expected = point.add_point(point.double()).to_be_bytes();
    let descriptor = EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?
        .descriptor(EvmPrecompileKind::Bls12G1Add)?;
    let start = Instant::now();
    for _ in 0..1000 {
        let mut meter = EvmGasMeter::try_new(EvmGas::new(375))?;
        let mut output = [0; 128];
        let result = descriptor
            .quote::<EvmBls12G1Add>(black_box(&input))?
            .authorize_and_execute_bls12_g1_add(&mut meter, black_box(&mut output))?;
        assert_eq!(result.status(), EvmPrecompileStatus::Success);
        assert_eq!(output, expected);
        assert_eq!(meter.used(), EvmGas::new(375));
        black_box(output);
    }
    let elapsed = start.elapsed();
    println!(
        "charged_g1_add: {} ns/op; 375 gas/call; 1000 calls; {:?} total",
        elapsed.as_nanos().checked_div(1000).unwrap_or(0),
        elapsed
    );
    assert!(
        elapsed.as_secs() < 1,
        "fixed-work smoke exceeded the one-second host ceiling"
    );
    Ok(())
}
