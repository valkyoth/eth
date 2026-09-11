#![no_main]
use eth_valkyoth_evm_core::{EvmBls12381G2Affine as Affine, EvmCoreError};
use libfuzzer_sys::fuzz_target;
#[path = "../../crates/eth-valkyoth-evm-core/tests/support/bls12_g2_oracle.rs"]
mod oracle;

fn check(data: &[u8]) -> Result<(), EvmCoreError> {
    let decoded = Affine::try_from_be_bytes(data);
    let reference = oracle::decode(data);
    assert_eq!(decoded.is_ok(), reference.is_ok());
    if let (Ok(actual), Ok(expected)) = (decoded, reference) {
        assert_eq!(actual.to_be_bytes(), oracle::encode(&expected)?);
    }
    let mut bytes = [0; 96];
    for (out, byte) in bytes.iter_mut().zip(data) {
        *out = *byte;
    }
    let point = oracle::from_x(&bytes[..48], &bytes[48..])?;
    let generator = oracle::generator()?;
    let a = Affine::try_from_be_bytes(&oracle::encode(&point)?)?.to_projective();
    let b = Affine::try_from_be_bytes(&oracle::encode(&generator)?)?.to_projective();
    assert_eq!(
        a.add_point(b).to_affine().to_be_bytes(),
        oracle::encode(&oracle::add_point(&point, &generator)?)?
    );
    assert_eq!(
        a.double().to_affine().to_be_bytes(),
        oracle::encode(&oracle::add_point(&point, &point)?)?
    );
    assert_eq!(a.double(), a.add_point(a));
    assert!(a.add_point(a.negate()).is_infinity());
    let a2 = oracle::add_point(&point, &point)?;
    let b2 = oracle::add_point(&generator, &generator)?;
    assert_eq!(
        a.double().add_point(b.double()).to_affine().to_be_bytes(),
        oracle::encode(&oracle::add_point(&a2, &b2)?)?
    );
    Ok(())
}
fuzz_target!(|data: &[u8]| {
    assert!(check(data).is_ok());
});
