use crate::{EVM_MAX_GAS_LIMIT, EvmCoreError, EvmFork, EvmGas};

use super::{EvmModExpInput, PAYLOAD_OFFSET, WORD_BYTES};

const EIP198_QUAD_DIVISOR: u128 = 20;
const EIP2565_QUAD_DIVISOR: u128 = 3;
const EIP2565_MIN_GAS: u64 = 200;
const UNPAYABLE_GAS: u64 = EVM_MAX_GAS_LIMIT + 1;
const WORD_BYTES_U8: u8 = 32;

pub(super) fn cost(fork: EvmFork, input: &[u8]) -> Result<EvmGas, EvmCoreError> {
    let parsed = super::parse_modexp_input(input)?;
    let max_len = parsed
        .base_len()
        .max(parsed.modulus_len())
        .saturating_u128();
    let adjusted_exponent = adjusted_exponent_len(input, parsed);
    let gas = if fork.get() >= EvmFork::BERLIN.get() {
        eip2565_gas(max_len, adjusted_exponent)
    } else {
        eip198_gas(max_len, adjusted_exponent)
    };
    Ok(EvmGas::new(cap_unpayable(gas)))
}

fn adjusted_exponent_len(input: &[u8], parsed: EvmModExpInput) -> u128 {
    if parsed.exponent_len().is_zero() {
        return 0;
    }
    let exponent_offset = parsed
        .base_len()
        .try_to_usize()
        .ok()
        .and_then(|length| PAYLOAD_OFFSET.checked_add(length));
    let head_len = if parsed.exponent_len().at_most(WORD_BYTES_U8) {
        parsed.exponent_len().try_to_usize().unwrap_or(WORD_BYTES)
    } else {
        WORD_BYTES
    };
    let highest = exponent_offset
        .and_then(|offset| highest_bit_index_in_field(input, offset, head_len))
        .map_or(0, u128::from);
    if parsed.exponent_len().at_most(WORD_BYTES_U8) {
        return highest;
    }
    parsed
        .exponent_len()
        .saturating_u128()
        .saturating_sub(WORD_BYTES as u128)
        .saturating_mul(8)
        .saturating_add(highest)
}

fn eip198_gas(max_len: u128, adjusted_exponent: u128) -> u128 {
    eip198_complexity(max_len).saturating_mul(adjusted_exponent.max(1)) / EIP198_QUAD_DIVISOR
}

fn eip2565_gas(max_len: u128, adjusted_exponent: u128) -> u128 {
    let words = max_len.saturating_add(7) / 8;
    words
        .saturating_mul(words)
        .saturating_mul(adjusted_exponent.max(1))
        .checked_div(EIP2565_QUAD_DIVISOR)
        .unwrap_or(u128::MAX)
        .max(u128::from(EIP2565_MIN_GAS))
}

fn eip198_complexity(max_len: u128) -> u128 {
    if max_len <= 64 {
        return max_len.saturating_mul(max_len);
    }
    if max_len <= 1_024 {
        return max_len
            .saturating_mul(max_len)
            .checked_div(4)
            .unwrap_or(u128::MAX)
            .saturating_add(96_u128.saturating_mul(max_len))
            .saturating_sub(3_072);
    }
    max_len
        .saturating_mul(max_len)
        .checked_div(16)
        .unwrap_or(u128::MAX)
        .saturating_add(480_u128.saturating_mul(max_len))
        .saturating_sub(199_680)
}

fn cap_unpayable(gas: u128) -> u64 {
    if gas > u128::from(EVM_MAX_GAS_LIMIT) {
        UNPAYABLE_GAS
    } else {
        u64::try_from(gas).unwrap_or(UNPAYABLE_GAS)
    }
}

fn highest_bit_index_in_field(input: &[u8], offset: usize, len: usize) -> Option<u8> {
    for index in 0..len {
        let byte = input
            .get(offset.saturating_add(index))
            .copied()
            .unwrap_or(0);
        if byte == 0 {
            continue;
        }
        let byte_index = u8::try_from(index).ok()?;
        let leading = u8::try_from(byte.leading_zeros()).ok()?;
        let width = u8::try_from(len).ok()?;
        let byte_offset = width.checked_sub(1)?.checked_sub(byte_index)?;
        let bit_offset = 7_u8.checked_sub(leading)?;
        return byte_offset.checked_mul(8)?.checked_add(bit_offset);
    }
    None
}
