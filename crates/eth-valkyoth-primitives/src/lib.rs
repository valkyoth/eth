#![no_std]
#![forbid(unsafe_code)]
//! Core `no_std` Ethereum primitive types used across the `eth` workspace.

use subtle::ConstantTimeEq as _;

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

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.get()
            }
        }
    };
}

id_type!(ChainId, u64, "Ethereum chain identifier.");
id_type!(BlockNumber, u64, "Ethereum execution-layer block number.");
id_type!(Gas, u64, "Gas quantity.");
id_type!(Nonce, u64, "Account transaction nonce.");
id_type!(UnixTimestamp, u64, "Block timestamp as Unix seconds.");

/// Primitive constructor failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveError {
    /// Transaction type exceeds the EIP-2718 single-byte typed envelope range.
    TransactionTypeTooLarge,
}

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

impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Address> for [u8; 20] {
    fn from(value: Address) -> Self {
        value.to_bytes()
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

    /// Compares two hashes in constant time.
    ///
    /// Use this instead of `==` whenever the comparison result could influence
    /// control flow observable by an untrusted caller.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl From<[u8; 32]> for B256 {
    fn from(bytes: [u8; 32]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<B256> for [u8; 32] {
    fn from(value: B256) -> Self {
        value.to_bytes()
    }
}

/// Wei amount encoded as an unsigned 256-bit big-endian integer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Wei([u8; 32]);

impl Wei {
    /// Zero wei.
    pub const ZERO: Self = Self([0_u8; 32]);

    /// Creates a wei amount from unsigned 256-bit big-endian bytes.
    #[must_use]
    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Creates a wei amount from a `u128`.
    #[must_use]
    pub const fn from_u128(value: u128) -> Self {
        let [
            b0,
            b1,
            b2,
            b3,
            b4,
            b5,
            b6,
            b7,
            b8,
            b9,
            b10,
            b11,
            b12,
            b13,
            b14,
            b15,
        ] = value.to_be_bytes();
        Self([
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15,
        ])
    }

    /// Returns unsigned 256-bit big-endian bytes.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl From<[u8; 32]> for Wei {
    fn from(bytes: [u8; 32]) -> Self {
        Self::from_be_bytes(bytes)
    }
}

impl From<Wei> for [u8; 32] {
    fn from(value: Wei) -> Self {
        value.to_be_bytes()
    }
}

/// EIP-2718 transaction envelope type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransactionType(u8);

impl TransactionType {
    /// Legacy transaction type used by APIs that need an explicit domain value.
    pub const LEGACY: Self = Self(0);
    /// Largest typed transaction value admitted by EIP-2718.
    pub const MAX_TYPED: u8 = 0x7f;

    /// Creates a transaction type after checking the EIP-2718 range.
    pub const fn try_new(value: u8) -> Result<Self, PrimitiveError> {
        if value > Self::MAX_TYPED {
            return Err(PrimitiveError::TransactionTypeTooLarge);
        }
        Ok(Self(value))
    }

    /// Returns the raw transaction type byte.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for TransactionType {
    type Error = PrimitiveError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<TransactionType> for u8 {
    fn from(value: TransactionType) -> Self {
        value.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_id_round_trips() {
        assert_eq!(u64::from(ChainId::from(1)), 1);
    }

    #[test]
    fn block_number_round_trips() {
        assert_eq!(u64::from(BlockNumber::from(2)), 2);
    }

    #[test]
    fn gas_round_trips() {
        assert_eq!(u64::from(Gas::from(21_000)), 21_000);
    }

    #[test]
    fn nonce_round_trips() {
        assert_eq!(u64::from(Nonce::from(7)), 7);
    }

    #[test]
    fn unix_timestamp_round_trips() {
        assert_eq!(u64::from(UnixTimestamp::from(1_700_000_000)), 1_700_000_000);
    }

    #[test]
    fn address_round_trips() {
        let bytes = [7_u8; 20];
        assert_eq!(<[u8; 20]>::from(Address::from(bytes)), bytes);
    }

    #[test]
    fn b256_constant_time_equality_result_matches_equality() {
        let left = B256::from_bytes([1_u8; 32]);
        let same = B256::from_bytes([1_u8; 32]);
        let different = B256::from_bytes([2_u8; 32]);
        assert!(left.ct_eq(&same));
        assert!(!left.ct_eq(&different));
    }

    #[test]
    fn b256_round_trips() {
        let bytes = [3_u8; 32];
        assert_eq!(<[u8; 32]>::from(B256::from(bytes)), bytes);
    }

    #[test]
    fn wei_round_trips() {
        let bytes = [9_u8; 32];
        assert_eq!(<[u8; 32]>::from(Wei::from(bytes)), bytes);
    }

    #[test]
    fn wei_from_u128_places_bytes_at_low_end() {
        let wei = Wei::from_u128(1);
        let expected = [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 1_u8,
        ];
        assert_eq!(wei.to_be_bytes(), expected);
    }

    #[test]
    fn transaction_type_accepts_eip_2718_range() {
        let tx_type = TransactionType::try_new(TransactionType::MAX_TYPED);
        assert_eq!(tx_type.map(TransactionType::get), Ok(0x7f));
    }

    #[test]
    fn transaction_type_rejects_reserved_range() {
        assert_eq!(
            TransactionType::try_new(0x80),
            Err(PrimitiveError::TransactionTypeTooLarge)
        );
    }

    #[test]
    fn transaction_type_round_trips() {
        assert_eq!(TransactionType::try_new(2).map(u8::from), Ok(2));
    }
}
