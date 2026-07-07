const BLOCK_BYTES: usize = 64;
const DIGEST_BYTES: usize = 32;
const LENGTH_BYTES: usize = 8;

const INITIAL_STATE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
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

fn finalize(state: &mut [u32; 8], remainder: &[u8], input_len: usize) {
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
    let len_bytes = bit_len.to_be_bytes();
    if let Some(target) = block.get_mut(BLOCK_BYTES - LENGTH_BYTES..) {
        target.copy_from_slice(&len_bytes);
    }
    compress(state, &block);
}

fn compress(state: &mut [u32; 8], block: &[u8]) {
    let mut words = [0u32; 64];
    for (word, chunk) in words.iter_mut().take(16).zip(block.chunks_exact(4)) {
        if let Ok(bytes) = <[u8; 4]>::try_from(chunk) {
            *word = u32::from_be_bytes(bytes);
        }
    }
    for index in 16..64 {
        let w15 = word_at(&words, previous_index(index, 15));
        let w2 = word_at(&words, previous_index(index, 2));
        let s0 = w15.rotate_right(7) ^ w15.rotate_right(18) ^ (w15 >> 3);
        let s1 = w2.rotate_right(17) ^ w2.rotate_right(19) ^ (w2 >> 10);
        let value = word_at(&words, previous_index(index, 16))
            .wrapping_add(s0)
            .wrapping_add(word_at(&words, previous_index(index, 7)))
            .wrapping_add(s1);
        set_word(&mut words, index, value);
    }

    let mut a = word_at(state, 0);
    let mut b = word_at(state, 1);
    let mut c = word_at(state, 2);
    let mut d = word_at(state, 3);
    let mut e = word_at(state, 4);
    let mut f = word_at(state, 5);
    let mut g = word_at(state, 6);
    let mut h = word_at(state, 7);

    for (index, constant) in ROUND_CONSTANTS.iter().copied().enumerate() {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(constant)
            .wrapping_add(word_at(&words, index));
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    add_word(state, 0, a);
    add_word(state, 1, b);
    add_word(state, 2, c);
    add_word(state, 3, d);
    add_word(state, 4, e);
    add_word(state, 5, f);
    add_word(state, 6, g);
    add_word(state, 7, h);
}

fn state_to_bytes(state: [u32; 8]) -> [u8; DIGEST_BYTES] {
    let mut output = [0u8; DIGEST_BYTES];
    for (chunk, word) in output.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    output
}

fn word_at(words: &[u32], index: usize) -> u32 {
    match words.get(index) {
        Some(word) => *word,
        None => 0,
    }
}

fn previous_index(index: usize, offset: usize) -> usize {
    index.saturating_sub(offset)
}

fn set_word(words: &mut [u32], index: usize, value: u32) {
    if let Some(word) = words.get_mut(index) {
        *word = value;
    }
}

fn add_word(words: &mut [u32], index: usize, value: u32) {
    if let Some(word) = words.get_mut(index) {
        *word = word.wrapping_add(value);
    }
}
