extern crate std;

use crate::{
    EVM_BN254_PAIRING_ITEM_BYTES, EvmCoreError,
    bn254::execute_bn254_add,
    bn254_final::final_exponentiation,
    bn254_line::{g2_addition_line, g2_doubling_line},
    bn254_miller::{BN254_SIX_U_PLUS_2_NAF, exercise_miller_loop_accumulation, miller_loop_tuple},
    bn254_pairing::for_each_valid_pairing_tuple,
    bn254_tower::Fp12,
};

#[test]
fn miller_loop_empty_input_is_neutral() -> Result<(), EvmCoreError> {
    let (pairs, acc) = exercise_miller_loop_accumulation(&[])?;
    assert_eq!(pairs, 0);
    assert_eq!(acc, Fp12::ONE);
    Ok(())
}

#[test]
fn miller_loop_generator_tuple_is_deterministic() -> Result<(), EvmCoreError> {
    let input = generator_pairing_tuple();
    let (pairs, first) = exercise_miller_loop_accumulation(&input)?;
    let (_, second) = exercise_miller_loop_accumulation(&input)?;
    assert_eq!(pairs, 1);
    assert_eq!(first, second);
    assert_ne!(first, Fp12::ONE);
    assert_ne!(final_exponentiation(first), Fp12::ONE);
    Ok(())
}

#[test]
fn miller_loop_batch_accumulates_tuple_products() -> Result<(), EvmCoreError> {
    let one = generator_pairing_tuple();
    let mut two = [0u8; EVM_BN254_PAIRING_ITEM_BYTES * 2];
    if let Some(first) = two.get_mut(..EVM_BN254_PAIRING_ITEM_BYTES) {
        first.copy_from_slice(&one);
    }
    if let Some(second) = two.get_mut(EVM_BN254_PAIRING_ITEM_BYTES..) {
        second.copy_from_slice(&one);
    }

    let (_, single) = exercise_miller_loop_accumulation(&one)?;
    let (pairs, batched) = exercise_miller_loop_accumulation(&two)?;
    assert_eq!(pairs, 2);
    assert_eq!(batched, single.mul(single));
    Ok(())
}

#[test]
fn miller_loop_treats_infinity_tuple_as_neutral() -> Result<(), EvmCoreError> {
    let input = g1_infinity_tuple();
    let (pairs, acc) = exercise_miller_loop_accumulation(&input)?;
    assert_eq!(pairs, 1);
    assert_eq!(acc, Fp12::ONE);

    let input = g2_infinity_tuple();
    let (pairs, acc) = exercise_miller_loop_accumulation(&input)?;
    assert_eq!(pairs, 1);
    assert_eq!(acc, Fp12::ONE);
    Ok(())
}

#[test]
fn complete_accumulator_keeps_inverse_batch_neutral() -> Result<(), EvmCoreError> {
    let input = generator_and_negated_generator_pairing_tuples();
    let (pairs, acc) = exercise_miller_loop_accumulation(&input)?;
    assert_eq!(pairs, 2);
    assert_eq!(final_exponentiation(acc), Fp12::ONE);
    Ok(())
}

#[test]
fn miller_loop_naf_reconstructs_six_u_plus_two() {
    let scalar = BN254_SIX_U_PLUS_2_NAF
        .iter()
        .enumerate()
        .fold(0i128, |acc, (index, digit)| {
            acc + i128::from(*digit) * (1i128 << index)
        });
    assert_eq!(scalar, 6 * 4_965_661_367_192_848_881i128 + 2);
}

#[test]
fn miller_loop_is_bilinear_over_g1_double() -> Result<(), EvmCoreError> {
    let generator = generator_pairing_tuple();
    let doubled = doubled_g1_pairing_tuple()?;
    let (_, generator_acc) = exercise_miller_loop_accumulation(&generator)?;
    let (_, doubled_acc) = exercise_miller_loop_accumulation(&doubled)?;

    assert_eq!(
        final_exponentiation(doubled_acc),
        final_exponentiation(generator_acc).square()
    );
    Ok(())
}

#[test]
fn miller_loop_tuple_matches_stream_accumulator() -> Result<(), EvmCoreError> {
    let input = generator_pairing_tuple();
    let mut tuple_acc = Fp12::ONE;
    let seen = for_each_valid_pairing_tuple(&input, |tuple| {
        tuple_acc = tuple_acc.mul(miller_loop_tuple(tuple));
    })?;
    let (pairs, stream_acc) = exercise_miller_loop_accumulation(&input)?;
    assert_eq!(seen, pairs);
    assert_eq!(tuple_acc, stream_acc);
    Ok(())
}

#[test]
fn sparse_line_factor_multiplication_matches_dense_carrier() -> Result<(), EvmCoreError> {
    let input = generator_pairing_tuple();
    let mut tuple = None;
    let seen = for_each_valid_pairing_tuple(&input, |item| {
        tuple = Some(item);
    })?;
    assert_eq!(seen, 1);

    if let Some(item) = tuple {
        let doubled = g2_doubling_line(item.g2);
        assert_eq!(
            Fp12::ONE.mul_by_fp6(doubled.line.evaluate_g1_fp6(item.g1)),
            Fp12::ONE.mul(doubled.line.evaluate_g1(item.g1))
        );

        let added = g2_addition_line(doubled.next, item.g2);
        assert_eq!(
            Fp12::ONE.mul_by_fp6(added.line.evaluate_g1_fp6(item.g1)),
            Fp12::ONE.mul(added.line.evaluate_g1(item.g1))
        );
    }
    Ok(())
}

#[test]
#[ignore = "release evidence benchmark; run explicitly for v0.50.6"]
fn miller_loop_wall_time_budget_smoke() -> Result<(), EvmCoreError> {
    let input = generator_pairing_tuple();
    let mut tuple = None;
    let seen = for_each_valid_pairing_tuple(&input, |item| {
        tuple = Some(item);
    })?;
    assert_eq!(seen, 1);

    if let Some(item) = tuple {
        let iterations = 3u32;
        let start = std::time::Instant::now();
        let mut acc = Fp12::ONE;
        for _ in 0..iterations {
            acc = std::hint::black_box(miller_loop_tuple(std::hint::black_box(item)));
        }
        std::hint::black_box(acc);
        let elapsed = start.elapsed();
        let average = elapsed
            .as_nanos()
            .checked_div(u128::from(iterations))
            .unwrap_or(0);
        std::println!(
            "bn254_miller_loop_tuple iterations={} total_ns={} average_ns={}",
            iterations,
            elapsed.as_nanos(),
            average
        );
        assert!(elapsed > std::time::Duration::ZERO);
    }
    Ok(())
}

fn generator_pairing_tuple() -> [u8; EVM_BN254_PAIRING_ITEM_BYTES] {
    let mut output = [0u8; EVM_BN254_PAIRING_ITEM_BYTES];
    write_g1_generator(&mut output);
    write_g2_generator(&mut output);
    output
}

fn doubled_g1_pairing_tuple() -> Result<[u8; EVM_BN254_PAIRING_ITEM_BYTES], EvmCoreError> {
    let mut add_input = [0u8; 128];
    let generator = generator_pairing_tuple();
    if let Some(first) = add_input.get_mut(..64) {
        first.copy_from_slice(&generator[..64]);
    }
    if let Some(second) = add_input.get_mut(64..) {
        second.copy_from_slice(&generator[..64]);
    }

    let mut doubled = [0u8; 64];
    assert_eq!(execute_bn254_add(&add_input, &mut doubled)?, 64);

    let mut output = [0u8; EVM_BN254_PAIRING_ITEM_BYTES];
    if let Some(g1) = output.get_mut(..64) {
        g1.copy_from_slice(&doubled);
    }
    write_g2_generator(&mut output);
    Ok(output)
}

fn g1_infinity_tuple() -> [u8; EVM_BN254_PAIRING_ITEM_BYTES] {
    let mut output = [0u8; EVM_BN254_PAIRING_ITEM_BYTES];
    write_g2_generator(&mut output);
    output
}

fn g2_infinity_tuple() -> [u8; EVM_BN254_PAIRING_ITEM_BYTES] {
    let mut output = [0u8; EVM_BN254_PAIRING_ITEM_BYTES];
    write_g1_generator(&mut output);
    output
}

fn generator_and_negated_generator_pairing_tuples() -> [u8; EVM_BN254_PAIRING_ITEM_BYTES * 2] {
    let generator = generator_pairing_tuple();
    let mut output = [0u8; EVM_BN254_PAIRING_ITEM_BYTES * 2];
    if let Some(first) = output.get_mut(..EVM_BN254_PAIRING_ITEM_BYTES) {
        first.copy_from_slice(&generator);
    }
    if let Some(second) = output.get_mut(EVM_BN254_PAIRING_ITEM_BYTES..) {
        second.copy_from_slice(&generator);
    }
    write_word(
        &mut output,
        EVM_BN254_PAIRING_ITEM_BYTES + 32,
        hex32("30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd45"),
    );
    output
}

fn write_g1_generator(output: &mut [u8; EVM_BN254_PAIRING_ITEM_BYTES]) {
    write_word(
        output,
        0,
        hex32("0000000000000000000000000000000000000000000000000000000000000001"),
    );
    write_word(
        output,
        32,
        hex32("0000000000000000000000000000000000000000000000000000000000000002"),
    );
}

fn write_g2_generator(output: &mut [u8; EVM_BN254_PAIRING_ITEM_BYTES]) {
    write_word(
        output,
        64,
        hex32("198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2"),
    );
    write_word(
        output,
        96,
        hex32("1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed"),
    );
    write_word(
        output,
        128,
        hex32("090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b"),
    );
    write_word(
        output,
        160,
        hex32("12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa"),
    );
}

fn write_word<const N: usize>(target: &mut [u8; N], offset: usize, word: [u8; 32]) {
    if let Some(output) = target.get_mut(offset..offset.saturating_add(32)) {
        output.copy_from_slice(&word);
    }
}

fn hex32(hex: &str) -> [u8; 32] {
    let mut output = [0u8; 32];
    for (target, pair) in output.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        let high = pair.first().copied().map(hex_nibble).unwrap_or(0);
        let low = pair.get(1).copied().map(hex_nibble).unwrap_or(0);
        *target = (high << 4) | low;
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
