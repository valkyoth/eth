use std::vec::Vec;

pub(crate) fn dense_bn254_mul() -> [u8; 96] {
    let mut input = [0_u8; 96];
    input[31] = 1;
    input[63] = 2;
    input[64..].fill(u8::MAX);
    input
}

pub(crate) fn generator_bn254_pairing() -> [u8; 192] {
    let mut input = [0_u8; 192];
    for (index, word) in [
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
        "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2",
        "1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed",
        "090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b",
        "12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
    ]
    .into_iter()
    .enumerate()
    {
        let start = index.saturating_mul(32);
        if let Some(target) = input.get_mut(start..start.saturating_add(32)) {
            target.copy_from_slice(&hex32(word));
        }
    }
    input
}

pub(crate) fn dense_modexp(operand_bytes: usize) -> Vec<u8> {
    let mut input = Vec::from([0_u8; 96]);
    for offset in [0_usize, 32, 64] {
        let bytes = operand_bytes.to_be_bytes();
        let start = offset
            .checked_add(32)
            .and_then(|end| end.checked_sub(bytes.len()));
        if let Some(target) = start.and_then(|start| {
            start
                .checked_add(bytes.len())
                .and_then(|end| input.get_mut(start..end))
        }) {
            target.copy_from_slice(&bytes);
        }
    }
    input.extend(std::iter::repeat_n(0xa5_u8, operand_bytes));
    input.extend(std::iter::repeat_n(u8::MAX, operand_bytes));
    input.extend(std::iter::repeat_n(0xf3_u8, operand_bytes));
    input
}

pub(crate) fn high_round_blake2f(rounds: u32) -> [u8; 213] {
    let mut input = [0xa5_u8; 213];
    input[..4].copy_from_slice(&rounds.to_be_bytes());
    input[212] = 1;
    input
}

fn hex32(hex: &str) -> [u8; 32] {
    let mut output = [0_u8; 32];
    for (target, pair) in output.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        let [high, low] = pair else {
            continue;
        };
        let high = hex_nibble(*high);
        let low = hex_nibble(*low);
        *target = high.wrapping_shl(4) | low;
    }
    output
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte.wrapping_sub(b'0'),
        b'a'..=b'f' => byte.wrapping_sub(b'a').wrapping_add(10),
        b'A'..=b'F' => byte.wrapping_sub(b'A').wrapping_add(10),
        _ => 0,
    }
}
