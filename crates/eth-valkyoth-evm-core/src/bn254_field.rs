use core::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Fp(U256);

impl Fp {
    pub(crate) const ZERO: Self = Self(U256::ZERO);
    pub(crate) const ONE: Self = Self(U256::R);
    pub(crate) const THREE: Self = Self(U256::THREE_R);

    pub(crate) fn from_be_bytes(bytes: [u8; 32]) -> Option<Self> {
        let value = U256::from_be_bytes(bytes);
        if value.cmp(&U256::MODULUS) == Ordering::Less {
            Some(Self(value.to_montgomery()))
        } else {
            None
        }
    }

    pub(crate) fn to_be_bytes(self) -> [u8; 32] {
        self.0.to_canonical().to_be_bytes()
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
        Self(self.0.montgomery_mul(rhs.0))
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
    const THREE_R: Self = Self([
        0x7a17_caa9_50ad_28d7,
        0x1f6a_c17a_e155_21b9,
        0x334b_ea4e_696b_d284,
        0x2a1f_6744_ce17_9d8e,
    ]);
    const MODULUS: Self = Self([
        0x3c20_8c16_d87c_fd47,
        0x9781_6a91_6871_ca8d,
        0xb850_45b6_8181_585d,
        0x3064_4e72_e131_a029,
    ]);
    const MODULUS_MINUS_TWO: Self = Self([
        0x3c20_8c16_d87c_fd45,
        0x9781_6a91_6871_ca8d,
        0xb850_45b6_8181_585d,
        0x3064_4e72_e131_a029,
    ]);
    const MONTGOMERY_INV: u64 = 0x87d2_0782_e486_6389;
    const R: Self = Self([
        0xd35d_438d_c58f_0d9d,
        0x0a78_eb28_f5c7_0b3d,
        0x666e_a36f_7879_462c,
        0x0e0a_77c1_9a07_df2f,
    ]);
    const R2: Self = Self([
        0xf32c_fc5b_538a_fa89,
        0xb5e7_1911_d445_01fb,
        0x47ab_1eff_0a41_7ff6,
        0x06d8_9f71_cab8_351f,
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

    fn limb(self, index: usize) -> u64 {
        self.0.get(index).copied().unwrap_or(0)
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

    fn to_montgomery(self) -> Self {
        self.montgomery_mul(Self::R2)
    }

    fn to_canonical(self) -> Self {
        self.montgomery_mul(Self::ONE)
    }

    fn montgomery_mul(self, rhs: Self) -> Self {
        let mut state = MontgomeryState::default();
        for right in rhs.0 {
            state.mul_add_limb(self, right);
            state.reduce_one_limb();
        }
        state.finish()
    }

    fn pow_mod(self, exponent: Self) -> Self {
        let mut result = Self::R;
        let mut base = self;
        for bit in 0..256 {
            if exponent.bit(bit) {
                result = result.montgomery_mul(base);
            }
            base = base.montgomery_mul(base);
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

const U64_BASE: u128 = 1u128 << 64;

#[derive(Default)]
struct MontgomeryState([u64; 5]);

impl MontgomeryState {
    fn mul_add_limb(&mut self, left: U256, right: u64) {
        let mut carry = 0u128;
        for (index, left_limb) in left.0.iter().enumerate() {
            let total = u128::from(self.limb(index))
                .wrapping_add(u128::from(*left_limb).wrapping_mul(u128::from(right)))
                .wrapping_add(carry);
            self.set_limb(index, low_u64(total));
            carry = high_u64(total);
        }
        self.set_limb(4, low_u64(carry));
    }

    fn reduce_one_limb(&mut self) {
        let factor = self.limb(0).wrapping_mul(U256::MONTGOMERY_INV);
        let mut carry = high_u64(
            u128::from(self.limb(0))
                .wrapping_add(u128::from(factor).wrapping_mul(u128::from(U256::MODULUS.limb(0)))),
        );
        for index in 1..4 {
            let total = u128::from(self.limb(index))
                .wrapping_add(
                    u128::from(factor).wrapping_mul(u128::from(U256::MODULUS.limb(index))),
                )
                .wrapping_add(carry);
            self.set_limb(index.saturating_sub(1), low_u64(total));
            carry = high_u64(total);
        }
        let total = u128::from(self.limb(4)).wrapping_add(carry);
        self.set_limb(3, low_u64(total));
        self.set_limb(4, low_u64(high_u64(total)));
    }

    fn finish(self) -> U256 {
        let value = U256([self.limb(0), self.limb(1), self.limb(2), self.limb(3)]);
        if self.limb(4) != 0 || value.cmp(&U256::MODULUS) != Ordering::Less {
            value.sub_raw(U256::MODULUS)
        } else {
            value
        }
    }

    fn limb(&self, index: usize) -> u64 {
        self.0.get(index).copied().unwrap_or(0)
    }

    fn set_limb(&mut self, index: usize, value: u64) {
        if let Some(slot) = self.0.get_mut(index) {
            *slot = value;
        }
    }
}

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
