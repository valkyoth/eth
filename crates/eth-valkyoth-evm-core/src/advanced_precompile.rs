use crate::{
    EVM_BLS12381_FP_BYTES, EVM_BLS12381_FP2_BYTES, EVM_BLS12381_G1_MSM_ITEM_BYTES,
    EVM_BLS12381_G1_POINT_BYTES, EVM_BLS12381_G2_MSM_ITEM_BYTES, EVM_BLS12381_G2_POINT_BYTES,
    EVM_BLS12381_PAIRING_ITEM_BYTES, EvmCoreError, EvmGas, EvmPrecompileGasPolicy,
    EvmPrecompileInputPolicy, EvmPrecompileKind, bls12_gas,
};

const KZG_INPUT_BYTES: usize = 192;
const KZG_OUTPUT_BYTES: usize = 64;
const BLS_G1_ADD_INPUT_BYTES: usize = EVM_BLS12381_G1_POINT_BYTES * 2;
const BLS_G2_ADD_INPUT_BYTES: usize = EVM_BLS12381_G2_POINT_BYTES * 2;
const BLS_PAIRING_OUTPUT_BYTES: usize = 32;

pub(crate) const fn input_policy(kind: EvmPrecompileKind) -> EvmPrecompileInputPolicy {
    match kind {
        EvmPrecompileKind::KzgPointEvaluation => EvmPrecompileInputPolicy::Exact(KZG_INPUT_BYTES),
        EvmPrecompileKind::Bls12G1Add => EvmPrecompileInputPolicy::Exact(BLS_G1_ADD_INPUT_BYTES),
        EvmPrecompileKind::Bls12G1Msm => {
            EvmPrecompileInputPolicy::NonEmptyMultipleOf(EVM_BLS12381_G1_MSM_ITEM_BYTES)
        }
        EvmPrecompileKind::Bls12G2Add => EvmPrecompileInputPolicy::Exact(BLS_G2_ADD_INPUT_BYTES),
        EvmPrecompileKind::Bls12G2Msm => {
            EvmPrecompileInputPolicy::NonEmptyMultipleOf(EVM_BLS12381_G2_MSM_ITEM_BYTES)
        }
        EvmPrecompileKind::Bls12PairingCheck => {
            EvmPrecompileInputPolicy::NonEmptyMultipleOf(EVM_BLS12381_PAIRING_ITEM_BYTES)
        }
        EvmPrecompileKind::Bls12MapFpToG1 => EvmPrecompileInputPolicy::Exact(EVM_BLS12381_FP_BYTES),
        EvmPrecompileKind::Bls12MapFp2ToG2 => {
            EvmPrecompileInputPolicy::Exact(EVM_BLS12381_FP2_BYTES)
        }
        _ => EvmPrecompileInputPolicy::BoundedAny,
    }
}

pub(crate) const fn gas_policy(kind: EvmPrecompileKind) -> EvmPrecompileGasPolicy {
    match kind {
        EvmPrecompileKind::KzgPointEvaluation => EvmPrecompileGasPolicy::Fixed(EvmGas::new(50_000)),
        EvmPrecompileKind::Bls12G1Add => EvmPrecompileGasPolicy::Fixed(EvmGas::new(375)),
        EvmPrecompileKind::Bls12G1Msm => EvmPrecompileGasPolicy::Bls12G1Msm,
        EvmPrecompileKind::Bls12G2Add => EvmPrecompileGasPolicy::Fixed(EvmGas::new(600)),
        EvmPrecompileKind::Bls12G2Msm => EvmPrecompileGasPolicy::Bls12G2Msm,
        EvmPrecompileKind::Bls12PairingCheck => EvmPrecompileGasPolicy::Bls12Pairing,
        EvmPrecompileKind::Bls12MapFpToG1 => EvmPrecompileGasPolicy::Fixed(EvmGas::new(5_500)),
        EvmPrecompileKind::Bls12MapFp2ToG2 => EvmPrecompileGasPolicy::Fixed(EvmGas::new(23_800)),
        _ => EvmPrecompileGasPolicy::DeferredDynamic,
    }
}

pub(crate) const fn output_len(kind: EvmPrecompileKind) -> usize {
    match kind {
        EvmPrecompileKind::KzgPointEvaluation => KZG_OUTPUT_BYTES,
        EvmPrecompileKind::Bls12G1Add
        | EvmPrecompileKind::Bls12G1Msm
        | EvmPrecompileKind::Bls12MapFpToG1 => EVM_BLS12381_G1_POINT_BYTES,
        EvmPrecompileKind::Bls12G2Add
        | EvmPrecompileKind::Bls12G2Msm
        | EvmPrecompileKind::Bls12MapFp2ToG2 => EVM_BLS12381_G2_POINT_BYTES,
        EvmPrecompileKind::Bls12PairingCheck => BLS_PAIRING_OUTPUT_BYTES,
        _ => 0,
    }
}

pub(crate) fn dynamic_gas(
    policy: EvmPrecompileGasPolicy,
    input_len: usize,
) -> Result<EvmGas, EvmCoreError> {
    match policy {
        EvmPrecompileGasPolicy::Bls12G1Msm => bls12_gas::g1_msm(input_len),
        EvmPrecompileGasPolicy::Bls12G2Msm => bls12_gas::g2_msm(input_len),
        EvmPrecompileGasPolicy::Bls12Pairing => bls12_gas::pairing(input_len),
        _ => Err(EvmCoreError::PrecompileBackendUnavailable),
    }
}
