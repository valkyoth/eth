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
/// This is still not a complete EIP-197 pairing result: the final
/// exponentiation is intentionally left to the next release. The result is used
/// only to keep the fail-closed pairing path executing the admitted algebra.
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
        acc = acc.square().mul(line_factor(doubled.line, tuple.g1));
        q = doubled.next;
        if ate_loop_bit(bit) {
            let added = g2_addition_line(q, tuple.g2);
            acc = acc.mul(line_factor(added.line, tuple.g1));
            q = added.next;
        }
    }
    acc
}

fn ate_loop_bit(bit: u32) -> bool {
    ((BN254_ATE_LOOP_COUNT >> bit) & 1) == 1
}

fn line_factor(line: G2LineCoefficients, g1: crate::bn254::G1Point) -> Fp12 {
    if line == G2LineCoefficients::ZERO {
        Fp12::ONE
    } else {
        line.evaluate_g1(g1)
    }
}
