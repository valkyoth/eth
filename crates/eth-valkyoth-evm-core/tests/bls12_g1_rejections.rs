//! Every official G1ADD negative fixture, without treating admission errors as CALL success.
use eth_valkyoth_evm_core::{
    EvmBls12G1Add, EvmCoreError, EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind,
    EvmPrecompileRegistry, EvmPrecompileStatus,
};

#[test]
fn official_negative_frames_fail_with_unchanged_output() -> Result<(), EvmCoreError> {
    let mut count = 0;
    for line in include_str!("fixtures/bls12_g1_add_fail.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('|').collect();
        let [name, encoded, error] = fields.as_slice() else {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        };
        assert!(encoded.len().is_multiple_of(2));
        let mut input = Vec::new();
        for pair in encoded.as_bytes().chunks_exact(2) {
            let text = core::str::from_utf8(pair)
                .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?;
            input.push(
                u8::from_str_radix(text, 16)
                    .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?,
            );
        }
        let expected = match *error {
            "invalid input length" => EvmCoreError::PrecompileInvalidInputLength,
            "invalid point: not on curve" => EvmCoreError::PrecompilePointNotOnCurve,
            "invalid fp.Element encoding" | "invalid field element top bytes" => {
                EvmCoreError::PrecompileFieldElementOutOfRange
            }
            _ => return Err(EvmCoreError::PrecompileBackendUnavailable),
        };
        let descriptor = EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?
            .descriptor(EvmPrecompileKind::Bls12G1Add)?;
        let mut meter = EvmGasMeter::try_new(EvmGas::new(999))?;
        let mut output = [0xa5; 129];
        match descriptor.quote::<EvmBls12G1Add>(&input) {
            Err(error) => {
                assert_eq!(error, expected, "{name}");
                assert_eq!(meter.used(), EvmGas::new(0));
            }
            Ok(quote) => {
                let result = quote.authorize_and_execute_bls12_g1_add(&mut meter, &mut output)?;
                assert_eq!(result.status(), EvmPrecompileStatus::CallFailure, "{name}");
                assert_eq!(result.error(), Some(expected), "{name}");
                assert_eq!(result.output_len(), 0);
                assert_eq!(result.gas_consumed(), EvmGas::new(999));
                assert_eq!(meter.used(), meter.limit());
            }
        }
        assert_eq!(output, [0xa5; 129]);
        count += 1;
    }
    assert_eq!(count, 7);
    Ok(())
}
