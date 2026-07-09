extern crate std;

use std::vec::Vec;

use crate::{EVM_BN254_PAIRING_OUTPUT_BYTES, EvmCoreError, bn254_pairing::execute_bn254_pairing};

#[test]
fn bn254_pairing_matches_geth_one_point_negative_vector() -> Result<(), EvmCoreError> {
    let input = hex_bytes(
        "0000000000000000000000000000000000000000000000000000000000000001\
         0000000000000000000000000000000000000000000000000000000000000002\
         198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
         1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
         090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
         12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
    );
    assert_eq!(execute_vector(&input)?, word(0));
    Ok(())
}

#[test]
fn bn254_pairing_matches_geth_two_point_positive_vector() -> Result<(), EvmCoreError> {
    let input = hex_bytes(
        "0000000000000000000000000000000000000000000000000000000000000001\
         0000000000000000000000000000000000000000000000000000000000000002\
         198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
         1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
         090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
         12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa\
         0000000000000000000000000000000000000000000000000000000000000001\
         0000000000000000000000000000000000000000000000000000000000000002\
         198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
         1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
         275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec\
         1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d",
    );
    assert_eq!(execute_vector(&input)?, word(1));
    Ok(())
}

fn execute_vector(input: &[u8]) -> Result<[u8; EVM_BN254_PAIRING_OUTPUT_BYTES], EvmCoreError> {
    let mut output = [0xabu8; EVM_BN254_PAIRING_OUTPUT_BYTES];
    assert_eq!(
        execute_bn254_pairing(input, &mut output)?,
        EVM_BN254_PAIRING_OUTPUT_BYTES
    );
    Ok(output)
}

fn word(value: u8) -> [u8; EVM_BN254_PAIRING_OUTPUT_BYTES] {
    let mut output = [0u8; EVM_BN254_PAIRING_OUTPUT_BYTES];
    if let Some(last) = output.last_mut() {
        *last = value;
    }
    output
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    let compact: Vec<u8> = hex
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    assert!(compact.len().is_multiple_of(2));
    compact.chunks_exact(2).map(hex_pair).collect()
}

fn hex_pair(pair: &[u8]) -> u8 {
    let high = pair.first().copied().map(hex_nibble).unwrap_or(0);
    let low = pair.get(1).copied().map(hex_nibble).unwrap_or(0);
    (high << 4) | low
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte.saturating_sub(b'0'),
        b'a'..=b'f' => byte.saturating_sub(b'a').saturating_add(10),
        b'A'..=b'F' => byte.saturating_sub(b'A').saturating_add(10),
        _ => 0,
    }
}
