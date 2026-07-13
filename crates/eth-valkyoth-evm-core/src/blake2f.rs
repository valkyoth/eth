use crate::{EvmCoreError, EvmGasMeter, EvmPrecompileKind, EvmPrecompilePlan};

/// Byte length of the EIP-152 BLAKE2F precompile input.
pub const EVM_BLAKE2F_INPUT_BYTES: usize = 213;
/// Byte length of the EIP-152 BLAKE2F precompile output.
pub const EVM_BLAKE2F_OUTPUT_BYTES: usize = 64;

const ROUNDS_BYTES: core::ops::Range<usize> = 0..4;
const H_BYTES: core::ops::Range<usize> = 4..68;
const M_BYTES: core::ops::Range<usize> = 68..196;
const T0_BYTES: core::ops::Range<usize> = 196..204;
const T1_BYTES: core::ops::Range<usize> = 204..212;
const FINAL_FLAG_INDEX: usize = 212;

const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

const SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

impl EvmPrecompilePlan {
    /// Executes the dependency-free EIP-152 BLAKE2F precompile into `output`.
    pub fn execute_blake2f(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Blake2F {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        let cost = self.checked_execution_cost(input)?;
        gas.charge(cost)?;
        execute_blake2f(input, output)
    }
}

/// Executes the dependency-free EIP-152 BLAKE2F precompile.
pub(crate) fn execute_blake2f(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    let parsed = Blake2FInput::parse(input)?;
    let target = output
        .get_mut(..EVM_BLAKE2F_OUTPUT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    let result = compress(parsed)?;
    for (chunk, word) in target.chunks_exact_mut(8).zip(result) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    Ok(EVM_BLAKE2F_OUTPUT_BYTES)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Blake2FInput {
    rounds: u32,
    h: [u64; 8],
    m: [u64; 16],
    t0: u64,
    t1: u64,
    final_block: bool,
}

impl Blake2FInput {
    fn parse(input: &[u8]) -> Result<Self, EvmCoreError> {
        if input.len() != EVM_BLAKE2F_INPUT_BYTES {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        let final_block = match input.get(FINAL_FLAG_INDEX).copied() {
            Some(0) => false,
            Some(1) => true,
            _ => return Err(EvmCoreError::PrecompileInvalidInputLength),
        };
        Ok(Self {
            rounds: u32::from_be_bytes(read_array(input, ROUNDS_BYTES)?),
            h: read_words(input, H_BYTES)?,
            m: read_words(input, M_BYTES)?,
            t0: u64::from_le_bytes(read_array(input, T0_BYTES)?),
            t1: u64::from_le_bytes(read_array(input, T1_BYTES)?),
            final_block,
        })
    }
}

fn compress(input: Blake2FInput) -> Result<[u64; 8], EvmCoreError> {
    let mut v = [0u64; 16];
    for (slot, value) in v.iter_mut().take(8).zip(input.h) {
        *slot = value;
    }
    for (slot, value) in v.iter_mut().skip(8).zip(IV) {
        *slot = value;
    }
    let counter_low = lane(&v, 12) ^ input.t0;
    set_lane(&mut v, 12, counter_low);
    let counter_high = lane(&v, 13) ^ input.t1;
    set_lane(&mut v, 13, counter_high);
    if input.final_block {
        let final_word = !lane(&v, 14);
        set_lane(&mut v, 14, final_word);
    }
    let rounds = usize::try_from(input.rounds).map_err(|_| EvmCoreError::PrecompileGasOverflow)?;
    for schedule in SIGMA.iter().copied().cycle().take(rounds) {
        round_function(&mut v, &input.m, schedule);
    }
    let mut h = input.h;
    let (low, high) = v.split_at(8);
    for ((out, left), right) in h.iter_mut().zip(low.iter()).zip(high.iter()) {
        *out ^= *left ^ *right;
    }
    Ok(h)
}

fn round_function(v: &mut [u64; 16], m: &[u64; 16], s: [usize; 16]) {
    let [
        s0,
        s1,
        s2,
        s3,
        s4,
        s5,
        s6,
        s7,
        s8,
        s9,
        s10,
        s11,
        s12,
        s13,
        s14,
        s15,
    ] = s;
    mix(v, message(m, s0), message(m, s1), 0, 4, 8, 12);
    mix(v, message(m, s2), message(m, s3), 1, 5, 9, 13);
    mix(v, message(m, s4), message(m, s5), 2, 6, 10, 14);
    mix(v, message(m, s6), message(m, s7), 3, 7, 11, 15);
    mix(v, message(m, s8), message(m, s9), 0, 5, 10, 15);
    mix(v, message(m, s10), message(m, s11), 1, 6, 11, 12);
    mix(v, message(m, s12), message(m, s13), 2, 7, 8, 13);
    mix(v, message(m, s14), message(m, s15), 3, 4, 9, 14);
}

fn mix(v: &mut [u64; 16], x: u64, y: u64, a: usize, b: usize, c: usize, d: usize) {
    let next_a = lane(v, a).wrapping_add(lane(v, b)).wrapping_add(x);
    set_lane(v, a, next_a);
    let next_d = (lane(v, d) ^ lane(v, a)).rotate_right(32);
    set_lane(v, d, next_d);
    let next_c = lane(v, c).wrapping_add(lane(v, d));
    set_lane(v, c, next_c);
    let next_b = (lane(v, b) ^ lane(v, c)).rotate_right(24);
    set_lane(v, b, next_b);
    let next_a = lane(v, a).wrapping_add(lane(v, b)).wrapping_add(y);
    set_lane(v, a, next_a);
    let next_d = (lane(v, d) ^ lane(v, a)).rotate_right(16);
    set_lane(v, d, next_d);
    let next_c = lane(v, c).wrapping_add(lane(v, d));
    set_lane(v, c, next_c);
    let next_b = (lane(v, b) ^ lane(v, c)).rotate_right(63);
    set_lane(v, b, next_b);
}

fn message(m: &[u64; 16], index: usize) -> u64 {
    match m.get(index) {
        Some(value) => *value,
        None => 0,
    }
}

fn lane(v: &[u64; 16], index: usize) -> u64 {
    match v.get(index) {
        Some(value) => *value,
        None => 0,
    }
}

fn set_lane(v: &mut [u64; 16], index: usize, value: u64) {
    if let Some(slot) = v.get_mut(index) {
        *slot = value;
    }
}

fn read_words<const N: usize>(
    input: &[u8],
    range: core::ops::Range<usize>,
) -> Result<[u64; N], EvmCoreError> {
    let bytes = input
        .get(range)
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)?;
    let mut words = [0u64; N];
    for (index, chunk) in bytes.chunks_exact(8).enumerate() {
        if let Some(slot) = words.get_mut(index) {
            *slot = u64::from_le_bytes(
                chunk
                    .try_into()
                    .map_err(|_| EvmCoreError::PrecompileInvalidInputLength)?,
            );
        }
    }
    Ok(words)
}

fn read_array<const N: usize>(
    input: &[u8],
    range: core::ops::Range<usize>,
) -> Result<[u8; N], EvmCoreError> {
    input
        .get(range)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(EvmCoreError::PrecompileInvalidInputLength)
}
