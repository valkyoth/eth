#![no_std]
#![forbid(unsafe_code)]
//! Core `no_std` Ethereum primitive types used across the `eth` workspace.

macro_rules! id_type {
    ($name:ident, $inner:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name($inner);

        impl $name {
            /// Creates a new value.
            #[must_use]
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }

            /// Returns the raw integer value.
            #[must_use]
            pub const fn get(self) -> $inner {
                self.0
            }
        }
    };
}

id_type!(ChainId, u64, "Ethereum chain identifier.");
id_type!(BlockNumber, u64, "Ethereum execution-layer block number.");
id_type!(Gas, u64, "Gas quantity.");
id_type!(Nonce, u64, "Account transaction nonce.");
id_type!(UnixTimestamp, u64, "Block timestamp as Unix seconds.");

/// Fixed-width Ethereum address bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Address([u8; 20]);

impl Address {
    /// Creates an address from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Returns the raw address bytes.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 20] {
        self.0
    }
}

/// Fixed-width 256-bit hash bytes.
///
/// `PartialEq` is suitable for ordinary public hash comparisons. Use
/// [`B256::ct_eq`] when comparison timing is part of a security boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct B256([u8; 32]);

impl B256 {
    /// Creates a hash from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw hash bytes.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Compares two hashes without early exit.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> bool {
        let mut diff = 0_u8;
        for (left, right) in self.0.iter().zip(other.0.iter()) {
            diff |= left ^ right;
        }
        diff == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_id_round_trips() {
        assert_eq!(ChainId::new(1).get(), 1);
    }

    #[test]
    fn address_round_trips() {
        let bytes = [7_u8; 20];
        assert_eq!(Address::from_bytes(bytes).to_bytes(), bytes);
    }

    #[test]
    fn b256_constant_time_equality_result_matches_equality() {
        let left = B256::from_bytes([1_u8; 32]);
        let same = B256::from_bytes([1_u8; 32]);
        let different = B256::from_bytes([2_u8; 32]);
        assert!(left.ct_eq(&same));
        assert!(!left.ct_eq(&different));
    }
}
