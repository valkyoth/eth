use crate::EvmCoreError;
use core::cmp::Ordering;

/// A canonical 256-bit EVM stack word.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EvmWord([u8; Self::LEN]);

impl EvmWord {
    /// Number of bytes in an EVM word.
    pub const LEN: usize = 32;
    /// The zero word.
    pub const ZERO: Self = Self([0u8; Self::LEN]);

    /// Constructs an EVM word from big-endian bytes.
    #[must_use]
    pub const fn from_be_bytes(bytes: [u8; Self::LEN]) -> Self {
        Self(bytes)
    }

    /// Constructs an EVM word from up to 32 big-endian bytes.
    pub fn from_be_slice(bytes: &[u8]) -> Result<Self, EvmCoreError> {
        if bytes.len() > Self::LEN {
            return Err(EvmCoreError::WordInputTooLarge);
        }
        let mut output = [0u8; Self::LEN];
        for (source, slot) in bytes.iter().rev().zip(output.iter_mut().rev()) {
            *slot = *source;
        }
        Ok(Self(output))
    }

    /// Constructs an EVM word from a `usize`.
    #[must_use]
    pub fn from_usize(value: usize) -> Self {
        let mut output = [0u8; Self::LEN];
        for (source, slot) in value
            .to_be_bytes()
            .iter()
            .rev()
            .zip(output.iter_mut().rev())
        {
            *slot = *source;
        }
        Self(output)
    }

    /// Constructs an EVM word from a boolean.
    #[must_use]
    pub const fn from_bool(value: bool) -> Self {
        if value {
            Self::from_be_bytes([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1,
            ])
        } else {
            Self::ZERO
        }
    }

    /// Returns the word as big-endian bytes.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; Self::LEN] {
        self.0
    }

    /// Borrows the word bytes.
    #[must_use]
    pub const fn as_be_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }

    /// Returns whether the word is zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0.iter().all(|byte| *byte == 0)
    }

    /// Converts the word to `usize` when it fits.
    pub fn to_usize(self) -> Result<usize, EvmCoreError> {
        let mut value = 0usize;
        let max_bytes = core::mem::size_of::<usize>();
        for (position, byte) in self.0.iter().enumerate() {
            if position < Self::LEN.saturating_sub(max_bytes) {
                if *byte != 0 {
                    return Err(EvmCoreError::WordInputTooLarge);
                }
                continue;
            }
            value = value
                .checked_mul(256)
                .and_then(|next| next.checked_add(usize::from(*byte)))
                .ok_or(EvmCoreError::WordInputTooLarge)?;
        }
        Ok(value)
    }

    /// Wrapping 256-bit addition.
    #[must_use]
    pub fn wrapping_add_word(self, rhs: Self) -> Self {
        let mut output = [0u8; Self::LEN];
        let mut carry = 0u16;
        for ((left, right), slot) in self
            .0
            .iter()
            .rev()
            .zip(rhs.0.iter().rev())
            .zip(output.iter_mut().rev())
        {
            let sum = add_u16(add_u16(u16::from(*left), u16::from(*right)), carry);
            *slot = low_u8_from_u16(sum);
            carry = sum / 256;
        }
        Self(output)
    }

    /// Wrapping 256-bit subtraction.
    #[must_use]
    pub fn wrapping_sub_word(self, rhs: Self) -> Self {
        let mut output = [0u8; Self::LEN];
        let mut borrow = 0u16;
        for ((left, right), slot) in self
            .0
            .iter()
            .rev()
            .zip(rhs.0.iter().rev())
            .zip(output.iter_mut().rev())
        {
            let lhs = u16::from(*left);
            let rhs_with_borrow = add_u16(u16::from(*right), borrow);
            if lhs >= rhs_with_borrow {
                let diff = match lhs.checked_sub(rhs_with_borrow) {
                    Some(value) => value,
                    None => return Self::ZERO,
                };
                *slot = low_u8_from_u16(diff);
                borrow = 0;
            } else {
                let diff = add_u16(256, lhs).saturating_sub(rhs_with_borrow);
                *slot = low_u8_from_u16(diff);
                borrow = 1;
            }
        }
        Self(output)
    }

    /// Wrapping 256-bit multiplication.
    #[must_use]
    pub fn wrapping_mul_word(self, rhs: Self) -> Self {
        let mut accumulator = [0u32; Self::LEN];
        for lhs_offset in 0..Self::LEN {
            for rhs_offset in 0..Self::LEN {
                let product_offset = match lhs_offset.checked_add(rhs_offset) {
                    Some(offset) => offset,
                    None => return Self::ZERO,
                };
                if product_offset >= Self::LEN {
                    continue;
                }
                let lhs_index = match index_from_low_offset(lhs_offset) {
                    Some(index) => index,
                    None => return Self::ZERO,
                };
                let rhs_index = match index_from_low_offset(rhs_offset) {
                    Some(index) => index,
                    None => return Self::ZERO,
                };
                let out_index = match index_from_low_offset(product_offset) {
                    Some(index) => index,
                    None => return Self::ZERO,
                };
                let left = match self.0.get(lhs_index) {
                    Some(byte) => u32::from(*byte),
                    None => return Self::ZERO,
                };
                let right = match rhs.0.get(rhs_index) {
                    Some(byte) => u32::from(*byte),
                    None => return Self::ZERO,
                };
                let product = match left.checked_mul(right) {
                    Some(value) => value,
                    None => return Self::ZERO,
                };
                let slot = match accumulator.get_mut(out_index) {
                    Some(slot) => slot,
                    None => return Self::ZERO,
                };
                *slot = add_u32(*slot, product);
            }
        }

        let mut carry = 0u32;
        for slot in accumulator.iter_mut().rev() {
            let sum = add_u32(*slot, carry);
            *slot = sum % 256;
            carry = sum / 256;
        }

        let mut output = [0u8; Self::LEN];
        for (value, slot) in accumulator.iter().zip(output.iter_mut()) {
            *slot = low_u8_from_u32(*value);
        }
        Self(output)
    }

    /// Bitwise `AND`.
    #[must_use]
    pub fn bitand_word(self, rhs: Self) -> Self {
        let mut output = [0u8; Self::LEN];
        for ((left, right), slot) in self.0.iter().zip(rhs.0.iter()).zip(output.iter_mut()) {
            *slot = *left & *right;
        }
        Self(output)
    }

    /// Bitwise `OR`.
    #[must_use]
    pub fn bitor_word(self, rhs: Self) -> Self {
        let mut output = [0u8; Self::LEN];
        for ((left, right), slot) in self.0.iter().zip(rhs.0.iter()).zip(output.iter_mut()) {
            *slot = *left | *right;
        }
        Self(output)
    }

    /// Bitwise `XOR`.
    #[must_use]
    pub fn bitxor_word(self, rhs: Self) -> Self {
        let mut output = [0u8; Self::LEN];
        for ((left, right), slot) in self.0.iter().zip(rhs.0.iter()).zip(output.iter_mut()) {
            *slot = *left ^ *right;
        }
        Self(output)
    }

    /// Bitwise `NOT`.
    #[must_use]
    pub fn bitnot_word(self) -> Self {
        let mut output = [0u8; Self::LEN];
        for (source, slot) in self.0.iter().zip(output.iter_mut()) {
            *slot = !*source;
        }
        Self(output)
    }
}

impl Ord for EvmWord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().cmp(other.0.iter())
    }
}

impl PartialOrd for EvmWord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn add_u16(left: u16, right: u16) -> u16 {
    left.saturating_add(right)
}

fn add_u32(left: u32, right: u32) -> u32 {
    left.saturating_add(right)
}

fn index_from_low_offset(offset: usize) -> Option<usize> {
    SelfLen::LEN.checked_sub(1)?.checked_sub(offset)
}

fn low_u8_from_u16(value: u16) -> u8 {
    u8::try_from(value & 0x00ff).unwrap_or_default()
}

fn low_u8_from_u32(value: u32) -> u8 {
    u8::try_from(value & 0x0000_00ff).unwrap_or_default()
}

struct SelfLen;

impl SelfLen {
    const LEN: usize = EvmWord::LEN;
}
