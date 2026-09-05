//! Public-input Fp kernel. Six little-endian Montgomery limbs, always below p.
//! Index arithmetic is bounded by 6 limbs/13 scratch words, never caller lengths.

use crate::bls12_wire::FP_MODULUS;

const N: usize = 6;
type Limbs = [u64; N];
const ONE: Limbs = [1, 0, 0, 0, 0, 0];
const INV: u64 = 0x89f3_fffc_fffc_fffd;
const R2: Limbs = [
    0xf4df_1f34_1c34_1746,
    0x0a76_e6a6_09d1_04f1,
    0x8de5_476c_4c95_b6d5,
    0x67eb_88a9_939d_83c0,
    0x9a79_3e85_b519_952d,
    0x1198_8fe5_92ca_e3aa,
];

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct Fp(Limbs);

impl Fp {
    pub(crate) fn from_wire(value: crate::EvmBls12381Fp) -> Self {
        // The canonical wire type enforces the Montgomery operand precondition.
        Self(montgomery(limbs(*value.value_bytes()), R2))
    }

    pub(crate) fn to_canonical(self) -> [u8; 48] {
        let mut bytes = [0; 48];
        for (chunk, limb) in bytes
            .chunks_exact_mut(8)
            .zip(montgomery(self.0, ONE).iter().rev())
        {
            chunk.copy_from_slice(&limb.to_be_bytes());
        }
        bytes
    }

    pub(crate) fn from_u64(value: u64) -> Self {
        let mut raw = [0; N];
        if let Some(first) = raw.first_mut() {
            *first = value;
        }
        Self(montgomery(raw, R2))
    }

    pub(crate) fn reduce(bytes: [u8; 96]) -> Self {
        let mut result = Self([0; N]);
        let one = Self::from_u64(1);
        for byte in bytes {
            for bit in (0..8).rev() {
                result = result.add(result);
                if byte & (1 << bit) != 0 {
                    result = result.add(one);
                }
            }
        }
        result
    }

    pub(crate) fn add(self, rhs: Self) -> Self {
        let mut sum = [0; N];
        let mut carry = 0u128;
        for ((a, b), out) in self.0.iter().zip(rhs.0).zip(&mut sum) {
            let total = u128::from(*a)
                .wrapping_add(u128::from(b))
                .wrapping_add(carry);
            *out = low(total);
            carry = total >> 64;
        }
        // Both operands < p; 2p < 2^384 so there is no final carry.
        Self(reduce_once(sum))
    }

    pub(crate) fn sub(self, rhs: Self) -> Self {
        if less(self.0, rhs.0) {
            Self(sub_raw(modulus(), sub_raw(rhs.0, self.0)))
        } else {
            Self(sub_raw(self.0, rhs.0))
        }
    }

    pub(crate) fn mul(self, rhs: Self) -> Self {
        Self(montgomery(self.0, rhs.0))
    }

    pub(crate) fn square(self) -> Self {
        self.mul(self)
    }

    fn pow(self, exponent: Limbs) -> Self {
        let mut result = Self::from_u64(1);
        // Fixed 384-bit public exponent, independent of the input value.
        for word in exponent.iter().rev() {
            for bit in (0..64).rev() {
                result = result.square();
                if word & (1 << bit) != 0 {
                    result = result.mul(self);
                }
            }
        }
        result
    }

    pub(crate) fn invert(self) -> Option<Self> {
        if self.0 == [0; N] {
            None
        } else {
            Some(self.pow(sub_raw(modulus(), [2, 0, 0, 0, 0, 0])))
        }
    }

    pub(crate) fn sqrt(self) -> Option<Self> {
        // p == 3 mod 4: (p+1)/4. Low limb cannot carry when adding one.
        let mut plus_one = modulus();
        if let Some(first) = plus_one.first_mut() {
            *first = first.wrapping_add(1);
        }
        let mut exponent = [0; N];
        let high_words = plus_one.iter().copied().skip(1).chain(core::iter::once(0));
        for ((out, low), high) in exponent.iter_mut().zip(plus_one).zip(high_words) {
            *out = (low >> 2) | (high << 62);
        }
        let root = self.pow(exponent);
        if root.square() != self {
            return None;
        }
        let negative = Self([0; N]).sub(root);
        Some(if root.to_canonical() <= negative.to_canonical() {
            root
        } else {
            negative
        })
    }
}

fn modulus() -> Limbs {
    limbs(FP_MODULUS)
}

fn limbs(bytes: [u8; 48]) -> Limbs {
    let mut output = [0; N];
    for (out, chunk) in output.iter_mut().zip(bytes.chunks_exact(8).rev()) {
        let mut word = [0; 8];
        word.copy_from_slice(chunk);
        *out = u64::from_be_bytes(word);
    }
    output
}

fn less(a: Limbs, b: Limbs) -> bool {
    a.iter().rev().cmp(b.iter().rev()).is_lt()
}

fn sub_raw(a: Limbs, b: Limbs) -> Limbs {
    let mut result = [0; N];
    let mut borrow = false;
    for ((a, b), out) in a.iter().zip(b).zip(&mut result) {
        let (value, first) = a.overflowing_sub(b);
        let (value, second) = value.overflowing_sub(u64::from(borrow));
        *out = value;
        borrow = first || second;
    }
    result
}

fn reduce_once(value: Limbs) -> Limbs {
    if less(value, modulus()) {
        value
    } else {
        sub_raw(value, modulus())
    }
}

fn low(value: u128) -> u64 {
    // Deliberately take the low word; upper bits are carried separately.
    #[allow(clippy::cast_possible_truncation)]
    {
        value as u64
    }
}

fn montgomery(a: Limbs, b: Limbs) -> Limbs {
    let mut t = [0u64; 13];
    for (i, right) in b.iter().enumerate() {
        let mut carry = 0u128;
        for (word, left) in t.iter_mut().skip(i).zip(a) {
            // Product + existing word + carry is at most 2^128 - 1.
            let total = u128::from(left)
                .wrapping_mul(u128::from(*right))
                .wrapping_add(u128::from(*word))
                .wrapping_add(carry);
            *word = low(total);
            carry = total >> 64;
        }
        for word in t.iter_mut().skip(i.saturating_add(N)).take(1) {
            *word = low(carry);
        }
    }
    for i in 0..N {
        let factor = t.get(i).copied().unwrap_or(0).wrapping_mul(INV);
        let mut carry = 0u128;
        for (word, modulus) in t.iter_mut().skip(i).zip(modulus()) {
            let total = u128::from(factor)
                .wrapping_mul(u128::from(modulus))
                .wrapping_add(u128::from(*word))
                .wrapping_add(carry);
            *word = low(total);
            carry = total >> 64;
        }
        // Fixed-length propagation, never a data-dependent carry loop.
        for word in t.iter_mut().skip(i.saturating_add(N)) {
            let total = u128::from(*word).wrapping_add(carry);
            *word = low(total);
            carry = total >> 64;
        }
    }
    // a,b < p gives REDC(a*b) < 2p < 2^384, hence t[12] == 0.
    let mut result = [0; N];
    for (out, word) in result.iter_mut().zip(t.iter().skip(N)) {
        *out = *word;
    }
    reduce_once(result)
}
