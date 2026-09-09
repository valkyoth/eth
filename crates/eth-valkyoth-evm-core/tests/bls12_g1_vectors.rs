//! All nine official addition vectors, tested as standalone arithmetic, not dispatch.

use eth_valkyoth_evm_core::{EvmBls12381G1Affine as Affine, EvmCoreError};

fn decode<const N: usize>(hex: &str) -> Result<[u8; N], EvmCoreError> {
    if hex.len() != N.saturating_mul(2) {
        return Err(EvmCoreError::PrecompileInvalidInputLength);
    }
    let mut result = [0; N];
    for (out, pair) in result.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        let text =
            core::str::from_utf8(pair).map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?;
        *out =
            u8::from_str_radix(text, 16).map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?;
    }
    Ok(result)
}

#[test]
fn all_official_g1_addition_vectors_match() -> Result<(), EvmCoreError> {
    let mut count = 0usize;
    for line in include_str!("fixtures/bls12_g1_add.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let mut fields = line.split('|');
        let name = fields
            .next()
            .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
        let input = decode::<256>(
            fields
                .next()
                .ok_or(EvmCoreError::PrecompileInvalidInputLength)?,
        )?;
        let expected = decode::<128>(
            fields
                .next()
                .ok_or(EvmCoreError::PrecompileInvalidInputLength)?,
        )?;
        assert!(fields.next().is_none());
        let (left, right) = input.split_at(128);
        let a = Affine::try_from_be_bytes(left)?;
        let b = Affine::try_from_be_bytes(right)?;
        assert_eq!(a.add_point(b).to_be_bytes(), expected, "{name}");
        assert_eq!(
            a.to_projective()
                .add_point(b.to_projective())
                .to_affine()
                .to_be_bytes(),
            expected,
            "{name}"
        );
        count = count.saturating_add(1);
    }
    assert_eq!(count, 9);
    Ok(())
}
