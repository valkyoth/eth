const BLOCK_BYTES: usize = 64;
const DIGEST_BYTES: usize = 20;
const LENGTH_BYTES: usize = 8;

const INITIAL_STATE: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

const LEFT_ORDER: [usize; 80] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9, 5,
    2, 14, 11, 8, 3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10, 0, 8, 12, 4,
    13, 3, 7, 15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13,
];

const RIGHT_ORDER: [usize; 80] = [
    5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8, 12,
    4, 9, 1, 2, 15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4, 1, 3, 11, 15, 0, 5,
    12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11,
];

const LEFT_ROTATIONS: [u32; 80] = [
    11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8, 13, 11, 9, 7, 15, 7, 12, 15,
    9, 11, 7, 13, 12, 11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11, 12, 14, 15, 14,
    15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11, 8, 5, 6,
];

const RIGHT_ROTATIONS: [u32; 80] = [
    8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13, 15, 7, 12, 8, 9, 11, 7, 7, 12,
    7, 6, 15, 13, 11, 9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5, 8, 11, 14, 14,
    6, 14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13, 11, 11,
];

pub(crate) fn digest(input: &[u8]) -> [u8; DIGEST_BYTES] {
    let mut state = INITIAL_STATE;
    let mut chunks = input.chunks_exact(BLOCK_BYTES);
    for chunk in &mut chunks {
        compress(&mut state, chunk);
    }
    finalize(&mut state, chunks.remainder(), input.len());
    state_to_bytes(state)
}

fn finalize(state: &mut [u32; 5], remainder: &[u8], input_len: usize) {
    let mut block = [0u8; BLOCK_BYTES];
    let remainder_len = remainder.len();
    if let Some(target) = block.get_mut(..remainder_len) {
        target.copy_from_slice(remainder);
    }
    if let Some(marker) = block.get_mut(remainder_len) {
        *marker = 0x80;
    }

    let needs_extra_block = remainder_len > BLOCK_BYTES - LENGTH_BYTES - 1;
    if needs_extra_block {
        compress(state, &block);
        block.fill(0);
    }

    let bit_len = u64::try_from(input_len)
        .ok()
        .and_then(|len| len.checked_mul(8))
        .unwrap_or(0);
    let len_bytes = bit_len.to_le_bytes();
    if let Some(target) = block.get_mut(BLOCK_BYTES - LENGTH_BYTES..) {
        target.copy_from_slice(&len_bytes);
    }
    compress(state, &block);
}

fn compress(state: &mut [u32; 5], block: &[u8]) {
    let mut words = [0u32; 16];
    for (word, chunk) in words.iter_mut().zip(block.chunks_exact(4)) {
        if let Ok(bytes) = <[u8; 4]>::try_from(chunk) {
            *word = u32::from_le_bytes(bytes);
        }
    }

    let mut left = WorkingState::from_state(state);
    let mut right = left;

    for round in 0..80 {
        left = left.step(
            left_function(round, left.b, left.c, left.d),
            word_at(&words, order_at(&LEFT_ORDER, round)),
            left_constant(round),
            rotation_at(&LEFT_ROTATIONS, round),
        );
        right = right.step(
            right_function(round, right.b, right.c, right.d),
            word_at(&words, order_at(&RIGHT_ORDER, round)),
            right_constant(round),
            rotation_at(&RIGHT_ROTATIONS, round),
        );
    }

    let h0 = word_at(state, 0);
    let h1 = word_at(state, 1);
    let h2 = word_at(state, 2);
    let h3 = word_at(state, 3);
    let h4 = word_at(state, 4);

    set_word(state, 0, h1.wrapping_add(left.c).wrapping_add(right.d));
    set_word(state, 1, h2.wrapping_add(left.d).wrapping_add(right.e));
    set_word(state, 2, h3.wrapping_add(left.e).wrapping_add(right.a));
    set_word(state, 3, h4.wrapping_add(left.a).wrapping_add(right.b));
    set_word(state, 4, h0.wrapping_add(left.b).wrapping_add(right.c));
}

#[derive(Clone, Copy)]
struct WorkingState {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
}

impl WorkingState {
    fn from_state(state: &[u32; 5]) -> Self {
        Self {
            a: word_at(state, 0),
            b: word_at(state, 1),
            c: word_at(state, 2),
            d: word_at(state, 3),
            e: word_at(state, 4),
        }
    }

    fn step(self, function: u32, word: u32, constant: u32, rotation: u32) -> Self {
        let next = self
            .a
            .wrapping_add(function)
            .wrapping_add(word)
            .wrapping_add(constant)
            .rotate_left(rotation)
            .wrapping_add(self.e);
        Self {
            a: self.e,
            b: next,
            c: self.b,
            d: self.c.rotate_left(10),
            e: self.d,
        }
    }
}

fn left_function(round: usize, x: u32, y: u32, z: u32) -> u32 {
    match round {
        0..=15 => x ^ y ^ z,
        16..=31 => (x & y) | (!x & z),
        32..=47 => (x | !y) ^ z,
        48..=63 => (x & z) | (y & !z),
        _ => x ^ (y | !z),
    }
}

fn right_function(round: usize, x: u32, y: u32, z: u32) -> u32 {
    match round {
        0..=15 => x ^ (y | !z),
        16..=31 => (x & z) | (y & !z),
        32..=47 => (x | !y) ^ z,
        48..=63 => (x & y) | (!x & z),
        _ => x ^ y ^ z,
    }
}

fn left_constant(round: usize) -> u32 {
    match round {
        0..=15 => 0x00000000,
        16..=31 => 0x5a827999,
        32..=47 => 0x6ed9eba1,
        48..=63 => 0x8f1bbcdc,
        _ => 0xa953fd4e,
    }
}

fn right_constant(round: usize) -> u32 {
    match round {
        0..=15 => 0x50a28be6,
        16..=31 => 0x5c4dd124,
        32..=47 => 0x6d703ef3,
        48..=63 => 0x7a6d76e9,
        _ => 0x00000000,
    }
}

fn state_to_bytes(state: [u32; 5]) -> [u8; DIGEST_BYTES] {
    let mut output = [0u8; DIGEST_BYTES];
    for (chunk, word) in output.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    output
}

fn word_at(words: &[u32], index: usize) -> u32 {
    match words.get(index) {
        Some(word) => *word,
        None => 0,
    }
}

fn order_at(words: &[usize], index: usize) -> usize {
    match words.get(index) {
        Some(word) => *word,
        None => 0,
    }
}

fn rotation_at(words: &[u32], index: usize) -> u32 {
    match words.get(index) {
        Some(word) => *word,
        None => 0,
    }
}

fn set_word(words: &mut [u32], index: usize, value: u32) {
    if let Some(word) = words.get_mut(index) {
        *word = value;
    }
}
