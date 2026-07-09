use crate::{
    EvmCoreError,
    bn254_line::{G2LineCoefficients, g2_addition_line, g2_doubling_line},
    bn254_pairing::{Bn254PairingTuple, for_each_valid_pairing_tuple},
    bn254_tower::Fp12,
};

// Ethereum's BN254 precompile uses the positive optimal ate-loop count for
// alt_bn128 pairing arithmetic.
const BN254_ATE_LOOP_COUNT: u128 = 0x0001_9d79_7039_be76_3ba8;
const BN254_ATE_LOOP_TOP_BIT: u32 = 64;

/// Exercises first-party Miller-loop accumulation over validated BN254 tuples.
///
/// This is still not a public EIP-197 pairing result. The fail-closed pairing
/// path consumes this bounded accumulator and then exercises final
/// exponentiation without admitting non-empty success until the post-loop line
/// carrier and result-admission releases are reviewed.
pub(crate) fn exercise_miller_loop_accumulation(
    input: &[u8],
) -> Result<(usize, Fp12), EvmCoreError> {
    let mut acc = Fp12::ONE;
    let pairs = for_each_valid_pairing_tuple(input, |tuple| {
        acc = acc.mul(miller_loop_tuple(tuple));
    })?;
    Ok((pairs, acc))
}

/// Runs the internal BN254 Miller accumulator for fuzz and test harnesses.
#[cfg(feature = "testing")]
pub fn testing_bn254_miller_pairs(input: &[u8]) -> Result<(usize, bool), EvmCoreError> {
    let (pairs, acc) = exercise_miller_loop_accumulation(input)?;
    Ok((pairs, acc == Fp12::ONE))
}

pub(crate) fn miller_loop_tuple(tuple: Bn254PairingTuple) -> Fp12 {
    if tuple.g1.infinity || tuple.g2.infinity {
        return Fp12::ONE;
    }
    let mut acc = Fp12::ONE;
    let mut q = tuple.g2;
    for bit in (0..BN254_ATE_LOOP_TOP_BIT).rev() {
        let doubled = g2_doubling_line(q);
        acc = multiply_by_line_factor(acc.square(), doubled.line, tuple.g1);
        q = doubled.next;
        if ate_loop_bit(bit) {
            let added = g2_addition_line(q, tuple.g2);
            acc = multiply_by_line_factor(acc, added.line, tuple.g1);
            q = added.next;
        }
    }
    core::hint::black_box(tuple.g2.optimal_ate_post_loop_points());
    acc
}

fn ate_loop_bit(bit: u32) -> bool {
    ((BN254_ATE_LOOP_COUNT >> bit) & 1) == 1
}

fn multiply_by_line_factor(acc: Fp12, line: G2LineCoefficients, g1: crate::bn254::G1Point) -> Fp12 {
    if line == G2LineCoefficients::ZERO {
        acc
    } else {
        acc.mul_by_fp6(line.evaluate_g1_fp6(g1))
    }
}
