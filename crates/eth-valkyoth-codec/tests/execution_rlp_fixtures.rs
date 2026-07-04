//! External Ethereum RLP fixture conformance tests.

use std::{env, error::Error, fs, path::Path, path::PathBuf, str, string::String, vec, vec::Vec};

use eth_valkyoth_codec::{
    DecodeError, DecodeLimits, decode_rlp_list, decode_rlp_scalar, encode_decoded_list,
    encode_decoded_scalar,
};
use serde_json::Value;

const FIXTURE_DIR_ENV: &str = "ETH_EXECUTION_TESTS_RLP_DIR";

#[test]
fn pinned_ethereum_rlp_fixtures_match_codec() -> Result<(), Box<dyn Error>> {
    let Some(dir) = env::var_os(FIXTURE_DIR_ENV) else {
        return Ok(());
    };
    let paths = json_files(PathBuf::from(dir))?;
    if paths.is_empty() {
        return Err("no RLP fixture JSON files found".into());
    }

    let mut cases_seen = 0usize;
    let mut failures = Vec::new();
    for path in paths {
        run_fixture_file(&path, &mut cases_seen, &mut failures)?;
    }

    if cases_seen == 0 {
        return Err("no RLP fixture cases found".into());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

fn json_files(root: PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = Vec::new();
    collect_json_files(&root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_json_files(path: &Path, paths: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, paths)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn run_fixture_file(
    path: &Path,
    cases_seen: &mut usize,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&text)?;
    let Some(cases) = json.as_object() else {
        return Err(format!("{} is not a JSON object", path.display()).into());
    };

    for (name, case) in cases {
        *cases_seen = cases_seen
            .checked_add(1)
            .ok_or("RLP fixture case counter overflow")?;
        run_case(path, name, case, failures)?;
    }
    Ok(())
}

fn run_case(
    path: &Path,
    name: &str,
    case: &Value,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let Some(output) = case.get("out").and_then(Value::as_str) else {
        return Err(format!("{}:{name} is missing string out", path.display()).into());
    };
    let encoded = decode_hex(output)?;
    let valid = !matches!(case.get("in"), Some(Value::String(marker)) if marker == "INVALID");

    match (valid, decode_and_reencode(&encoded)) {
        (true, Ok(round_trip)) if round_trip == encoded => Ok(()),
        (true, Ok(_)) => {
            failures.push(format!(
                "{}:{name} re-encoded to different bytes",
                path.display()
            ));
            Ok(())
        }
        (true, Err(error)) => {
            failures.push(format!(
                "{}:{name} rejected valid RLP: {error:?}",
                path.display()
            ));
            Ok(())
        }
        (false, Ok(_)) => {
            failures.push(format!("{}:{name} accepted invalid RLP", path.display()));
            Ok(())
        }
        (false, Err(_)) => Ok(()),
    }
}

fn decode_and_reencode(input: &[u8]) -> Result<Vec<u8>, DecodeError> {
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

fn decode_hex(input: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let hex = match input.strip_prefix("0x") {
        Some(stripped) => stripped,
        None => input,
    };
    let mut chunks = hex.as_bytes().chunks_exact(2);
    if !chunks.remainder().is_empty() {
        return Err(format!("hex string has odd length: {input}").into());
    }
    let mut output = Vec::new();
    for chunk in &mut chunks {
        let pair = str::from_utf8(chunk)?;
        output.push(u8::from_str_radix(pair, 16)?);
    }
    Ok(output)
}
