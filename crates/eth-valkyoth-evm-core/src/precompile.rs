use crate::{
    EvmAddress, EvmCoreError, EvmFork, EvmGas, advanced_precompile, hash_precompile, precompile_gas,
};

/// Maximum precompile calldata bytes admitted by the native planning boundary.
pub const EVM_PRECOMPILE_INPUT_LIMIT: usize = 1_048_576;
pub(crate) const WORD_BYTES: usize = 32;
pub(crate) const PAIRING_ITEM_BYTES: usize = 192;
const BLAKE2F_INPUT_BYTES: usize = 213;
const BLAKE2F_OUTPUT_BYTES: usize = 64;
const BN254_POINT_OUTPUT_BYTES: usize = 64;

/// Precompile identity known to the native EVM core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmPrecompileKind {
    /// `ecrecover` at address `0x01`.
    EcRecover,
    /// SHA-256 at address `0x02`.
    Sha256,
    /// RIPEMD-160 at address `0x03`.
    Ripemd160,
    /// Identity at address `0x04`.
    Identity,
    /// Big integer modular exponentiation at address `0x05`.
    Modexp,
    /// BN254 point addition at address `0x06`.
    Bn254Add,
    /// BN254 scalar multiplication at address `0x07`.
    Bn254Mul,
    /// BN254 pairing check at address `0x08`.
    Bn254Pairing,
    /// BLAKE2 compression function at address `0x09`.
    Blake2F,
    /// KZG point evaluation at address `0x0a`.
    KzgPointEvaluation,
    /// BLS12-381 G1 addition at address `0x0b`.
    Bls12G1Add,
    /// BLS12-381 G1 multiscalar multiplication at address `0x0c`.
    Bls12G1Msm,
    /// BLS12-381 G2 addition at address `0x0d`.
    Bls12G2Add,
    /// BLS12-381 G2 multiscalar multiplication at address `0x0e`.
    Bls12G2Msm,
    /// BLS12-381 pairing check at address `0x0f`.
    Bls12PairingCheck,
    /// BLS12-381 field-to-G1 map at address `0x10`.
    Bls12MapFpToG1,
    /// BLS12-381 field-to-G2 map at address `0x11`.
    Bls12MapFp2ToG2,
}

impl EvmPrecompileKind {
    /// Returns the canonical precompile account address.
    #[must_use]
    pub const fn address(self) -> EvmAddress {
        precompile_address(self.low_u16())
    }

    /// Returns the first fork where this precompile is admitted.
    #[must_use]
    pub const fn introduced_in(self) -> EvmFork {
        match self {
            Self::EcRecover | Self::Sha256 | Self::Ripemd160 | Self::Identity => EvmFork::FRONTIER,
            Self::Modexp | Self::Bn254Add | Self::Bn254Mul | Self::Bn254Pairing => {
                EvmFork::BYZANTIUM
            }
            Self::Blake2F => EvmFork::ISTANBUL,
            Self::KzgPointEvaluation => EvmFork::CANCUN,
            Self::Bls12G1Add
            | Self::Bls12G1Msm
            | Self::Bls12G2Add
            | Self::Bls12G2Msm
            | Self::Bls12PairingCheck
            | Self::Bls12MapFpToG1
            | Self::Bls12MapFp2ToG2 => EvmFork::PRAGUE,
        }
    }

    const fn low_u16(self) -> u16 {
        match self {
            Self::EcRecover => 0x01,
            Self::Sha256 => 0x02,
            Self::Ripemd160 => 0x03,
            Self::Identity => 0x04,
            Self::Modexp => 0x05,
            Self::Bn254Add => 0x06,
            Self::Bn254Mul => 0x07,
            Self::Bn254Pairing => 0x08,
            Self::Blake2F => 0x09,
            Self::KzgPointEvaluation => 0x0a,
            Self::Bls12G1Add => 0x0b,
            Self::Bls12G1Msm => 0x0c,
            Self::Bls12G2Add => 0x0d,
            Self::Bls12G2Msm => 0x0e,
            Self::Bls12PairingCheck => 0x0f,
            Self::Bls12MapFpToG1 => 0x10,
            Self::Bls12MapFp2ToG2 => 0x11,
        }
    }
}

/// Implementation boundary for a precompile descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmPrecompileImplementation {
    /// The release can execute ECRECOVER with caller-provided backends.
    NativeEcRecover,
    /// The release can execute this precompile without third-party crypto.
    NativeIdentity,
    /// The release can execute SHA-256 dependency-free.
    NativeSha256,
    /// The release can execute RIPEMD-160 dependency-free.
    NativeRipemd160,
    /// The release can execute bounded dependency-free ModExp.
    NativeModexp,
    /// The release can execute dependency-free BN254 point addition.
    NativeBn254Add,
    /// The release can execute dependency-free BN254 scalar multiplication.
    NativeBn254Mul,
    /// The release can execute dependency-free BN254 pairing frames.
    NativeBn254PairingFrame,
    /// The release can execute dependency-free BLAKE2F.
    NativeBlake2F,
    /// The release admits planning only; execution must fail closed.
    RequiresCryptoBackend,
}

/// Input shape admitted by the precompile planning boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmPrecompileInputPolicy {
    /// Any input length up to [`EVM_PRECOMPILE_INPUT_LIMIT`] is accepted.
    BoundedAny,
    /// The input must have exactly this byte length.
    Exact(usize),
    /// The input length must be a multiple of this byte length.
    MultipleOf(usize),
    /// The input must contain at least one complete item of this byte length.
    NonEmptyMultipleOf(usize),
}

/// Gas policy used by a precompile descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmPrecompileGasPolicy {
    /// Fixed gas.
    Fixed(EvmGas),
    /// Linear word gas.
    Words {
        /// Constant base gas charged once.
        base: EvmGas,
        /// Gas charged per 32-byte input word, rounded up.
        per_word: EvmGas,
    },
    /// BN254 pairing gas, with Byzantium and Istanbul pricing split by fork.
    Bn254Pairing,
    /// ModExp gas, with Byzantium and Berlin pricing split by fork.
    Modexp,
    /// Gas is the BLAKE2F round count stored in the first four input bytes.
    Blake2FRounds,
    /// EIP-2537 BLS12-381 G1 multiscalar-multiplication gas.
    Bls12G1Msm,
    /// EIP-2537 BLS12-381 G2 multiscalar-multiplication gas.
    Bls12G2Msm,
    /// EIP-2537 BLS12-381 pairing-check gas.
    Bls12Pairing,
    /// Dynamic gas formula is intentionally not implemented in this release.
    DeferredDynamic,
}

/// Fork-aware precompile descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmPrecompileDescriptor {
    /// Precompile identity.
    pub kind: EvmPrecompileKind,
    /// Canonical precompile account address.
    pub address: EvmAddress,
    /// Registry fork that produced this descriptor.
    pub fork: EvmFork,
    /// First fork where this descriptor is admitted.
    pub introduced_in: EvmFork,
    /// Execution backend boundary.
    pub implementation: EvmPrecompileImplementation,
    /// Input length policy.
    pub input_policy: EvmPrecompileInputPolicy,
    /// Gas policy.
    pub gas_policy: EvmPrecompileGasPolicy,
    /// Fixed output length when known.
    pub output_len: Option<usize>,
}

/// Validated plan for one precompile call.
///
/// Execution methods recompute gas from their actual input and reject a
/// content-dependent cost that differs from this plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmPrecompilePlan {
    descriptor: EvmPrecompileDescriptor,
    input_len: usize,
    gas_cost: Option<EvmGas>,
}

impl EvmPrecompilePlan {
    /// Builds a bounded plan for a precompile input.
    pub fn try_new(
        descriptor: EvmPrecompileDescriptor,
        input: &[u8],
    ) -> Result<Self, EvmCoreError> {
        validate_input_policy(descriptor.input_policy, input.len())?;
        let gas_cost = precompile_gas::gas_cost(descriptor, input)?;
        Ok(Self {
            descriptor,
            input_len: input.len(),
            gas_cost,
        })
    }

    /// Returns the descriptor used to create the plan.
    #[must_use]
    pub const fn descriptor(self) -> EvmPrecompileDescriptor {
        self.descriptor
    }

    /// Returns the planned input length.
    #[must_use]
    pub const fn input_len(self) -> usize {
        self.input_len
    }

    /// Returns the gas cost when this release implements the gas formula.
    #[must_use]
    pub const fn gas_cost(self) -> Option<EvmGas> {
        self.gas_cost
    }

    /// Revalidates that execution input has the cost recorded by this plan.
    pub(crate) fn checked_execution_cost(self, input: &[u8]) -> Result<EvmGas, EvmCoreError> {
        if input.len() != self.input_len {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        validate_input_policy(self.descriptor.input_policy, input.len())?;
        let actual = precompile_gas::gas_cost(self.descriptor, input)?
            .ok_or(EvmCoreError::PrecompileBackendUnavailable)?;
        if self.gas_cost != Some(actual) {
            return Err(EvmCoreError::PrecompilePlanInputMismatch);
        }
        Ok(actual)
    }
}

/// Fork-aware precompile registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmPrecompileRegistry {
    fork: EvmFork,
}

impl EvmPrecompileRegistry {
    /// Creates a registry for a supported fork.
    pub const fn try_new(fork: EvmFork) -> Result<Self, EvmCoreError> {
        if !fork.is_supported() {
            return Err(EvmCoreError::UnsupportedFork);
        }
        Ok(Self { fork })
    }

    /// Returns the fork this registry was created for.
    #[must_use]
    pub const fn fork(self) -> EvmFork {
        self.fork
    }

    /// Looks up a known precompile address for this fork.
    pub fn lookup(
        self,
        address: EvmAddress,
    ) -> Result<Option<EvmPrecompileDescriptor>, EvmCoreError> {
        let Some(kind) = precompile_kind_from_address(address) else {
            return Ok(None);
        };
        self.descriptor(kind).map(Some)
    }

    /// Returns a descriptor for `kind` when it exists in this registry fork.
    pub const fn descriptor(
        self,
        kind: EvmPrecompileKind,
    ) -> Result<EvmPrecompileDescriptor, EvmCoreError> {
        if self.fork.get() < kind.introduced_in().get() {
            return Err(EvmCoreError::PrecompileNotAvailableInFork);
        }
        Ok(descriptor_for_kind(kind, self.fork))
    }
}

pub(crate) fn execute_identity(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    let target = output
        .get_mut(..input.len())
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    target.copy_from_slice(input);
    Ok(input.len())
}

pub(crate) fn execute_sha256(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    hash_precompile::execute_sha256(input, output)
}

pub(crate) fn execute_ripemd160(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    hash_precompile::execute_ripemd160(input, output)
}

const fn precompile_address(value: u16) -> EvmAddress {
    let mut bytes = [0u8; EvmAddress::LEN];
    let low = value.to_be_bytes();
    bytes[18] = low[0];
    bytes[19] = low[1];
    EvmAddress::from_bytes(bytes)
}

fn precompile_kind_from_address(address: EvmAddress) -> Option<EvmPrecompileKind> {
    let bytes = address.to_bytes();
    if !prefix_is_zero(&bytes) {
        return None;
    }
    match u16::from_be_bytes([bytes[18], bytes[19]]) {
        0x01 => Some(EvmPrecompileKind::EcRecover),
        0x02 => Some(EvmPrecompileKind::Sha256),
        0x03 => Some(EvmPrecompileKind::Ripemd160),
        0x04 => Some(EvmPrecompileKind::Identity),
        0x05 => Some(EvmPrecompileKind::Modexp),
        0x06 => Some(EvmPrecompileKind::Bn254Add),
        0x07 => Some(EvmPrecompileKind::Bn254Mul),
        0x08 => Some(EvmPrecompileKind::Bn254Pairing),
        0x09 => Some(EvmPrecompileKind::Blake2F),
        0x0a => Some(EvmPrecompileKind::KzgPointEvaluation),
        0x0b => Some(EvmPrecompileKind::Bls12G1Add),
        0x0c => Some(EvmPrecompileKind::Bls12G1Msm),
        0x0d => Some(EvmPrecompileKind::Bls12G2Add),
        0x0e => Some(EvmPrecompileKind::Bls12G2Msm),
        0x0f => Some(EvmPrecompileKind::Bls12PairingCheck),
        0x10 => Some(EvmPrecompileKind::Bls12MapFpToG1),
        0x11 => Some(EvmPrecompileKind::Bls12MapFp2ToG2),
        _ => None,
    }
}

fn prefix_is_zero(bytes: &[u8; EvmAddress::LEN]) -> bool {
    bytes[..18].iter().all(|byte| *byte == 0)
}

const fn descriptor_for_kind(kind: EvmPrecompileKind, fork: EvmFork) -> EvmPrecompileDescriptor {
    let implementation = match kind {
        EvmPrecompileKind::EcRecover => EvmPrecompileImplementation::NativeEcRecover,
        EvmPrecompileKind::Sha256 => EvmPrecompileImplementation::NativeSha256,
        EvmPrecompileKind::Ripemd160 => EvmPrecompileImplementation::NativeRipemd160,
        EvmPrecompileKind::Identity => EvmPrecompileImplementation::NativeIdentity,
        EvmPrecompileKind::Modexp => EvmPrecompileImplementation::NativeModexp,
        EvmPrecompileKind::Bn254Add => EvmPrecompileImplementation::NativeBn254Add,
        EvmPrecompileKind::Bn254Mul => EvmPrecompileImplementation::NativeBn254Mul,
        EvmPrecompileKind::Bn254Pairing => EvmPrecompileImplementation::NativeBn254PairingFrame,
        EvmPrecompileKind::Blake2F => EvmPrecompileImplementation::NativeBlake2F,
        _ => EvmPrecompileImplementation::RequiresCryptoBackend,
    };
    EvmPrecompileDescriptor {
        kind,
        address: kind.address(),
        fork,
        introduced_in: kind.introduced_in(),
        implementation,
        input_policy: input_policy(kind),
        gas_policy: gas_policy_for_kind(kind, fork),
        output_len: output_len(kind),
    }
}

const fn input_policy(kind: EvmPrecompileKind) -> EvmPrecompileInputPolicy {
    match kind {
        EvmPrecompileKind::Bn254Pairing => EvmPrecompileInputPolicy::MultipleOf(PAIRING_ITEM_BYTES),
        EvmPrecompileKind::Blake2F => EvmPrecompileInputPolicy::Exact(BLAKE2F_INPUT_BYTES),
        EvmPrecompileKind::KzgPointEvaluation => advanced_precompile::input_policy(kind),
        EvmPrecompileKind::Bls12G1Add
        | EvmPrecompileKind::Bls12G1Msm
        | EvmPrecompileKind::Bls12G2Add
        | EvmPrecompileKind::Bls12G2Msm
        | EvmPrecompileKind::Bls12PairingCheck
        | EvmPrecompileKind::Bls12MapFpToG1
        | EvmPrecompileKind::Bls12MapFp2ToG2 => advanced_precompile::input_policy(kind),
        _ => EvmPrecompileInputPolicy::BoundedAny,
    }
}

const fn gas_policy_for_kind(kind: EvmPrecompileKind, fork: EvmFork) -> EvmPrecompileGasPolicy {
    match kind {
        EvmPrecompileKind::EcRecover => EvmPrecompileGasPolicy::Fixed(EvmGas::new(3_000)),
        EvmPrecompileKind::Sha256 => EvmPrecompileGasPolicy::Words {
            base: EvmGas::new(60),
            per_word: EvmGas::new(12),
        },
        EvmPrecompileKind::Ripemd160 => EvmPrecompileGasPolicy::Words {
            base: EvmGas::new(600),
            per_word: EvmGas::new(120),
        },
        EvmPrecompileKind::Identity => EvmPrecompileGasPolicy::Words {
            base: EvmGas::new(15),
            per_word: EvmGas::new(3),
        },
        EvmPrecompileKind::Modexp => EvmPrecompileGasPolicy::Modexp,
        EvmPrecompileKind::Bn254Add if fork.get() >= EvmFork::ISTANBUL.get() => {
            EvmPrecompileGasPolicy::Fixed(EvmGas::new(150))
        }
        EvmPrecompileKind::Bn254Add => EvmPrecompileGasPolicy::Fixed(EvmGas::new(500)),
        EvmPrecompileKind::Bn254Mul if fork.get() >= EvmFork::ISTANBUL.get() => {
            EvmPrecompileGasPolicy::Fixed(EvmGas::new(6_000))
        }
        EvmPrecompileKind::Bn254Mul => EvmPrecompileGasPolicy::Fixed(EvmGas::new(40_000)),
        EvmPrecompileKind::Bn254Pairing => EvmPrecompileGasPolicy::Bn254Pairing,
        EvmPrecompileKind::Blake2F => EvmPrecompileGasPolicy::Blake2FRounds,
        EvmPrecompileKind::KzgPointEvaluation
        | EvmPrecompileKind::Bls12G1Add
        | EvmPrecompileKind::Bls12G1Msm
        | EvmPrecompileKind::Bls12G2Add
        | EvmPrecompileKind::Bls12G2Msm
        | EvmPrecompileKind::Bls12PairingCheck
        | EvmPrecompileKind::Bls12MapFpToG1
        | EvmPrecompileKind::Bls12MapFp2ToG2 => advanced_precompile::gas_policy(kind),
    }
}

const fn output_len(kind: EvmPrecompileKind) -> Option<usize> {
    match kind {
        EvmPrecompileKind::EcRecover
        | EvmPrecompileKind::Sha256
        | EvmPrecompileKind::Ripemd160
        | EvmPrecompileKind::Bn254Pairing => Some(WORD_BYTES),
        EvmPrecompileKind::Bn254Add | EvmPrecompileKind::Bn254Mul => Some(BN254_POINT_OUTPUT_BYTES),
        EvmPrecompileKind::Blake2F => Some(BLAKE2F_OUTPUT_BYTES),
        EvmPrecompileKind::KzgPointEvaluation
        | EvmPrecompileKind::Bls12G1Add
        | EvmPrecompileKind::Bls12G1Msm
        | EvmPrecompileKind::Bls12G2Add
        | EvmPrecompileKind::Bls12G2Msm
        | EvmPrecompileKind::Bls12PairingCheck
        | EvmPrecompileKind::Bls12MapFpToG1
        | EvmPrecompileKind::Bls12MapFp2ToG2 => Some(advanced_precompile::output_len(kind)),
        _ => None,
    }
}

pub(crate) fn validate_input_policy(
    policy: EvmPrecompileInputPolicy,
    len: usize,
) -> Result<(), EvmCoreError> {
    if len > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    match policy {
        EvmPrecompileInputPolicy::BoundedAny => Ok(()),
        EvmPrecompileInputPolicy::Exact(expected) if len == expected => Ok(()),
        EvmPrecompileInputPolicy::MultipleOf(chunk) if chunk != 0 && len.is_multiple_of(chunk) => {
            Ok(())
        }
        EvmPrecompileInputPolicy::NonEmptyMultipleOf(chunk)
            if chunk != 0 && len != 0 && len.is_multiple_of(chunk) =>
        {
            Ok(())
        }
        _ => Err(EvmCoreError::PrecompileInvalidInputLength),
    }
}
