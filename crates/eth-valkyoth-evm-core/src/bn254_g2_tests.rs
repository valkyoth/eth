use crate::{
    EVM_BN254_PAIRING_ITEM_BYTES, EvmCoreError,
    bn254_g2::{G2Point, read_g2_point},
};

#[test]
fn g2_frobenius_maps_generator_to_expected_q1() -> Result<(), EvmCoreError> {
    let q1 = g2_generator()?.frobenius();
    assert_g2_point(
        q1,
        "1e90992b13fc8e562f6839592f0f452b62d162aea782a401520fddb6b3cd6250",
        "0d6b98e29dca4786eaf4ea76d5e5883ed4169e25e55f247b9f5ac1e62410e140",
        "0957319384dacb13be8dc707070ab3d2f7279a2907b2f1aa942881b2d7c9d081",
        "211deca8c1d666d2bbf7862af1db06c460561e089558a2fbdfdd52b511756e53",
    );
    Ok(())
}

#[test]
fn g2_frobenius_p2_negated_maps_generator_to_expected_minus_q2() -> Result<(), EvmCoreError> {
    let minus_q2 = g2_generator()?.frobenius_p2_negated();
    assert_g2_point(
        minus_q2,
        "2f05d40e0a908579339a64b1e404b9ddc3ff696d01800a9069601c4e6c6942b9",
        "148e02f3766351d2edd3ec9bfeac8977e491187c53713c2e1ba2afaed69531bf",
        "12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
        "090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b",
    );
    Ok(())
}

#[test]
fn g2_frobenius_preserves_infinity() {
    assert_eq!(G2Point::INFINITY.frobenius(), G2Point::INFINITY);
    assert_eq!(G2Point::INFINITY.frobenius_p2_negated(), G2Point::INFINITY);
}

fn g2_generator() -> Result<G2Point, EvmCoreError> {
    let mut input = [0u8; EVM_BN254_PAIRING_ITEM_BYTES];
    write_word(
        &mut input,
        64,
        hex32("198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2"),
    );
    write_word(
        &mut input,
        96,
        hex32("1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed"),
    );
    write_word(
        &mut input,
        128,
        hex32("090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b"),
    );
    write_word(
        &mut input,
        160,
        hex32("12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa"),
    );
    read_g2_point(&input, 64)
}

fn assert_g2_point(
    point: G2Point,
    expected_x_real: &str,
    expected_x_imaginary: &str,
    expected_y_real: &str,
    expected_y_imaginary: &str,
) {
    assert!(!point.infinity);
    assert_eq!(point.x.c0.to_be_bytes(), hex32(expected_x_real));
    assert_eq!(point.x.c1.to_be_bytes(), hex32(expected_x_imaginary));
    assert_eq!(point.y.c0.to_be_bytes(), hex32(expected_y_real));
    assert_eq!(point.y.c1.to_be_bytes(), hex32(expected_y_imaginary));
}

fn write_word(output: &mut [u8], offset: usize, word: [u8; 32]) {
    if let Some(slot) = output.get_mut(offset..offset.saturating_add(32)) {
        slot.copy_from_slice(&word);
    }
}

fn hex32(input: &str) -> [u8; 32] {
    let mut output = [0u8; 32];
    for (index, chunk) in input.as_bytes().chunks_exact(2).enumerate() {
        if let (Some(slot), Some(high), Some(low)) =
            (output.get_mut(index), chunk.first(), chunk.get(1))
        {
            *slot = hex_nibble(*high).wrapping_shl(4) | hex_nibble(*low);
        }
    }
    output
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value.saturating_sub(b'0'),
        b'a'..=b'f' => 10u8.saturating_add(value.saturating_sub(b'a')),
        b'A'..=b'F' => 10u8.saturating_add(value.saturating_sub(b'A')),
        _ => 0,
    }
}
