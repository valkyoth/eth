//! All official G2 addition fixtures: standalone arithmetic, not charged execution.
use eth_valkyoth_evm_core::{EvmBls12381G2Affine as Affine, EvmCoreError};

fn decode(hex: &str) -> Result<Vec<u8>, EvmCoreError> {
    let (chunks, remainder) = hex.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(EvmCoreError::PrecompileInvalidInputLength);
    }
    chunks
        .iter()
        .map(|pair| {
            let text = core::str::from_utf8(pair)
                .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?;
            u8::from_str_radix(text, 16).map_err(|_| EvmCoreError::PrecompileInvalidInputLength)
        })
        .collect()
}

fn add_frame(input: &[u8]) -> Result<[u8; 256], EvmCoreError> {
    if input.len() != 512 {
        return Err(EvmCoreError::PrecompileInvalidInputLength);
    }
    let (a, b) = input.split_at(256);
    Ok(Affine::try_from_be_bytes(a)?
        .add_point(Affine::try_from_be_bytes(b)?)
        .to_be_bytes())
}

#[test]
fn official_addition_and_rejection_vectors() -> Result<(), EvmCoreError> {
    let mut positive = 0;
    for line in include_str!("fixtures/bls12_g2_add.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let parts: Vec<_> = line.split('|').collect();
        let [name, input, expected] = parts.as_slice() else {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        };
        let input = decode(input)?;
        let expected = decode(expected)?;
        assert_eq!(add_frame(&input)?.as_slice(), expected, "{name}");
        let (left, right) = input.split_at(256);
        let a = Affine::try_from_be_bytes(left)?.to_projective();
        let b = Affine::try_from_be_bytes(right)?.to_projective();
        assert_eq!(
            a.add_point(b).to_affine().to_be_bytes().as_slice(),
            expected,
            "{name}"
        );
        assert_eq!(a.add_point(b), b.add_point(a));
        positive += 1;
    }
    let mut negative = 0;
    for line in include_str!("fixtures/bls12_g2_add_fail.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let parts: Vec<_> = line.split('|').collect();
        let [name, input, error] = parts.as_slice() else {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        };
        let expected = match *error {
            "invalid input length" => EvmCoreError::PrecompileInvalidInputLength,
            "invalid point: not on curve" => EvmCoreError::PrecompilePointNotOnCurve,
            "invalid fp.Element encoding" | "invalid field element top bytes" => {
                EvmCoreError::PrecompileFieldElementOutOfRange
            }
            _ => return Err(EvmCoreError::PrecompileBackendUnavailable),
        };
        assert_eq!(add_frame(&decode(input)?), Err(expected), "{name}");
        negative += 1;
    }
    assert_eq!((positive, negative), (9, 7));
    Ok(())
}
