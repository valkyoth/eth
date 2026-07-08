use crate::{
    EVM_PRECOMPILE_INPUT_LIMIT, EvmCoreError, EvmFork, EvmGas, EvmPrecompileKind, EvmPrecompilePlan,
};

/// Canonical ModExp header byte length: base, exponent, and modulus lengths.
pub const EVM_MODEXP_HEADER_BYTES: usize = 96;
/// Maximum operand bytes executed by this release's no-alloc ModExp engine.
pub const EVM_MODEXP_MAX_OPERAND_BYTES: usize = 64;

const WORD_BYTES: usize = 32;
const LENGTH_BASE_OFFSET: usize = 0;
const LENGTH_EXPONENT_OFFSET: usize = 32;
const LENGTH_MODULUS_OFFSET: usize = 64;
const PAYLOAD_OFFSET: usize = 96;
const EIP198_QUAD_DIVISOR: u128 = 20;
const EIP2565_QUAD_DIVISOR: u128 = 3;
const EIP2565_MIN_GAS: u64 = 200;

/// Parsed ModExp input lengths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmModExpInput {
    base_len: usize,
    exponent_len: usize,
    modulus_len: usize,
}

impl EvmModExpInput {
    /// Returns the declared base byte length.
    #[must_use]
    pub const fn base_len(self) -> usize {
        self.base_len
    }

    /// Returns the declared exponent byte length.
    #[must_use]
    pub const fn exponent_len(self) -> usize {
        self.exponent_len
    }

    /// Returns the declared modulus byte length.
    #[must_use]
    pub const fn modulus_len(self) -> usize {
        self.modulus_len
    }
}

impl EvmPrecompilePlan {
    /// Executes the bounded first-party ModExp precompile into `output`.
    pub fn execute_modexp(self, input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Modexp {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_modexp(input, output)
    }
}

/// Parses ModExp input lengths using EIP-198 right-padding semantics.
pub fn parse_modexp_input(input: &[u8]) -> Result<EvmModExpInput, EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    let parsed = EvmModExpInput {
        base_len: checked_len_word(input, LENGTH_BASE_OFFSET)?,
        exponent_len: checked_len_word(input, LENGTH_EXPONENT_OFFSET)?,
        modulus_len: checked_len_word(input, LENGTH_MODULUS_OFFSET)?,
    };
    validate_operand_len(parsed.base_len)?;
    validate_operand_len(parsed.exponent_len)?;
    validate_operand_len(parsed.modulus_len)?;
    Ok(parsed)
}

/// Executes the bounded first-party ModExp precompile.
pub fn execute_modexp(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    let parsed = parse_modexp_input(input)?;
    let target = output
        .get_mut(..parsed.modulus_len)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    if parsed.modulus_len == 0 {
        return Ok(0);
    }

    let layout = ModExpLayout::new(parsed)?;
    let mut modulus = [0u8; EVM_MODEXP_MAX_OPERAND_BYTES];
    copy_segment(
        input,
        layout.modulus_offset,
        parsed.modulus_len,
        &mut modulus,
    );
    if is_zero(&modulus, parsed.modulus_len) {
        target.fill(0);
        return Ok(parsed.modulus_len);
    }

    let mut base = [0u8; EVM_MODEXP_MAX_OPERAND_BYTES];
    reduce_segment(
        input,
        layout.base_offset,
        parsed.base_len,
        &modulus,
        parsed.modulus_len,
        &mut base,
    );

    let mut result = [0u8; EVM_MODEXP_MAX_OPERAND_BYTES];
    one_mod(&modulus, parsed.modulus_len, &mut result);
    for bit_index in 0..parsed.exponent_len.saturating_mul(8) {
        square_mod(&mut result, &modulus, parsed.modulus_len);
        if exponent_bit(input, layout.exponent_offset, bit_index) {
            mul_mod_assign(&mut result, &base, &modulus, parsed.modulus_len);
        }
    }

    let result = result
        .get(..parsed.modulus_len)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    target.copy_from_slice(result);
    Ok(parsed.modulus_len)
}

pub(crate) fn modexp_gas_cost(fork: EvmFork, input: &[u8]) -> Result<EvmGas, EvmCoreError> {
    let parsed = parse_modexp_input(input)?;
    let layout = ModExpLayout::new(parsed)?;
    let adjusted_exponent =
        adjusted_exponent_len(input, layout.exponent_offset, parsed.exponent_len)?;
    let gas = if fork.get() >= EvmFork::BERLIN.get() {
        eip2565_gas(parsed, adjusted_exponent)?
    } else {
        eip198_gas(parsed, adjusted_exponent)?
    };
    Ok(EvmGas::new(gas))
}

#[derive(Clone, Copy)]
struct ModExpLayout {
    base_offset: usize,
    exponent_offset: usize,
    modulus_offset: usize,
}

impl ModExpLayout {
    fn new(parsed: EvmModExpInput) -> Result<Self, EvmCoreError> {
        let exponent_offset = PAYLOAD_OFFSET
            .checked_add(parsed.base_len)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        let modulus_offset = exponent_offset
            .checked_add(parsed.exponent_len)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        modulus_offset
            .checked_add(parsed.modulus_len)
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
        Ok(Self {
            base_offset: PAYLOAD_OFFSET,
            exponent_offset,
            modulus_offset,
        })
    }
}

fn checked_len_word(input: &[u8], offset: usize) -> Result<usize, EvmCoreError> {
    let word = padded_word(input, offset);
    let max_bytes = core::mem::size_of::<usize>();
    let high_len = WORD_BYTES
        .checked_sub(max_bytes)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    let high = word
        .get(..high_len)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    if high.iter().any(|byte| *byte != 0) {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    let mut value = 0usize;
    let low = word
        .get(high_len..)
        .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    for byte in low {
        value = value
            .checked_mul(256)
            .and_then(|current| current.checked_add(usize::from(*byte)))
            .ok_or(EvmCoreError::PrecompileInputTooLarge)?;
    }
    Ok(value)
}

fn validate_operand_len(len: usize) -> Result<(), EvmCoreError> {
    if len <= EVM_MODEXP_MAX_OPERAND_BYTES {
        return Ok(());
    }
    Err(EvmCoreError::PrecompileInputTooLarge)
}

fn adjusted_exponent_len(
    input: &[u8],
    exponent_offset: usize,
    exponent_len: usize,
) -> Result<u128, EvmCoreError> {
    if exponent_len == 0 {
        return Ok(0);
    }
    let exponent_head = padded_word(input, exponent_offset);
    let highest = highest_bit_index(&exponent_head);
    if exponent_len <= WORD_BYTES {
        return Ok(highest.map_or(0, u128::from));
    }
    let tail_bits = exponent_len
        .checked_sub(WORD_BYTES)
        .and_then(|len| len.checked_mul(8))
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let tail_bits = u128::try_from(tail_bits).map_err(|_| EvmCoreError::PrecompileGasOverflow)?;
    tail_bits
        .checked_add(highest.map_or(0, u128::from))
        .ok_or(EvmCoreError::PrecompileGasOverflow)
}

fn eip198_gas(parsed: EvmModExpInput, adjusted_exponent: u128) -> Result<u64, EvmCoreError> {
    let max_len = parsed.base_len.max(parsed.modulus_len);
    let complexity = eip198_complexity(max_len)?;
    gas_u64(complexity.saturating_mul(adjusted_exponent.max(1)) / EIP198_QUAD_DIVISOR)
}

fn eip2565_gas(parsed: EvmModExpInput, adjusted_exponent: u128) -> Result<u64, EvmCoreError> {
    let max_len = parsed.base_len.max(parsed.modulus_len);
    let words = max_len
        .checked_add(7)
        .and_then(|len| len.checked_div(8))
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let words = u128::try_from(words).map_err(|_| EvmCoreError::PrecompileGasOverflow)?;
    let complexity = words
        .checked_mul(words)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let gas = complexity
        .checked_mul(adjusted_exponent.max(1))
        .ok_or(EvmCoreError::PrecompileGasOverflow)?
        / EIP2565_QUAD_DIVISOR;
    gas_u64(gas.max(u128::from(EIP2565_MIN_GAS)))
}

fn eip198_complexity(max_len: usize) -> Result<u128, EvmCoreError> {
    let x = u128::try_from(max_len).map_err(|_| EvmCoreError::PrecompileGasOverflow)?;
    if max_len <= 64 {
        return x.checked_mul(x).ok_or(EvmCoreError::PrecompileGasOverflow);
    }
    if max_len <= 1024 {
        return x
            .checked_mul(x)
            .map(|square| square / 4)
            .and_then(|square| square.checked_add(96u128.saturating_mul(x)))
            .and_then(|value| value.checked_sub(3072))
            .ok_or(EvmCoreError::PrecompileGasOverflow);
    }
    x.checked_mul(x)
        .map(|square| square / 16)
        .and_then(|square| square.checked_add(480u128.saturating_mul(x)))
        .and_then(|value| value.checked_sub(199_680))
        .ok_or(EvmCoreError::PrecompileGasOverflow)
}

fn gas_u64(value: u128) -> Result<u64, EvmCoreError> {
    u64::try_from(value).map_err(|_| EvmCoreError::PrecompileGasOverflow)
}

fn highest_bit_index(bytes: &[u8; WORD_BYTES]) -> Option<u8> {
    for (byte_index, byte) in bytes.iter().enumerate() {
        if *byte == 0 {
            continue;
        }
        let byte_index = u8::try_from(byte_index).ok()?;
        let leading = u8::try_from(byte.leading_zeros()).ok()?;
        let byte_offset = 31u8.checked_sub(byte_index)?;
        let bit_offset = 7u8.checked_sub(leading)?;
        return byte_offset.checked_mul(8)?.checked_add(bit_offset);
    }
    None
}

fn reduce_segment(
    input: &[u8],
    offset: usize,
    len: usize,
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus_len: usize,
    output: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
) {
    output.fill(0);
    for index in 0..len {
        let byte = input
            .get(offset.saturating_add(index))
            .copied()
            .unwrap_or(0);
        for bit in 0..8 {
            double_mod(output, modulus, modulus_len);
            if byte & (0x80 >> bit) != 0 {
                add_one_mod(output, modulus, modulus_len);
            }
        }
    }
}

fn square_mod(
    value: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) {
    let left = *value;
    mul_mod(value, &left, &left, modulus, len);
}

fn mul_mod_assign(
    value: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    rhs: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) {
    let left = *value;
    mul_mod(value, &left, rhs, modulus, len);
}

fn mul_mod(
    output: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    left: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    right: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) {
    output.fill(0);
    let mut addend = *left;
    for bit in 0..len.saturating_mul(8) {
        if bit_from_low(right, len, bit) {
            add_mod(output, &addend, modulus, len);
        }
        double_mod(&mut addend, modulus, len);
    }
}

fn add_mod(
    value: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    addend: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) {
    let mut carry = 0u16;
    let Some(value_slice) = value.get_mut(..len) else {
        return;
    };
    let Some(addend_slice) = addend.get(..len) else {
        return;
    };
    for (slot, addend_byte) in value_slice.iter_mut().rev().zip(addend_slice.iter().rev()) {
        let sum = u16::from(*slot)
            .saturating_add(u16::from(*addend_byte))
            .saturating_add(carry);
        *slot = low_u8(sum & 0xff);
        carry = sum >> 8;
    }
    if carry != 0 || cmp_be(value, modulus, len).is_ge() {
        sub_modulus(value, modulus, len);
    }
}

fn double_mod(
    value: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) {
    let copy = *value;
    add_mod(value, &copy, modulus, len);
}

fn add_one_mod(
    value: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) {
    let Some(value_slice) = value.get_mut(..len) else {
        return;
    };
    for slot in value_slice.iter_mut().rev() {
        let (next, carry) = slot.overflowing_add(1);
        *slot = next;
        if !carry {
            break;
        }
    }
    if cmp_be(value, modulus, len).is_ge() {
        sub_modulus(value, modulus, len);
    }
}

fn one_mod(
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
    output: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
) {
    output.fill(0);
    if let Some(slot) = output.get_mut(len.saturating_sub(1)) {
        *slot = 1;
    }
    if cmp_be(output, modulus, len).is_ge() {
        sub_modulus(output, modulus, len);
    }
}

fn sub_modulus(
    value: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    modulus: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) {
    let mut borrow = 0u16;
    let Some(value_slice) = value.get_mut(..len) else {
        return;
    };
    let Some(modulus_slice) = modulus.get(..len) else {
        return;
    };
    for (slot, modulus_byte) in value_slice.iter_mut().rev().zip(modulus_slice.iter().rev()) {
        let rhs = u16::from(*modulus_byte).saturating_add(borrow);
        let lhs = u16::from(*slot);
        if lhs >= rhs {
            *slot = low_u8(lhs.saturating_sub(rhs));
            borrow = 0;
        } else {
            *slot = low_u8(256u16.saturating_add(lhs).saturating_sub(rhs));
            borrow = 1;
        }
    }
}

fn cmp_be(
    left: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    right: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES],
    len: usize,
) -> core::cmp::Ordering {
    let left = left.get(..len).unwrap_or(&[]);
    let right = right.get(..len).unwrap_or(&[]);
    left.cmp(right)
}

fn exponent_bit(input: &[u8], offset: usize, bit_index: usize) -> bool {
    let byte = input
        .get(offset.saturating_add(bit_index / 8))
        .copied()
        .unwrap_or(0);
    byte & (0x80 >> (bit_index % 8)) != 0
}

fn bit_from_low(bytes: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES], len: usize, bit_index: usize) -> bool {
    let byte_index = len.saturating_sub(1).saturating_sub(bit_index / 8);
    bytes
        .get(byte_index)
        .is_some_and(|byte| byte & (1 << (bit_index % 8)) != 0)
}

fn copy_segment(
    input: &[u8],
    offset: usize,
    len: usize,
    output: &mut [u8; EVM_MODEXP_MAX_OPERAND_BYTES],
) {
    output.fill(0);
    let Some(output) = output.get_mut(..len) else {
        return;
    };
    for (index, slot) in output.iter_mut().enumerate() {
        *slot = input
            .get(offset.saturating_add(index))
            .copied()
            .unwrap_or(0);
    }
}

fn padded_word(input: &[u8], offset: usize) -> [u8; WORD_BYTES] {
    let mut output = [0u8; WORD_BYTES];
    for (index, slot) in output.iter_mut().enumerate() {
        *slot = input
            .get(offset.saturating_add(index))
            .copied()
            .unwrap_or(0);
    }
    output
}

fn is_zero(bytes: &[u8; EVM_MODEXP_MAX_OPERAND_BYTES], len: usize) -> bool {
    bytes
        .get(..len)
        .is_none_or(|bytes| bytes.iter().all(|byte| *byte == 0))
}

fn low_u8(value: u16) -> u8 {
    u8::try_from(value).unwrap_or(0)
}
