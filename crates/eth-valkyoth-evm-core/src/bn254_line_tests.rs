use crate::{
    EvmCoreError,
    bn254::read_g1_point,
    bn254_g2::{Fp2, G2Point, read_g2_point},
    bn254_line::{evaluate_line_foundation_at_g1, g2_addition_line, g2_doubling_line},
    bn254_tower::{Fp6, Fp12},
};

#[test]
fn g2_doubling_line_vanishes_at_tangent_points() -> Result<(), EvmCoreError> {
    let point = g2_generator()?;
    let step = g2_doubling_line(point);
    assert!(!step.next.infinity);
    assert_eq!(step.line.evaluate_g2(point), Fp2::ZERO);
    assert_eq!(step.line.evaluate_g2(neg_g2(step.next)), Fp2::ZERO);
    Ok(())
}

#[test]
fn g2_addition_line_vanishes_at_input_points() -> Result<(), EvmCoreError> {
    let point = g2_generator()?;
    let doubled = g2_doubling_line(point).next;
    let step = g2_addition_line(point, doubled);
    assert!(!step.next.infinity);
    assert_eq!(step.line.evaluate_g2(point), Fp2::ZERO);
    assert_eq!(step.line.evaluate_g2(doubled), Fp2::ZERO);
    assert_eq!(step.line.evaluate_g2(neg_g2(step.next)), Fp2::ZERO);
    Ok(())
}

#[test]
fn g2_addition_line_handles_vertical_inverse_case() -> Result<(), EvmCoreError> {
    let point = g2_generator()?;
    let inverse = neg_g2(point);
    let step = g2_addition_line(point, inverse);
    assert!(step.next.infinity);
    assert_eq!(step.line.evaluate_g2(point), Fp2::ZERO);
    assert_eq!(step.line.evaluate_g2(inverse), Fp2::ZERO);
    Ok(())
}

#[test]
fn g1_line_evaluation_uses_fp12_carrier_without_claiming_pairing() -> Result<(), EvmCoreError> {
    let g1 = read_g1_point(&g1_generator_input(), 0)?;
    let g2 = g2_generator()?;
    let carrier = evaluate_line_foundation_at_g1(g1, g2);
    assert_ne!(carrier, Fp12::ONE);
    assert_eq!(carrier.c1, Fp6::ZERO);
    Ok(())
}

fn g2_generator() -> Result<G2Point, EvmCoreError> {
    read_g2_point(&g2_generator_input(), 0)
}

fn neg_g2(point: G2Point) -> G2Point {
    if point.infinity {
        point
    } else {
        G2Point {
            x: point.x,
            y: point.y.neg(),
            infinity: false,
        }
    }
}

fn g1_generator_input() -> [u8; 64] {
    let mut output = [0u8; 64];
    write_word(
        &mut output,
        0,
        hex32("0000000000000000000000000000000000000000000000000000000000000001"),
    );
    write_word(
        &mut output,
        32,
        hex32("0000000000000000000000000000000000000000000000000000000000000002"),
    );
    output
}

fn g2_generator_input() -> [u8; 128] {
    let mut output = [0u8; 128];
    write_word(
        &mut output,
        0,
        hex32("198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2"),
    );
    write_word(
        &mut output,
        32,
        hex32("1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed"),
    );
    write_word(
        &mut output,
        64,
        hex32("090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b"),
    );
    write_word(
        &mut output,
        96,
        hex32("12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa"),
    );
    output
}

fn write_word<const N: usize>(target: &mut [u8; N], offset: usize, word: [u8; 32]) {
    if let Some(slot) = target.get_mut(offset..offset.saturating_add(32)) {
        slot.copy_from_slice(&word);
    }
}

fn hex32(input: &str) -> [u8; 32] {
    let mut output = [0u8; 32];
    for (index, chunk) in input.as_bytes().chunks_exact(2).enumerate() {
        if let Some(slot) = output.get_mut(index) {
            let high = chunk.first().copied().map(hex_nibble).unwrap_or(0);
            let low = chunk.get(1).copied().map(hex_nibble).unwrap_or(0);
            *slot = high.wrapping_shl(4) | low;
        }
    }
    output
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte.saturating_sub(b'0'),
        b'a'..=b'f' => byte.saturating_sub(b'a').saturating_add(10),
        b'A'..=b'F' => byte.saturating_sub(b'A').saturating_add(10),
        _ => 0,
    }
}
