#![no_main]

#[path = "../../crates/eth-valkyoth-evm-core/tests/support/bls12_g1_oracle.rs"]
mod oracle;

use eth_valkyoth_evm_core::EvmBls12381G1Affine as Affine;
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;

fuzz_target!(|data: &[u8]| {
    assert_eq!(
        Affine::try_from_be_bytes(data).map(Affine::to_be_bytes),
        oracle::decode(data).and_then(|point| oracle::encode(&point))
    );
    if let Ok(point) = Affine::try_from_be_bytes(data) {
        assert_eq!(point.to_be_bytes().as_slice(), data);
        assert!(
            point
                .to_projective()
                .add_point(point.negate().to_projective())
                .is_infinity()
        );
    }
    // Independent point generation avoids spending the corpus on off-curve bytes.
    let mut seed = [0; 48];
    for (out, byte) in seed.iter_mut().zip(data) {
        *out = *byte;
    }
    let left = oracle::from_x(BigUint::from_bytes_be(&seed)).expect("fixed field");
    seed.reverse();
    let right = oracle::from_x(BigUint::from_bytes_be(&seed)).expect("fixed field");
    let a = Affine::try_from_be_bytes(&oracle::encode(&left).expect("bounded point"))
        .expect("oracle on curve");
    let b = Affine::try_from_be_bytes(&oracle::encode(&right).expect("bounded point"))
        .expect("oracle on curve");
    let expected = oracle::add(&left, &right).expect("fixed field");
    assert_eq!(
        a.add_point(b).to_be_bytes(),
        oracle::encode(&expected).expect("bounded result")
    );
    let aa = oracle::add(&left, &left).expect("fixed field");
    let bb = oracle::add(&right, &right).expect("fixed field");
    let ap = a.to_projective().double();
    let bp = b.to_projective().double();
    assert_eq!(
        ap.add_point(bp).to_affine().to_be_bytes(),
        oracle::encode(&oracle::add(&aa, &bb).expect("fixed field")).expect("bounded result")
    );
    let g = Affine::try_from_be_bytes(
        &oracle::encode(&oracle::generator().expect("fixed generator")).expect("bounded generator"),
    )
    .expect("on curve")
    .to_projective();
    assert_eq!(ap.add_point(bp).add_point(g), ap.add_point(bp.add_point(g)));
    assert_eq!(ap.add_point(ap), ap.double());
    assert_eq!(ap.add_point(bp), bp.add_point(ap));
    assert!(ap.add_point(ap.negate()).is_infinity());
});
