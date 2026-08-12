mod arithmetic;
mod gas;
mod length;

use crate::{EvmCoreError, EvmFork, EvmGas};

pub use length::EvmModExpLength;

/// ModExp header byte length: base, exponent, and modulus lengths.
pub const EVM_MODEXP_HEADER_BYTES: usize = 96;

pub(super) const WORD_BYTES: usize = 32;
const LENGTH_BASE_OFFSET: usize = 0;
const LENGTH_EXPONENT_OFFSET: usize = 32;
const LENGTH_MODULUS_OFFSET: usize = 64;
pub(super) const PAYLOAD_OFFSET: usize = 96;

/// Parsed 256-bit ModExp input lengths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmModExpInput {
    base_len: EvmModExpLength,
    exponent_len: EvmModExpLength,
    modulus_len: EvmModExpLength,
}

impl EvmModExpInput {
    /// Returns the declared 256-bit base byte length.
    #[must_use]
    pub const fn base_len(self) -> EvmModExpLength {
        self.base_len
    }

    /// Returns the declared 256-bit exponent byte length.
    #[must_use]
    pub const fn exponent_len(self) -> EvmModExpLength {
        self.exponent_len
    }

    /// Returns the declared 256-bit modulus byte length.
    #[must_use]
    pub const fn modulus_len(self) -> EvmModExpLength {
        self.modulus_len
    }
}

/// Caller-owned storage for dependency-free arbitrary-length ModExp execution.
pub struct EvmModExpWorkspace<'storage> {
    limbs: &'storage mut [u32],
}

impl<'storage> EvmModExpWorkspace<'storage> {
    /// Wraps caller-owned limbs for one ModExp execution.
    pub fn new(limbs: &'storage mut [u32]) -> Self {
        Self { limbs }
    }

    /// Returns the available limb capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.limbs.len()
    }

    pub(crate) fn storage(&mut self) -> &mut [u32] {
        self.limbs
    }
}

impl core::fmt::Debug for EvmModExpWorkspace<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EvmModExpWorkspace")
            .field("capacity", &self.capacity())
            .finish_non_exhaustive()
    }
}

/// Parses all ModExp length words with EIP-198 right-padding semantics.
///
/// Lengths remain 256-bit values. Parsing does not impose a private operand
/// ceiling and does not allocate attacker-declared absent payload bytes.
///
/// # Security
///
/// This parser accepts public EVM calldata and is not constant-time. Convert a
/// length to `usize` only after the fork gas quote proves the call payable in
/// the configured execution envelope.
pub fn parse_modexp_input(input: &[u8]) -> Result<EvmModExpInput, EvmCoreError> {
    Ok(EvmModExpInput {
        base_len: EvmModExpLength::read_padded(input, LENGTH_BASE_OFFSET),
        exponent_len: EvmModExpLength::read_padded(input, LENGTH_EXPONENT_OFFSET),
        modulus_len: EvmModExpLength::read_padded(input, LENGTH_MODULUS_OFFSET),
    })
}

/// Returns caller-owned workspace limbs required by this ModExp frame.
///
/// This conversion is intended for an already quoted, payable call. An
/// unrepresentable modulus length is necessarily outside this crate's bounded
/// execution gas envelope and returns `PrecompileInputTooLarge`.
pub fn modexp_workspace_limbs(input: &[u8]) -> Result<usize, EvmCoreError> {
    let modulus_len = parse_modexp_input(input)?.modulus_len().try_to_usize()?;
    arithmetic::required_limbs(modulus_len)
}

pub(crate) fn execute_modexp(
    input: &[u8],
    output: &mut [u8],
    workspace: &mut EvmModExpWorkspace<'_>,
) -> Result<usize, EvmCoreError> {
    let parsed = parse_modexp_input(input)?;
    let modulus_len = parsed.modulus_len().try_to_usize()?;
    let target = output
        .get_mut(..modulus_len)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    if modulus_len == 0 {
        return Ok(0);
    }
    let required = arithmetic::required_limbs(modulus_len)?;
    if workspace.capacity() < required {
        return Err(EvmCoreError::PrecompileWorkspaceTooSmall);
    }
    let layout = ModExpLayout::new(parsed)?;
    arithmetic::execute(input, parsed, layout, target, workspace.storage())?;
    Ok(modulus_len)
}

pub(crate) fn modexp_gas_cost(fork: EvmFork, input: &[u8]) -> Result<EvmGas, EvmCoreError> {
    gas::cost(fork, input)
}

#[derive(Clone, Copy)]
pub(super) struct ModExpLayout {
    pub(super) base_offset: usize,
    pub(super) exponent_offset: usize,
    pub(super) modulus_offset: usize,
}

impl ModExpLayout {
    fn new(parsed: EvmModExpInput) -> Result<Self, EvmCoreError> {
        let base_len = parsed.base_len().try_to_usize()?;
        let exponent_len = parsed.exponent_len().try_to_usize()?;
        let modulus_len = parsed.modulus_len().try_to_usize()?;
        let exponent_offset = PAYLOAD_OFFSET
            .checked_add(base_len)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        let modulus_offset = exponent_offset
            .checked_add(exponent_len)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        modulus_offset
            .checked_add(modulus_len)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        Ok(Self {
            base_offset: PAYLOAD_OFFSET,
            exponent_offset,
            modulus_offset,
        })
    }
}
