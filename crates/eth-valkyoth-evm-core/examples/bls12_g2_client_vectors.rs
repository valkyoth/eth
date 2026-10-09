//! Emit pinned G2 vectors after verifying first-party paid execution.
use eth_valkyoth_evm_core::{
    EvmBls12G2Add, EvmFork, EvmGas, EvmGasMeter, EvmPrecompileKind, EvmPrecompileRegistry,
    EvmPrecompileStatus,
};

fn main() -> Result<(), String> {
    for (prefix, body, success) in [
        (
            "valid",
            include_str!("../tests/fixtures/bls12_g2_add.txt"),
            true,
        ),
        (
            "invalid",
            include_str!("../tests/fixtures/bls12_g2_add_fail.txt"),
            false,
        ),
    ] {
        for (n, line) in body.lines().filter(|l| !l.starts_with('#')).enumerate() {
            let parts: Vec<_> = line.split('|').collect();
            let [_, input, expected] = parts.as_slice() else {
                return Err("fixture columns".into());
            };
            let (pairs, tail) = input.as_bytes().as_chunks::<2>();
            if !tail.is_empty() {
                return Err("fixture hex".into());
            }
            let bytes = pairs
                .iter()
                .map(|pair| {
                    let value = core::str::from_utf8(pair).map_err(|e| e.to_string())?;
                    u8::from_str_radix(value, 16).map_err(|e| e.to_string())
                })
                .collect::<Result<Vec<_>, String>>()?;
            let descriptor = EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)
                .and_then(|r| r.descriptor(EvmPrecompileKind::Bls12G2Add))
                .map_err(|e| format!("descriptor: {e:?}"))?;
            let mut gas = EvmGasMeter::try_new(EvmGas::new(1000)).map_err(|e| format!("{e:?}"))?;
            let mut output = [0xa5; 256];
            let result = descriptor
                .quote::<EvmBls12G2Add>(&bytes)
                .and_then(|q| q.authorize_and_execute_bls12_g2_add(&mut gas, &mut output));
            let actual = match result {
                Ok(outcome) if outcome.status() == EvmPrecompileStatus::Success => {
                    use std::fmt::Write;
                    let mut encoded = String::from("0x");
                    for byte in output {
                        write!(encoded, "{byte:02x}").map_err(|e| e.to_string())?;
                    }
                    encoded
                }
                _ => String::from("ERROR"),
            };
            let expected = if success {
                format!("0x{expected}")
            } else {
                String::from("ERROR")
            };
            if actual != expected {
                return Err(format!("{prefix}-{n}: fixture mismatch"));
            }
            println!("{prefix}-{n}\t0x{input}\t{actual}");
        }
    }
    Ok(())
}
