use crate::EvmCoreError;

use super::WORD_BYTES;

/// One 256-bit length declared by EIP-198 ModExp calldata.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EvmModExpLength([u8; WORD_BYTES]);

impl EvmModExpLength {
    pub(super) fn read_padded(input: &[u8], offset: usize) -> Self {
        let mut bytes = [0_u8; WORD_BYTES];
        for (index, slot) in bytes.iter_mut().enumerate() {
            *slot = input
                .get(offset.saturating_add(index))
                .copied()
                .unwrap_or(0);
        }
        Self(bytes)
    }

    /// Returns the canonical 32-byte big-endian representation.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; WORD_BYTES] {
        self.0
    }

    /// Returns whether the declared length is zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0.iter().all(|byte| *byte == 0)
    }

    /// Converts this length after a gas bound has proved it host-representable.
    pub fn try_to_usize(self) -> Result<usize, EvmCoreError> {
        let host_bytes = core::mem::size_of::<usize>();
        let high_len = WORD_BYTES
            .checked_sub(host_bytes)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        if self
            .0
            .get(..high_len)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(EvmCoreError::PrecompileInputTooLarge);
        }

        let mut value = 0_usize;
        for byte in self
            .0
            .get(high_len..)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?
        {
            value = value
                .checked_mul(256)
                .and_then(|current| current.checked_add(usize::from(*byte)))
                .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        }
        Ok(value)
    }

    pub(super) fn saturating_u128(self) -> u128 {
        let high_len = WORD_BYTES.saturating_sub(core::mem::size_of::<u128>());
        if self
            .0
            .get(..high_len)
            .is_some_and(|high| high.iter().any(|byte| *byte != 0))
        {
            return u128::MAX;
        }
        self.0
            .get(high_len..)
            .unwrap_or(&[])
            .iter()
            .fold(0_u128, |value, byte| {
                value.saturating_mul(256).saturating_add(u128::from(*byte))
            })
    }

    pub(super) fn at_most(self, value: u8) -> bool {
        self.saturating_u128() <= u128::from(value)
    }
}
