//! Emits deterministic ModExp frames and first-party results for client comparison.

use eth_valkyoth_evm_core::{
    EvmFork, EvmGasMeter, EvmModExpWorkspace, EvmModexp, EvmPrecompileKind, EvmPrecompileRegistry,
    EvmPrecompileStatus,
};

const HEADER_BYTES: usize = 96;

struct Case {
    name: &'static str,
    input: Vec<u8>,
}

fn main() -> Result<(), String> {
    for case in cases()? {
        let output = execute(&case.input)?;
        println!(
            "{}\t{}\t{}",
            case.name,
            encode_hex(&case.input)?,
            encode_hex(&output)?
        );
    }
    Ok(())
}

fn cases() -> Result<Vec<Case>, String> {
    let mut cases = vec![
        complete_case("scalar", &[5], &[3], &[7])?,
        complete_case("modulus-leading-zero-4", &[2], &[3], &[0, 0, 0, 0, 5])?,
        complete_case(
            "modulus-leading-zero-8",
            &[2],
            &[3],
            &[0, 0, 0, 0, 0, 0, 0, 0, 5],
        )?,
        complete_case(
            "modulus-leading-zero-12",
            &[2],
            &[3],
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5],
        )?,
        complete_case(
            "unequal-widths",
            &patterned_bytes(17, 37, 11),
            &[3],
            &patterned_bytes(65, 19, 7),
        )?,
        complete_case(
            "dense-80-byte",
            &patterned_bytes(80, 37, 11),
            &[1, 0, 1],
            &patterned_bytes(80, 19, 7),
        )?,
        complete_case(
            "dense-256-byte",
            &patterned_bytes(256, 37, 11),
            &[3],
            &patterned_bytes(256, 19, 7),
        )?,
    ];

    let base = patterned_bytes(65, 37, 11);
    let mut even_modulus = patterned_bytes(80, 19, 7);
    if let Some(last) = even_modulus.last_mut() {
        *last &= 0xfe;
    }
    cases.push(complete_case(
        "even-modulus",
        &base,
        &[0, 0, 0, 1],
        &even_modulus,
    )?);
    let short_base = base
        .get(..33)
        .ok_or_else(|| String::from("short-base fixture is unavailable"))?;
    cases.push(complete_case(
        "zero-modulus",
        short_base,
        &[0; 32],
        &[0; 65],
    )?);
    let mut sparse_exponent = vec![0; 32];
    if let Some(last) = sparse_exponent.last_mut() {
        *last = 1;
    }
    cases.push(complete_case(
        "sparse-exponent",
        short_base,
        &sparse_exponent,
        &even_modulus,
    )?);

    let mut truncated = frame(2, 2, 4)?;
    truncated.extend_from_slice(&[1, 2, 0, 3, 5]);
    cases.push(Case {
        name: "truncated-right-padding",
        input: truncated,
    });
    Ok(cases)
}

fn complete_case(
    name: &'static str,
    base: &[u8],
    exponent: &[u8],
    modulus: &[u8],
) -> Result<Case, String> {
    let mut input = frame(base.len(), exponent.len(), modulus.len())?;
    input.extend_from_slice(base);
    input.extend_from_slice(exponent);
    input.extend_from_slice(modulus);
    Ok(Case { name, input })
}

fn frame(base_len: usize, exponent_len: usize, modulus_len: usize) -> Result<Vec<u8>, String> {
    let mut input = vec![0; HEADER_BYTES];
    write_len(&mut input, 0, base_len)?;
    write_len(&mut input, 32, exponent_len)?;
    write_len(&mut input, 64, modulus_len)?;
    Ok(input)
}

fn write_len(input: &mut [u8], offset: usize, value: usize) -> Result<(), String> {
    let bytes = value.to_be_bytes();
    let end = offset
        .checked_add(32)
        .ok_or_else(|| String::from("length offset overflow"))?;
    let start = end
        .checked_sub(bytes.len())
        .ok_or_else(|| String::from("length word underflow"))?;
    input
        .get_mut(start..end)
        .ok_or_else(|| String::from("length word outside frame"))?
        .copy_from_slice(&bytes);
    Ok(())
}

fn execute(input: &[u8]) -> Result<Vec<u8>, String> {
    let registry = EvmPrecompileRegistry::try_new(EvmFork::BERLIN)
        .map_err(|error| format!("registry: {error:?}"))?;
    let descriptor = registry
        .descriptor(EvmPrecompileKind::Modexp)
        .map_err(|error| format!("descriptor: {error:?}"))?;
    let quote = descriptor
        .quote::<EvmModexp>(input)
        .map_err(|error| format!("quote: {error:?}"))?;
    let mut gas =
        EvmGasMeter::try_new(quote.gas_cost()).map_err(|error| format!("gas meter: {error:?}"))?;
    let mut output = vec![0; quote.output_len()];
    let workspace_len = quote
        .modexp_workspace_limbs()
        .map_err(|error| format!("workspace size: {error:?}"))?;
    let mut storage = vec![0; workspace_len];
    let mut workspace = EvmModExpWorkspace::new(&mut storage);
    let outcome = quote
        .authorize_and_execute_modexp(&mut gas, &mut output, &mut workspace)
        .map_err(|error| format!("execution admission: {error:?}"))?;
    if outcome.status() != EvmPrecompileStatus::Success {
        return Err(format!("execution failed: {:?}", outcome.error()));
    }
    Ok(output)
}

fn patterned_bytes(length: usize, multiplier: usize, addend: usize) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let value = index.wrapping_mul(multiplier).wrapping_add(addend) & 0xff;
            u8::try_from(value).unwrap_or(0)
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> Result<String, String> {
    let capacity = bytes
        .len()
        .checked_mul(2)
        .and_then(|length| length.checked_add(2))
        .ok_or_else(|| String::from("hex capacity overflow"))?;
    let mut encoded = String::with_capacity(capacity);
    encoded.push_str("0x");
    for byte in bytes {
        use std::fmt::Write;
        write!(encoded, "{byte:02x}").map_err(|error| error.to_string())?;
    }
    Ok(encoded)
}
