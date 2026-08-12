use core::{fmt, marker::PhantomData};

use crate::{
    EVM_MAX_GAS_LIMIT, EvmCoreError, EvmGas, EvmGasMeter, EvmModExpWorkspace,
    EvmPrecompileDescriptor, EvmPrecompileImplementation, EvmPrecompileKind, EvmPrecompileRegistry,
    blake2f::execute_blake2f,
    bn254::{execute_bn254_add, execute_bn254_mul},
    bn254_pairing::execute_bn254_pairing,
    ecrecover::{EvmEcRecoverBackend, EvmPrecompileKeccak256, execute_ecrecover},
    modexp::{execute_modexp, modexp_workspace_limbs, parse_modexp_input},
    precompile::{execute_identity, execute_ripemd160, execute_sha256, validate_input_policy},
    precompile_gas,
};

/// Result category passed from a precompile to CALL integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmPrecompileStatus {
    /// Execution succeeded and `output_len` bytes are available.
    Success,
    /// Execution failed and the complete supplied precompile gas was consumed.
    CallFailure,
}

/// One terminal, CALL-ready precompile result.
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use eth_valkyoth_evm_core::{
///     EvmFork, EvmGas, EvmGasMeter, EvmIdentity, EvmPrecompileKind,
///     EvmPrecompileRegistry,
/// };
///
/// let descriptor = EvmPrecompileRegistry::try_new(EvmFork::FRONTIER)?
///     .descriptor(EvmPrecompileKind::Identity)?;
/// let input = [1_u8, 2, 3];
/// let quote = descriptor.quote::<EvmIdentity>(&input)?;
/// let mut gas = EvmGasMeter::try_new(EvmGas::new(18))?;
/// let mut output = [0_u8; 3];
/// quote.authorize_and_execute_identity(&mut gas, &mut output)?;
/// # Ok::<(), eth_valkyoth_evm_core::EvmCoreError>(())
/// ```
#[must_use = "CALL status, gas, output length, and rollback must be handled"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmPrecompileOutcome {
    status: EvmPrecompileStatus,
    gas_consumed: EvmGas,
    output_len: usize,
    error: Option<EvmCoreError>,
}

impl EvmPrecompileOutcome {
    /// Returns the terminal CALL result category.
    #[must_use]
    pub const fn status(self) -> EvmPrecompileStatus {
        self.status
    }

    /// Returns gas consumed by this precompile call.
    #[must_use]
    pub const fn gas_consumed(self) -> EvmGas {
        self.gas_consumed
    }

    /// Returns the valid output prefix length, or zero on call failure.
    #[must_use]
    pub const fn output_len(self) -> usize {
        self.output_len
    }

    /// Returns the bounded failure category, when execution failed.
    #[must_use]
    pub const fn error(self) -> Option<EvmCoreError> {
        self.error
    }

    /// Returns whether CALL integration must revert child effects and push zero.
    #[must_use]
    pub const fn requires_rollback(self) -> bool {
        matches!(self.status, EvmPrecompileStatus::CallFailure)
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Sealed identity for a precompile executable in this release.
pub trait EvmExecutablePrecompile: sealed::Sealed {
    /// Canonical precompile identity.
    const KIND: EvmPrecompileKind;
    /// Canonical implementation boundary.
    const IMPLEMENTATION: EvmPrecompileImplementation;

    #[doc(hidden)]
    fn output_len(input: &[u8], gas_cost: EvmGas) -> Result<usize, EvmCoreError>;
}

macro_rules! fixed_precompile {
    ($name:ident, $kind:ident, $implementation:ident, $output:expr) => {
        #[doc = concat!("Type identity for the `", stringify!($kind), "` precompile.")]
        pub enum $name {}

        impl sealed::Sealed for $name {}

        impl EvmExecutablePrecompile for $name {
            const KIND: EvmPrecompileKind = EvmPrecompileKind::$kind;
            const IMPLEMENTATION: EvmPrecompileImplementation =
                EvmPrecompileImplementation::$implementation;

            fn output_len(_input: &[u8], _gas_cost: EvmGas) -> Result<usize, EvmCoreError> {
                Ok($output)
            }
        }
    };
}

fixed_precompile!(EvmEcRecover, EcRecover, NativeEcRecover, 32);
fixed_precompile!(EvmSha256, Sha256, NativeSha256, 32);
fixed_precompile!(EvmRipemd160, Ripemd160, NativeRipemd160, 32);
fixed_precompile!(EvmBn254Add, Bn254Add, NativeBn254Add, 64);
fixed_precompile!(EvmBn254Mul, Bn254Mul, NativeBn254Mul, 64);
fixed_precompile!(EvmBn254Pairing, Bn254Pairing, NativeBn254PairingFrame, 32);
fixed_precompile!(EvmBlake2F, Blake2F, NativeBlake2F, 64);

/// Type identity for the identity precompile.
pub enum EvmIdentity {}

impl sealed::Sealed for EvmIdentity {}

impl EvmExecutablePrecompile for EvmIdentity {
    const KIND: EvmPrecompileKind = EvmPrecompileKind::Identity;
    const IMPLEMENTATION: EvmPrecompileImplementation = EvmPrecompileImplementation::NativeIdentity;

    fn output_len(input: &[u8], _gas_cost: EvmGas) -> Result<usize, EvmCoreError> {
        Ok(input.len())
    }
}

/// Type identity for the gas-bounded ModExp precompile.
pub enum EvmModexp {}

impl sealed::Sealed for EvmModexp {}

impl EvmExecutablePrecompile for EvmModexp {
    const KIND: EvmPrecompileKind = EvmPrecompileKind::Modexp;
    const IMPLEMENTATION: EvmPrecompileImplementation = EvmPrecompileImplementation::NativeModexp;

    fn output_len(input: &[u8], gas_cost: EvmGas) -> Result<usize, EvmCoreError> {
        if gas_cost.get() > EVM_MAX_GAS_LIMIT {
            return Ok(0);
        }
        parse_modexp_input(input)?.modulus_len().try_to_usize()
    }
}

/// Exact-input gas quote for one executable precompile.
///
/// The immutable input borrow is the TOCTOU binding: safe Rust cannot replace
/// or mutate the quoted bytes while this value or its paid successor exists.
///
/// ```compile_fail
/// use eth_valkyoth_evm_core::{EvmFork, EvmIdentity, EvmPrecompileKind, EvmPrecompileRegistry};
///
/// let descriptor = EvmPrecompileRegistry::try_new(EvmFork::FRONTIER)?
///     .descriptor(EvmPrecompileKind::Identity)?;
/// let mut input = [1_u8, 2, 3];
/// let quote = descriptor.quote::<EvmIdentity>(&input)?;
/// input[0] = 9; // rejected: the exact quoted bytes remain immutably borrowed
/// let _ = quote;
/// # Ok::<(), eth_valkyoth_evm_core::EvmCoreError>(())
/// ```
///
/// Raw paid authority cannot be created or named by external safe Rust:
///
/// ```compile_fail
/// use eth_valkyoth_evm_core::{
///     EvmFork, EvmGas, EvmGasMeter, EvmIdentity, EvmPrecompileKind,
///     EvmPrecompileRegistry,
/// };
///
/// let descriptor = EvmPrecompileRegistry::try_new(EvmFork::FRONTIER)?
///     .descriptor(EvmPrecompileKind::Identity)?;
/// let input = [1_u8, 2, 3];
/// let quote = descriptor.quote::<EvmIdentity>(&input)?;
/// let mut gas = EvmGasMeter::try_new(EvmGas::new(18))?;
/// let mut output = [0_u8; 3];
/// let _paid = quote.authorize_internal(&mut gas, &mut output)?;
/// # Ok::<(), eth_valkyoth_evm_core::EvmCoreError>(())
/// ```
///
/// ```compile_fail
/// use eth_valkyoth_evm_core::PaidPrecompile;
/// # fn main() {}
/// ```
pub struct EvmPrecompileGasQuote<'input, K> {
    descriptor: EvmPrecompileDescriptor,
    input: &'input [u8],
    gas_cost: EvmGas,
    output_len: usize,
    marker: PhantomData<K>,
}

impl<K> fmt::Debug for EvmPrecompileGasQuote<'_, K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EvmPrecompileGasQuote")
            .field("kind", &self.descriptor.kind)
            .field("input_len", &self.input.len())
            .field("gas_cost", &self.gas_cost)
            .field("output_len", &self.output_len)
            .finish()
    }
}

impl EvmPrecompileDescriptor {
    /// Quotes one native precompile while borrowing its exact input.
    pub fn quote<K: EvmExecutablePrecompile>(
        self,
        input: &[u8],
    ) -> Result<EvmPrecompileGasQuote<'_, K>, EvmCoreError> {
        let canonical = EvmPrecompileRegistry::try_new(self.fork)?.descriptor(self.kind)?;
        if self != canonical {
            return Err(EvmCoreError::PrecompileDescriptorMismatch);
        }
        if self.kind != K::KIND || self.implementation != K::IMPLEMENTATION {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        validate_input_policy(self.input_policy, input.len())?;
        let gas_cost = precompile_gas::gas_cost(self, input)?
            .ok_or(EvmCoreError::PrecompileBackendUnavailable)?;
        let output_len = K::output_len(input, gas_cost)?;
        Ok(EvmPrecompileGasQuote {
            descriptor: self,
            input,
            gas_cost,
            output_len,
            marker: PhantomData,
        })
    }
}

impl<'input, K> EvmPrecompileGasQuote<'input, K> {
    /// Returns the canonical gas quote.
    #[must_use]
    pub const fn gas_cost(&self) -> EvmGas {
        self.gas_cost
    }

    /// Returns the required execution output capacity.
    #[must_use]
    pub const fn output_len(&self) -> usize {
        self.output_len
    }

    /// Charges the quote and creates crate-private one-shot authority.
    pub(crate) fn authorize_internal<'meter, 'output>(
        self,
        gas_meter: &'meter mut EvmGasMeter,
        output: &'output mut [u8],
    ) -> Result<PaidPrecompile<'input, 'meter, 'output, K>, EvmCoreError> {
        if output.len() < self.output_len {
            return Err(EvmCoreError::PrecompileOutputTooSmall);
        }
        let supplied_gas = gas_meter.remaining()?;
        gas_meter.charge(self.gas_cost)?;
        Ok(PaidPrecompile {
            quote: self,
            gas_meter,
            output,
            supplied_gas,
            armed: true,
        })
    }
}

/// Crate-private proof that exact precompile work was admitted and charged.
///
/// This capability is intentionally neither constructible, cloneable, nor
/// copyable outside its authorization path.
#[must_use = "paid precompile authority must be executed to a terminal outcome"]
pub(crate) struct PaidPrecompile<'input, 'meter, 'output, K> {
    quote: EvmPrecompileGasQuote<'input, K>,
    gas_meter: &'meter mut EvmGasMeter,
    output: &'output mut [u8],
    supplied_gas: EvmGas,
    armed: bool,
}

impl<K> PaidPrecompile<'_, '_, '_, K> {
    fn finish(mut self, result: Result<usize, EvmCoreError>) -> EvmPrecompileOutcome {
        let outcome = match result {
            Ok(output_len) => EvmPrecompileOutcome {
                status: EvmPrecompileStatus::Success,
                gas_consumed: self.quote.gas_cost,
                output_len,
                error: None,
            },
            Err(error) => {
                self.gas_meter.consume_remaining();
                EvmPrecompileOutcome {
                    status: EvmPrecompileStatus::CallFailure,
                    gas_consumed: self.supplied_gas,
                    output_len: 0,
                    error: Some(error),
                }
            }
        };
        self.armed = false;
        outcome
    }
}

impl<K> Drop for PaidPrecompile<'_, '_, '_, K> {
    fn drop(&mut self) {
        if self.armed {
            self.gas_meter.consume_remaining();
        }
    }
}

macro_rules! native_execution {
    ($marker:ty, $method:ident, $execute:path) => {
        impl PaidPrecompile<'_, '_, '_, $marker> {
            #[doc = concat!("Executes the paid `", stringify!($method), "` precompile.")]
            pub(crate) fn $method(self) -> EvmPrecompileOutcome {
                let result = $execute(self.quote.input, self.output);
                self.finish(result)
            }
        }
    };
}

macro_rules! atomic_native_execution {
    ($marker:ty, $method:ident, $execute:ident) => {
        impl EvmPrecompileGasQuote<'_, $marker> {
            /// Authorizes and executes without exposing the armed capability.
            pub fn $method(
                self,
                gas_meter: &mut EvmGasMeter,
                output: &mut [u8],
            ) -> Result<EvmPrecompileOutcome, EvmCoreError> {
                Ok(self.authorize_internal(gas_meter, output)?.$execute())
            }
        }
    };
}

native_execution!(EvmIdentity, execute_identity, execute_identity);
native_execution!(EvmSha256, execute_sha256, execute_sha256);
native_execution!(EvmRipemd160, execute_ripemd160, execute_ripemd160);
native_execution!(EvmBn254Add, execute_bn254_add, execute_bn254_add);
native_execution!(EvmBn254Mul, execute_bn254_mul, execute_bn254_mul);
native_execution!(
    EvmBn254Pairing,
    execute_bn254_pairing,
    execute_bn254_pairing
);
native_execution!(EvmBlake2F, execute_blake2f, execute_blake2f);

atomic_native_execution!(
    EvmIdentity,
    authorize_and_execute_identity,
    execute_identity
);
atomic_native_execution!(EvmSha256, authorize_and_execute_sha256, execute_sha256);
atomic_native_execution!(
    EvmRipemd160,
    authorize_and_execute_ripemd160,
    execute_ripemd160
);
atomic_native_execution!(
    EvmBn254Add,
    authorize_and_execute_bn254_add,
    execute_bn254_add
);
atomic_native_execution!(
    EvmBn254Mul,
    authorize_and_execute_bn254_mul,
    execute_bn254_mul
);
atomic_native_execution!(
    EvmBn254Pairing,
    authorize_and_execute_bn254_pairing,
    execute_bn254_pairing
);
atomic_native_execution!(EvmBlake2F, authorize_and_execute_blake2f, execute_blake2f);

impl PaidPrecompile<'_, '_, '_, EvmModexp> {
    pub(crate) fn execute_modexp(
        self,
        workspace: &mut EvmModExpWorkspace<'_>,
    ) -> EvmPrecompileOutcome {
        let result = execute_modexp(self.quote.input, self.output, workspace);
        self.finish(result)
    }
}

impl EvmPrecompileGasQuote<'_, EvmModexp> {
    /// Returns caller-owned limbs required to execute this quoted frame.
    pub fn modexp_workspace_limbs(&self) -> Result<usize, EvmCoreError> {
        if self.gas_cost.get() > EVM_MAX_GAS_LIMIT {
            return Ok(0);
        }
        modexp_workspace_limbs(self.input)
    }

    /// Authorizes and executes ModExp with explicit caller-owned workspace.
    pub fn authorize_and_execute_modexp(
        self,
        gas_meter: &mut EvmGasMeter,
        output: &mut [u8],
        workspace: &mut EvmModExpWorkspace<'_>,
    ) -> Result<EvmPrecompileOutcome, EvmCoreError> {
        let required = self.modexp_workspace_limbs()?;
        if workspace.capacity() < required {
            return Err(EvmCoreError::PrecompileWorkspaceTooSmall);
        }
        Ok(self
            .authorize_internal(gas_meter, output)?
            .execute_modexp(workspace))
    }
}

impl PaidPrecompile<'_, '_, '_, EvmEcRecover> {
    /// Executes paid ECRECOVER with caller-provided cryptographic backends.
    pub(crate) fn execute_ecrecover<B, H>(self, backend: B, hasher: H) -> EvmPrecompileOutcome
    where
        B: EvmEcRecoverBackend,
        H: EvmPrecompileKeccak256,
    {
        let result = execute_ecrecover(self.quote.input, self.output, backend, hasher);
        self.finish(result)
    }
}

impl EvmPrecompileGasQuote<'_, EvmEcRecover> {
    /// Authorizes and executes ECRECOVER without exposing an armed capability.
    pub fn authorize_and_execute_ecrecover<B, H>(
        self,
        gas_meter: &mut EvmGasMeter,
        output: &mut [u8],
        backend: B,
        hasher: H,
    ) -> Result<EvmPrecompileOutcome, EvmCoreError>
    where
        B: EvmEcRecoverBackend,
        H: EvmPrecompileKeccak256,
    {
        Ok(self
            .authorize_internal(gas_meter, output)?
            .execute_ecrecover(backend, hasher))
    }
}
