use crate::{
    EvmCoreError,
    bn254_g2::G2Point,
    bn254_line::ProjectiveG2LineState,
    bn254_pairing::{Bn254PairingTuple, for_each_valid_pairing_tuple},
    bn254_tower::Fp12,
};

pub(crate) const BN254_SIX_U_PLUS_2_NAF: &[i8] = &[
    0, 0, 0, 1, 0, 1, 0, -1, 0, 0, -1, 0, 0, 0, 1, 0, 0, -1, 0, -1, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0,
    -1, 0, 0, 1, 0, -1, 0, 0, 1, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 1, 0, -1, 0, 0, 0, -1, 0, -1, 0,
    0, 0, 1, 0, -1, 0, 1,
];

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

/// Exercises the admitted BN254 post-loop Frobenius point helper for fuzzing.
#[cfg(feature = "testing")]
pub fn testing_bn254_post_loop_point_pairs(input: &[u8]) -> Result<(usize, usize), EvmCoreError> {
    let mut non_infinity = 0usize;
    let pairs = for_each_valid_pairing_tuple(input, |tuple| {
        let (q1, minus_q2) = tuple.g2.optimal_ate_post_loop_points();
        if !tuple.g2.infinity && !q1.infinity && !minus_q2.infinity {
            non_infinity = non_infinity.saturating_add(1);
        }
    })?;
    Ok((pairs, non_infinity))
}

/// Exercises the completed fail-closed BN254 accumulator for fuzzing.
#[cfg(feature = "testing")]
pub fn testing_bn254_complete_accumulator_pairs(
    input: &[u8],
) -> Result<(usize, bool), EvmCoreError> {
    let (pairs, acc) = exercise_miller_loop_accumulation(input)?;
    Ok((pairs, acc == Fp12::ONE))
}

pub(crate) fn miller_loop_tuple(tuple: Bn254PairingTuple) -> Fp12 {
    if tuple.g1.infinity || tuple.g2.infinity {
        return Fp12::ONE;
    }
    let mut acc = Fp12::ONE;
    let mut q = ProjectiveG2LineState::from_affine(tuple.g2);
    let minus_q = neg_g2(tuple.g2);
    let y2 = tuple.g2.y.square();

    for index in (1..BN254_SIX_U_PLUS_2_NAF.len()).rev() {
        let doubled = q.doubling_line(tuple.g1);
        if index != BN254_SIX_U_PLUS_2_NAF.len().saturating_sub(1) {
            acc = acc.square();
        }
        acc = doubled.line.multiply_accumulator(acc);
        q = doubled.next;

        let Some(next_digit) = BN254_SIX_U_PLUS_2_NAF.get(index.saturating_sub(1)).copied() else {
            continue;
        };
        let added = match next_digit {
            1 => Some(q.addition_line(tuple.g2, y2, tuple.g1)),
            -1 => Some(q.addition_line(minus_q, y2, tuple.g1)),
            _ => None,
        };
        if let Some(step) = added {
            acc = step.line.multiply_accumulator(acc);
            q = step.next;
        }
    }

    let (q1, minus_q2) = tuple.g2.optimal_ate_post_loop_points();
    let q1_added = q.addition_line(q1, q1.y.square(), tuple.g1);
    acc = q1_added.line.multiply_accumulator(acc);
    let q2_added = q1_added
        .next
        .addition_line(minus_q2, minus_q2.y.square(), tuple.g1);
    acc = q2_added.line.multiply_accumulator(acc);
    acc
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
