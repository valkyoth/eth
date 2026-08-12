use core::cmp::Ordering;

use super::{EvmModExpInput, ModExpLayout};
use crate::EvmCoreError;

const LIMB_BYTES: usize = core::mem::size_of::<u32>();
const LIMB_BASE: u64 = 1_u64 << u32::BITS;
const LIMB_MASK: u64 = LIMB_BASE - 1;

pub(super) fn required_limbs(modulus_len: usize) -> Result<usize, EvmCoreError> {
    if modulus_len == 0 {
        return Ok(0);
    }
    let limb_count = modulus_len
        .checked_add(LIMB_BYTES.saturating_sub(1))
        .and_then(|length| length.checked_div(LIMB_BYTES))
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    limb_count
        .checked_mul(6)
        .and_then(|length| length.checked_add(1))
        .ok_or(EvmCoreError::PrecompileInputTooLarge)
}

pub(super) fn execute(
    input: &[u8],
    parsed: EvmModExpInput,
    layout: ModExpLayout,
    output: &mut [u8],
    workspace: &mut [u32],
) -> Result<(), EvmCoreError> {
    let modulus_len = parsed.modulus_len().try_to_usize()?;
    let base_len = parsed.base_len().try_to_usize()?;
    let exponent_len = parsed.exponent_len().try_to_usize()?;
    let required = required_limbs(modulus_len)?;
    let declared_limbs = required
        .checked_sub(1)
        .and_then(|length| length.checked_div(6))
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let limbs = workspace
        .get_mut(..required)
        .ok_or(EvmCoreError::PrecompileWorkspaceTooSmall)?;
    limbs.fill(0);

    let (result, rest) = split_mut(limbs, declared_limbs)?;
    let (base, rest) = split_mut(rest, declared_limbs)?;
    let (modulus, rest) = split_mut(rest, declared_limbs)?;
    let (normalized_modulus, dividend) = split_mut(rest, declared_limbs)?;

    load_segment_le(input, layout.modulus_offset, modulus_len, modulus)?;
    let limb_count = significant_len(modulus);
    if limb_count == 0 {
        output.fill(0);
        return Ok(());
    }

    let result = prefix_mut(result, limb_count)?;
    let base = prefix_mut(base, limb_count)?;
    let modulus = prefix_mut(modulus, limb_count)?;
    let normalized_modulus = prefix_mut(normalized_modulus, limb_count)?;
    let dividend_len = limb_count
        .checked_mul(2)
        .and_then(|length| length.checked_add(1))
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let dividend = prefix_mut(dividend, dividend_len)?;

    let first_set = (0..exponent_len).find_map(|byte_index| {
        let byte = virtual_byte(input, layout.exponent_offset, byte_index);
        (byte != 0).then_some((byte_index, byte, byte.leading_zeros()))
    });
    let Some((first_byte_index, first_byte, leading)) = first_set else {
        set_one_mod(result, modulus)?;
        return write_result_be(result, output);
    };

    reduce_virtual_segment(input, layout.base_offset, base_len, base, modulus)?;
    result.copy_from_slice(base);
    for bit in leading.saturating_add(1)..8 {
        square_mod(result, modulus, normalized_modulus, dividend)?;
        if first_byte & (0x80_u8 >> bit) != 0 {
            multiply_mod(result, base, modulus, normalized_modulus, dividend)?;
        }
    }
    for byte_index in first_byte_index.saturating_add(1)..exponent_len {
        let byte = virtual_byte(input, layout.exponent_offset, byte_index);
        for bit in 0..8 {
            square_mod(result, modulus, normalized_modulus, dividend)?;
            if byte & (0x80_u8 >> bit) != 0 {
                multiply_mod(result, base, modulus, normalized_modulus, dividend)?;
            }
        }
    }
    write_result_be(result, output)
}

fn split_mut(values: &mut [u32], at: usize) -> Result<(&mut [u32], &mut [u32]), EvmCoreError> {
    if at > values.len() {
        return Err(EvmCoreError::PrecompileWorkspaceTooSmall);
    }
    Ok(values.split_at_mut(at))
}

fn prefix_mut(values: &mut [u32], len: usize) -> Result<&mut [u32], EvmCoreError> {
    values
        .get_mut(..len)
        .ok_or(EvmCoreError::PrecompileWorkspaceTooSmall)
}

fn significant_len(values: &[u32]) -> usize {
    values
        .iter()
        .rposition(|limb| *limb != 0)
        .and_then(|index| index.checked_add(1))
        .unwrap_or(0)
}

fn load_segment_le(
    input: &[u8],
    offset: usize,
    len: usize,
    output: &mut [u32],
) -> Result<(), EvmCoreError> {
    output.fill(0);
    for index in 0..len {
        let low_index = len
            .checked_sub(1)
            .and_then(|last| last.checked_sub(index))
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        let limb_index = low_index / LIMB_BYTES;
        let shift = (low_index % LIMB_BYTES).saturating_mul(8);
        let byte = u32::from(virtual_byte(input, offset, index));
        let slot = output
            .get_mut(limb_index)
            .ok_or(EvmCoreError::PrecompileWorkspaceTooSmall)?;
        *slot |= byte
            .checked_shl(u32::try_from(shift).unwrap_or(u32::MAX))
            .unwrap_or(0);
    }
    Ok(())
}

fn reduce_virtual_segment(
    input: &[u8],
    offset: usize,
    len: usize,
    value: &mut [u32],
    modulus: &[u32],
) -> Result<(), EvmCoreError> {
    value.fill(0);
    for index in 0..len {
        let byte = virtual_byte(input, offset, index);
        for bit in 0..8_u8 {
            double_mod(value, modulus)?;
            if byte & (0x80_u8 >> bit) != 0 {
                add_one_mod(value, modulus)?;
            }
        }
    }
    Ok(())
}

fn square_mod(
    value: &mut [u32],
    modulus: &[u32],
    normalized_modulus: &mut [u32],
    dividend: &mut [u32],
) -> Result<(), EvmCoreError> {
    multiply_product(value, value, dividend)?;
    reduce_product(value, modulus, normalized_modulus, dividend)
}

fn multiply_mod(
    value: &mut [u32],
    right: &[u32],
    modulus: &[u32],
    normalized_modulus: &mut [u32],
    dividend: &mut [u32],
) -> Result<(), EvmCoreError> {
    multiply_product(value, right, dividend)?;
    reduce_product(value, modulus, normalized_modulus, dividend)
}

fn multiply_product(left: &[u32], right: &[u32], product: &mut [u32]) -> Result<(), EvmCoreError> {
    product.fill(0);
    for left_index in 0..left.len() {
        let mut carry = 0_u64;
        for right_index in 0..right.len() {
            let product_index = left_index
                .checked_add(right_index)
                .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
            let total = u64::from(limb(left, left_index)?)
                .wrapping_mul(u64::from(limb(right, right_index)?))
                .wrapping_add(u64::from(limb(product, product_index)?))
                .wrapping_add(carry);
            set_limb(product, product_index, low_limb(total))?;
            carry = total >> u32::BITS;
        }
        let carry_index = left_index
            .checked_add(right.len())
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        set_limb(product, carry_index, low_limb(carry))?;
    }
    Ok(())
}

fn reduce_product(
    output: &mut [u32],
    modulus: &[u32],
    normalized_modulus: &mut [u32],
    dividend: &mut [u32],
) -> Result<(), EvmCoreError> {
    let n = modulus.len();
    let product_len = n
        .checked_mul(2)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let shift = limb(modulus, n.saturating_sub(1))?.leading_zeros();
    normalize_copy(modulus, normalized_modulus, shift)?;
    normalize_dividend(dividend, product_len, shift)?;

    for quotient_index in (0..=n).rev() {
        divide_digit(dividend, normalized_modulus, quotient_index)?;
    }
    denormalize_remainder(dividend, output, shift)
}

fn normalize_copy(input: &[u32], output: &mut [u32], shift: u32) -> Result<(), EvmCoreError> {
    let mut carry = 0_u32;
    for index in 0..input.len() {
        let current = limb(input, index)?;
        let next = if shift == 0 {
            current
        } else {
            current.wrapping_shl(shift) | carry
        };
        set_limb(output, index, next)?;
        carry = if shift == 0 {
            0
        } else {
            current.wrapping_shr(u32::BITS.saturating_sub(shift))
        };
    }
    Ok(())
}

fn normalize_dividend(
    dividend: &mut [u32],
    product_len: usize,
    shift: u32,
) -> Result<(), EvmCoreError> {
    let mut carry = 0_u32;
    for index in 0..product_len {
        let current = limb(dividend, index)?;
        let next = if shift == 0 {
            current
        } else {
            current.wrapping_shl(shift) | carry
        };
        set_limb(dividend, index, next)?;
        carry = if shift == 0 {
            0
        } else {
            current.wrapping_shr(u32::BITS.saturating_sub(shift))
        };
    }
    set_limb(dividend, product_len, carry)
}

fn divide_digit(
    dividend: &mut [u32],
    divisor: &[u32],
    quotient_index: usize,
) -> Result<(), EvmCoreError> {
    let n = divisor.len();
    let top_index = quotient_index
        .checked_add(n)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let high = u64::from(limb(dividend, top_index)?);
    let low = u64::from(limb(dividend, top_index.saturating_sub(1))?);
    let divisor_high = u64::from(limb(divisor, n.saturating_sub(1))?);
    let numerator = (u128::from(high) << u32::BITS) | u128::from(low);
    let mut estimate = if high == divisor_high {
        LIMB_MASK
    } else {
        u64::try_from(
            numerator
                .checked_div(u128::from(divisor_high))
                .unwrap_or(u128::MAX),
        )
        .unwrap_or(LIMB_MASK)
    };
    let mut remainder = u64::try_from(
        numerator.saturating_sub(u128::from(estimate).saturating_mul(u128::from(divisor_high))),
    )
    .unwrap_or(u64::MAX);
    if n > 1 {
        let next_divisor = u64::from(limb(divisor, n.saturating_sub(2))?);
        let next_dividend = u64::from(limb(dividend, top_index.saturating_sub(2))?);
        while u128::from(estimate).saturating_mul(u128::from(next_divisor))
            > (u128::from(remainder) << u32::BITS) | u128::from(next_dividend)
        {
            estimate = estimate.saturating_sub(1);
            remainder = remainder.saturating_add(divisor_high);
            if remainder >= LIMB_BASE {
                break;
            }
        }
    }

    let underflow = subtract_product(dividend, divisor, quotient_index, estimate)?;
    if underflow {
        add_back(dividend, divisor, quotient_index)?;
    }
    Ok(())
}

fn subtract_product(
    dividend: &mut [u32],
    divisor: &[u32],
    offset: usize,
    estimate: u64,
) -> Result<bool, EvmCoreError> {
    let mut carry = 0_u64;
    let mut borrow = 0_u64;
    for index in 0..divisor.len() {
        let product = estimate
            .wrapping_mul(u64::from(limb(divisor, index)?))
            .wrapping_add(carry);
        carry = product >> u32::BITS;
        let subtrahend = (product & LIMB_MASK).saturating_add(borrow);
        let target_index = offset
            .checked_add(index)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        let current = u64::from(limb(dividend, target_index)?);
        let (next, next_borrow) = subtract_limb(current, subtrahend);
        set_limb(dividend, target_index, low_limb(next))?;
        borrow = next_borrow;
    }
    let top_index = offset
        .checked_add(divisor.len())
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let current = u64::from(limb(dividend, top_index)?);
    let (next, underflow) = subtract_limb(current, carry.saturating_add(borrow));
    set_limb(dividend, top_index, low_limb(next))?;
    Ok(underflow != 0)
}

fn subtract_limb(current: u64, subtrahend: u64) -> (u64, u64) {
    if current >= subtrahend {
        (current.saturating_sub(subtrahend), 0)
    } else {
        (
            LIMB_BASE.saturating_add(current).saturating_sub(subtrahend),
            1,
        )
    }
}

fn add_back(dividend: &mut [u32], divisor: &[u32], offset: usize) -> Result<(), EvmCoreError> {
    let mut carry = 0_u64;
    for index in 0..divisor.len() {
        let target_index = offset
            .checked_add(index)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        let sum = u64::from(limb(dividend, target_index)?)
            .saturating_add(u64::from(limb(divisor, index)?))
            .saturating_add(carry);
        set_limb(dividend, target_index, low_limb(sum))?;
        carry = sum >> u32::BITS;
    }
    let top_index = offset
        .checked_add(divisor.len())
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let top = u64::from(limb(dividend, top_index)?).saturating_add(carry);
    set_limb(dividend, top_index, low_limb(top))
}

fn denormalize_remainder(
    dividend: &[u32],
    output: &mut [u32],
    shift: u32,
) -> Result<(), EvmCoreError> {
    for index in 0..output.len() {
        let current = limb(dividend, index)?;
        let next = if shift == 0 {
            current
        } else {
            let high = limb(dividend, index.saturating_add(1))?;
            current.wrapping_shr(shift) | high.wrapping_shl(u32::BITS.saturating_sub(shift))
        };
        set_limb(output, index, next)?;
    }
    Ok(())
}

fn double_mod(value: &mut [u32], modulus: &[u32]) -> Result<(), EvmCoreError> {
    let mut carry = 0_u64;
    for index in 0..value.len() {
        let doubled = u64::from(limb(value, index)?)
            .saturating_mul(2)
            .saturating_add(carry);
        set_limb(value, index, low_limb(doubled))?;
        carry = doubled >> u32::BITS;
    }
    if carry != 0 || compare(value, modulus) != Ordering::Less {
        subtract_modulus(value, modulus)?;
    }
    Ok(())
}

fn add_one_mod(value: &mut [u32], modulus: &[u32]) -> Result<(), EvmCoreError> {
    let mut carry = 1_u64;
    for index in 0..value.len() {
        let sum = u64::from(limb(value, index)?).saturating_add(carry);
        set_limb(value, index, low_limb(sum))?;
        carry = sum >> u32::BITS;
        if carry == 0 {
            break;
        }
    }
    if carry != 0 || compare(value, modulus) != Ordering::Less {
        subtract_modulus(value, modulus)?;
    }
    Ok(())
}

fn set_one_mod(value: &mut [u32], modulus: &[u32]) -> Result<(), EvmCoreError> {
    value.fill(0);
    set_limb(value, 0, 1)?;
    if compare(value, modulus) != Ordering::Less {
        subtract_modulus(value, modulus)?;
    }
    Ok(())
}

fn subtract_modulus(value: &mut [u32], modulus: &[u32]) -> Result<(), EvmCoreError> {
    let mut borrow = 0_u64;
    for index in 0..value.len() {
        let subtrahend = u64::from(limb(modulus, index)?).saturating_add(borrow);
        let current = u64::from(limb(value, index)?);
        let (next, next_borrow) = subtract_limb(current, subtrahend);
        set_limb(value, index, low_limb(next))?;
        borrow = next_borrow;
    }
    Ok(())
}

fn compare(left: &[u32], right: &[u32]) -> Ordering {
    for (left_limb, right_limb) in left.iter().rev().zip(right.iter().rev()) {
        match left_limb.cmp(right_limb) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }
    Ordering::Equal
}

fn write_result_be(result: &[u32], output: &mut [u8]) -> Result<(), EvmCoreError> {
    let output_len = output.len();
    for (index, slot) in output.iter_mut().enumerate() {
        let low_index = output_len
            .checked_sub(1)
            .and_then(|last| last.checked_sub(index))
            .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
        let limb_index = low_index / LIMB_BYTES;
        let shift = (low_index % LIMB_BYTES).saturating_mul(8);
        let value = result
            .get(limb_index)
            .copied()
            .unwrap_or(0)
            .wrapping_shr(u32::try_from(shift).unwrap_or(0));
        *slot = u8::try_from(value & 0xff).unwrap_or(0);
    }
    Ok(())
}

fn virtual_byte(input: &[u8], offset: usize, index: usize) -> u8 {
    offset
        .checked_add(index)
        .and_then(|position| input.get(position))
        .copied()
        .unwrap_or(0)
}

fn limb(values: &[u32], index: usize) -> Result<u32, EvmCoreError> {
    values
        .get(index)
        .copied()
        .ok_or(EvmCoreError::PrecompileWorkspaceTooSmall)
}

fn set_limb(values: &mut [u32], index: usize, value: u32) -> Result<(), EvmCoreError> {
    let slot = values
        .get_mut(index)
        .ok_or(EvmCoreError::PrecompileWorkspaceTooSmall)?;
    *slot = value;
    Ok(())
}

fn low_limb(value: u64) -> u32 {
    u32::try_from(value & LIMB_MASK).unwrap_or(0)
}
