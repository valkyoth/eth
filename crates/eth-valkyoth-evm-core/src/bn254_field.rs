use core::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Fp(U256);

impl Fp {
    pub(crate) const ZERO: Self = Self(U256::ZERO);
    pub(crate) const ONE: Self = Self(U256::ONE);
    pub(crate) const THREE: Self = Self(U256::THREE);

    pub(crate) fn from_be_bytes(bytes: [u8; 32]) -> Option<Self> {
        let value = U256::from_be_bytes(bytes);
        if value.cmp(&U256::MODULUS) == Ordering::Less {
            Some(Self(value))
        } else {
            None
        }
    }

    pub(crate) fn to_be_bytes(self) -> [u8; 32] {
        self.0.to_be_bytes()
    }

    pub(crate) fn is_zero(self) -> bool {
        self.0 == U256::ZERO
    }

    pub(crate) fn add(self, rhs: Self) -> Self {
        Self(self.0.add_mod(rhs.0))
    }

    pub(crate) fn sub(self, rhs: Self) -> Self {
        Self(self.0.sub_mod(rhs.0))
    }

    pub(crate) fn double(self) -> Self {
        self.add(self)
    }

    pub(crate) fn mul(self, rhs: Self) -> Self {
        Self(self.0.mul_mod(rhs.0))
    }

    pub(crate) fn square(self) -> Self {
        self.mul(self)
    }

    pub(crate) fn invert(self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            Some(Self(self.0.pow_mod(U256::MODULUS_MINUS_TWO)))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct U256([u64; 4]);

impl U256 {
    const ZERO: Self = Self([0, 0, 0, 0]);
    const ONE: Self = Self([1, 0, 0, 0]);
    const THREE: Self = Self([3, 0, 0, 0]);
    const MODULUS: Self = Self([
        0x43e1_f593_f000_0001,
        0x2833_e848_79b9_7091,
        0xb850_45b6_8181_585d,
        0x3064_4e72_e131_a029,
    ]);
    const MODULUS_MINUS_TWO: Self = Self([
        0x43e1_f593_efff_ffff,
        0x2833_e848_79b9_7091,
        0xb850_45b6_8181_585d,
        0x3064_4e72_e131_a029,
    ]);

    fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for (index, chunk) in bytes.chunks_exact(8).rev().enumerate() {
            let mut word = [0u8; 8];
            word.copy_from_slice(chunk);
            if let Some(slot) = limbs.get_mut(index) {
                *slot = u64::from_be_bytes(word);
            }
        }
        Self(limbs)
    }

    fn to_be_bytes(self) -> [u8; 32] {
        let mut output = [0u8; 32];
        for (chunk, limb) in output.chunks_exact_mut(8).zip(self.0.iter().rev()) {
            chunk.copy_from_slice(&limb.to_be_bytes());
        }
        output
    }

    fn bit(self, bit: usize) -> bool {
        let limb = bit / 64;
        let offset = bit % 64;
        self.0
            .get(limb)
            .copied()
            .map(|value| ((value >> offset) & 1) == 1)
            .unwrap_or(false)
    }

    fn add_mod(self, rhs: Self) -> Self {
        let (sum, carry) = self.add_full(rhs);
        if carry || sum.cmp(&Self::MODULUS) != Ordering::Less {
            sum.sub_raw(Self::MODULUS)
        } else {
            sum
        }
    }

    fn sub_mod(self, rhs: Self) -> Self {
        if self.cmp(&rhs) == Ordering::Less {
            Self::MODULUS.sub_raw(rhs.sub_raw(self))
        } else {
            self.sub_raw(rhs)
        }
    }

    fn mul_mod(self, rhs: Self) -> Self {
        U512::mul(self, rhs).reduce_mod()
    }

    fn pow_mod(self, exponent: Self) -> Self {
        let mut result = Self::ONE;
        let mut base = self;
        for bit in 0..256 {
            if exponent.bit(bit) {
                result = result.mul_mod(base);
            }
            base = base.mul_mod(base);
        }
        result
    }

    fn add_full(self, rhs: Self) -> (Self, bool) {
        let mut output = [0u64; 4];
        let mut carry = 0u128;
        for ((left, right), slot) in self.0.iter().zip(rhs.0.iter()).zip(output.iter_mut()) {
            let sum = u128::from(*left)
                .wrapping_add(u128::from(*right))
                .wrapping_add(carry);
            *slot = low_u64(sum);
            carry = high_u64(sum);
        }
        (Self(output), carry != 0)
    }

    fn sub_raw(self, rhs: Self) -> Self {
        let mut output = [0u64; 4];
        let mut borrow = 0u128;
        for ((left, right), slot) in self.0.iter().zip(rhs.0.iter()).zip(output.iter_mut()) {
            let subtrahend = u128::from(*right).wrapping_add(borrow);
            let minuend = u128::from(*left);
            if minuend >= subtrahend {
                *slot = low_u64(minuend.wrapping_sub(subtrahend));
                borrow = 0;
            } else {
                *slot = low_u64(U64_BASE.wrapping_add(minuend).wrapping_sub(subtrahend));
                borrow = 1;
            }
        }
        Self(output)
    }

    fn shl1(self) -> Self {
        let mut output = [0u64; 4];
        let mut carry = 0u64;
        for (source, slot) in self.0.iter().zip(output.iter_mut()) {
            *slot = (*source << 1) | carry;
            carry = *source >> 63;
        }
        Self(output)
    }

    fn add_small(self, value: u64) -> Self {
        let mut output = self.0;
        let mut carry = value;
        for slot in &mut output {
            let (next, overflow) = slot.overflowing_add(carry);
            *slot = next;
            carry = u64::from(overflow);
            if carry == 0 {
                break;
            }
        }
        Self(output)
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy)]
struct U512([u64; 8]);

impl U512 {
    fn mul(left: U256, right: U256) -> Self {
        let mut output = [0u64; 8];
        for (left_index, left_limb) in left.0.iter().enumerate() {
            let mut carry = 0u128;
            for (right_index, right_limb) in right.0.iter().enumerate() {
                let out_index = left_index.saturating_add(right_index);
                let Some(slot) = output.get_mut(out_index) else {
                    return Self(output);
                };
                let product = u128::from(*left_limb).wrapping_mul(u128::from(*right_limb));
                let total = u128::from(*slot).wrapping_add(product).wrapping_add(carry);
                *slot = low_u64(total);
                carry = high_u64(total);
            }
            propagate_carry(&mut output, left_index.saturating_add(4), low_u64(carry));
        }
        Self(output)
    }

    fn bit(self, bit: usize) -> bool {
        let limb = bit / 64;
        let offset = bit % 64;
        self.0
            .get(limb)
            .copied()
            .map(|value| ((value >> offset) & 1) == 1)
            .unwrap_or(false)
    }

    fn reduce_mod(self) -> U256 {
        let mut remainder = U256::ZERO;
        for bit in (0..512).rev() {
            remainder = remainder.shl1();
            if self.bit(bit) {
                remainder = remainder.add_small(1);
            }
            if remainder.cmp(&U256::MODULUS) != Ordering::Less {
                remainder = remainder.sub_raw(U256::MODULUS);
            }
        }
        remainder
    }
}

fn propagate_carry(output: &mut [u64; 8], mut index: usize, mut carry: u64) {
    while carry != 0 {
        let Some(slot) = output.get_mut(index) else {
            return;
        };
        let (next, overflow) = slot.overflowing_add(carry);
        *slot = next;
        carry = u64::from(overflow);
        index = index.saturating_add(1);
    }
}

const U64_BASE: u128 = 1u128 << 64;

fn low_u64(value: u128) -> u64 {
    let bytes = value.to_le_bytes();
    let mut low = [0u8; 8];
    if let Some(source) = bytes.get(..8) {
        low.copy_from_slice(source);
    }
    u64::from_le_bytes(low)
}

fn high_u64(value: u128) -> u128 {
    let bytes = value.to_le_bytes();
    let mut high = [0u8; 8];
    if let Some(source) = bytes.get(8..16) {
        high.copy_from_slice(source);
    }
    u128::from(u64::from_le_bytes(high))
}
