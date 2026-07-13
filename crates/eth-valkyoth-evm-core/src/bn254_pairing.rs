use crate::{
    EVM_PRECOMPILE_INPUT_LIMIT, EvmCoreError, EvmGasMeter, EvmPrecompileKind, EvmPrecompilePlan,
    bn254::{G1Point, read_g1_point},
    bn254_final::final_exponentiation,
    bn254_g2::{G2Point, read_g2_point},
    bn254_miller::exercise_miller_loop_accumulation,
};

/// Byte length of one EIP-197 BN254 pairing tuple.
pub const EVM_BN254_PAIRING_ITEM_BYTES: usize = 192;
/// Byte length of the BN254 pairing precompile output word.
pub const EVM_BN254_PAIRING_OUTPUT_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Bn254PairingTuple {
    pub(crate) g1: G1Point,
    pub(crate) g2: G2Point,
}

/// Validates the EIP-197 BN254 pairing input frame and returns its tuple count.
///
/// This is an unmetered low-level parser for standalone validation and fuzzing.
/// Interpreter integrations must prefer
/// [`EvmPrecompilePlan::execute_bn254_pairing`], which charges the supplied gas
/// meter before tuple validation and subgroup checks are reachable.
///
/// This parser validates tuple segmentation, G1 points, G2 field elements, G2
/// curve membership, and G2 subgroup membership. It does not execute the
/// pairing; callers that need the EIP-197 result must execute a charged
/// [`EvmPrecompilePlan`].
pub fn parse_bn254_pairing_input(input: &[u8]) -> Result<usize, EvmCoreError> {
    let mut pairs = 0usize;
    for_each_valid_pairing_tuple(input, |_| {
        pairs = pairs.saturating_add(1);
    })?;
    Ok(pairs)
}

pub(crate) fn for_each_valid_pairing_tuple(
    input: &[u8],
    mut visit: impl FnMut(Bn254PairingTuple),
) -> Result<usize, EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    if !input.len().is_multiple_of(EVM_BN254_PAIRING_ITEM_BYTES) {
        return Err(EvmCoreError::PrecompileInvalidInputLength);
    }
    let mut offset = 0usize;
    let mut pairs = 0usize;
    while offset < input.len() {
        let g1 = read_g1_point(input, offset)?;
        let g2 = read_g2_point(input, offset.saturating_add(64))?;
        visit(Bn254PairingTuple { g1, g2 });
        offset = offset.saturating_add(EVM_BN254_PAIRING_ITEM_BYTES);
        pairs = pairs.saturating_add(1);
    }
    Ok(pairs)
}

pub(crate) fn execute_bn254_pairing(
    input: &[u8],
    output: &mut [u8],
) -> Result<usize, EvmCoreError> {
    let target = output
        .get_mut(..EVM_BN254_PAIRING_OUTPUT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    let (pairs, miller) = exercise_miller_loop_accumulation(input)?;
    let result = if pairs == 0 || final_exponentiation(miller) == crate::bn254_tower::Fp12::ONE {
        1
    } else {
        0
    };
    target.fill(0);
    if let Some(last) = target.last_mut() {
        *last = result;
    }
    Ok(EVM_BN254_PAIRING_OUTPUT_BYTES)
}

impl EvmPrecompilePlan {
    /// Executes the EIP-197 BN254 pairing frame into `output`.
    ///
    /// This method charges this exact plan's precompile gas before the pairing
    /// parser and subgroup checks are reachable.
    pub fn execute_bn254_pairing(
        self,
        gas_meter: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Bn254Pairing {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        let gas_cost = self.checked_execution_cost(input)?;
        gas_meter.charge(gas_cost)?;
        execute_bn254_pairing(input, output)
    }
}
