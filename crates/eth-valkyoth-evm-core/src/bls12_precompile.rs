//! Paid EIP-2537 G1 addition. No scalar, subgroup or other BLS execution.
//!
//! ```
//! use eth_valkyoth_evm_core::{EvmBls12G1Add, EvmFork, EvmGas, EvmGasMeter,
//!     EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus};
//! let descriptor = EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?
//!     .descriptor(EvmPrecompileKind::Bls12G1Add)?;
//! let input = [0u8; 256]; // infinity + infinity
//! let mut output = [0u8; 128];
//! let mut gas = EvmGasMeter::try_new(EvmGas::new(375))?;
//! let outcome = descriptor.quote::<EvmBls12G1Add>(&input)?
//!     .authorize_and_execute_bls12_g1_add(&mut gas, &mut output)?;
//! assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
//! assert_eq!(outcome.output_len(), 128);
//! # Ok::<(), eth_valkyoth_evm_core::EvmCoreError>(())
//! ```

use crate::{EvmBls12381G1Affine, EvmCoreError, parse_bls12381_g1_add};

pub(crate) fn execute_g1_add(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    let target = output
        .get_mut(..128)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    // Validate both canonical encodings before any curve arithmetic.
    let frame = parse_bls12381_g1_add(input)?;
    #[cfg(test)]
    record_validation();
    let left = EvmBls12381G1Affine::try_from_wire(frame.left)?;
    #[cfg(test)]
    record_validation();
    let right = EvmBls12381G1Affine::try_from_wire(frame.right)?;
    #[cfg(test)]
    ADDITIONS.with(|count| count.set(count.get().saturating_add(1)));
    let encoded = left.add_point(right).to_be_bytes();
    // The only caller-buffer write is after all fallible work has completed.
    target.copy_from_slice(&encoded);
    Ok(128)
}

#[cfg(test)]
std::thread_local! {
    static VALIDATIONS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
    static ADDITIONS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_validation() {
    VALIDATIONS.with(|count| count.set(count.get().saturating_add(1)));
}

#[cfg(test)]
#[path = "bls12_precompile_tests.rs"]
mod tests;
