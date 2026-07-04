//! Differential RLP checks against alloy-rlp.

use std::{error::Error, string::String, vec, vec::Vec};

use alloy_rlp::Header;
use eth_valkyoth_codec::{
    DecodeError, DecodeLimits, decode_rlp_list, decode_rlp_scalar, encode_decoded_list,
    encode_decoded_scalar,
};

#[test]
fn structural_rlp_matches_alloy_reference() -> Result<(), Box<dyn Error>> {
    let mut failures = Vec::new();

    for case in rlp_cases()? {
        let alloy = alloy_accepts(case.bytes.as_slice());
        let valkyoth = valkyoth_round_trip(case.bytes.as_slice());
        compare_case(&case, alloy, valkyoth, &mut failures);
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

fn compare_case(
    case: &Case,
    alloy: bool,
    valkyoth: Result<Vec<u8>, DecodeError>,
    failures: &mut Vec<String>,
) {
    match (case.valid, alloy, valkyoth) {
        (true, true, Ok(round_trip)) if round_trip == case.bytes => {}
        (true, false, _) => {
            failures.push(format!("{}: alloy rejected claimed-valid RLP", case.name))
        }
        (true, true, Ok(_)) => failures.push(format!(
            "{}: valkyoth re-encoded different bytes",
            case.name
        )),
        (true, true, Err(error)) => {
            failures.push(format!(
                "{}: valkyoth rejected alloy-accepted RLP: {error:?}",
                case.name
            ));
        }
        (false, false, Err(_)) => {}
        (false, true, _) => {
            failures.push(format!("{}: alloy accepted claimed-invalid RLP", case.name))
        }
        (false, false, Ok(_)) => {
            failures.push(format!(
                "{}: valkyoth accepted alloy-rejected RLP",
                case.name
            ));
        }
    }
}

fn alloy_accepts(input: &[u8]) -> bool {
    let mut remaining = input;
    match Header::decode_raw(&mut remaining) {
        Ok(_) => remaining.is_empty(),
        Err(_) => false,
    }
}

fn valkyoth_round_trip(input: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let prefix = *input.first().ok_or(DecodeError::Malformed)?;
    let mut output = vec![0_u8; input.len()];
    let written = if prefix <= 0xbf {
        let scalar = decode_rlp_scalar(input, DecodeLimits::TEST_FIXTURE)?;
        encode_decoded_scalar(scalar, &mut output)?
    } else {
        let list = decode_rlp_list(input, DecodeLimits::TEST_FIXTURE)?;
        encode_decoded_list(list, &mut output)?
    };
    output.truncate(written);
    Ok(output)
}

fn rlp_cases() -> Result<Vec<Case>, Box<dyn Error>> {
    Ok(vec![
        valid("single-byte scalar", vec![0x7f]),
        valid("empty scalar", vec![0x80]),
        valid("short scalar", vec![0x83, b'd', b'o', b'g']),
        valid("empty list", vec![0xc0]),
        valid(
            "short list",
            vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'],
        ),
        valid(
            "nested list",
            vec![0xc6, 0xc0, 0xc1, 0x80, 0xc2, 0x01, 0x02],
        ),
        valid("long scalar boundary", long_scalar_boundary()?),
        invalid("empty input", Vec::new()),
        invalid("noncanonical single-byte scalar", vec![0x81, 0x7f]),
        invalid("short scalar missing payload", vec![0x82, 0x01]),
        invalid("noncanonical long scalar length", vec![0xb8, 0x37, 0x00]),
        invalid("long scalar missing length byte", vec![0xb9, 0x01]),
        invalid("short list missing payload", vec![0xc2, 0x80]),
        invalid("noncanonical long list length", vec![0xf8, 0x00]),
    ])
}

fn long_scalar_boundary() -> Result<Vec<u8>, Box<dyn Error>> {
    let payload = vec![b'a'; 56];
    let mut encoded = Vec::with_capacity(58);
    encoded.push(0xb8);
    encoded.push(u8::try_from(payload.len())?);
    encoded.extend_from_slice(&payload);
    Ok(encoded)
}

fn valid(name: &'static str, bytes: Vec<u8>) -> Case {
    Case {
        name,
        bytes,
        valid: true,
    }
}

fn invalid(name: &'static str, bytes: Vec<u8>) -> Case {
    Case {
        name,
        bytes,
        valid: false,
    }
}

struct Case {
    name: &'static str,
    bytes: Vec<u8>,
    valid: bool,
}
