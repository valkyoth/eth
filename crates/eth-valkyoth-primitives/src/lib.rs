#![no_std]
#![forbid(unsafe_code)]
//! Core `no_std` Ethereum primitive types used across the `eth` workspace.

use core::hash::{Hash, Hasher};
use eth_valkyoth_codec::{
    DecodeError, rlp_integer_payload_to_u64, rlp_integer_payload_to_u256_bytes,
};
pub use subtle::Choice;
use subtle::ConstantTimeEq as _;

mod rlp;
#[cfg(test)]
mod tests;

pub use rlp::PrimitiveRlpError;

fn parse_canonical_u64(bytes: &[u8]) -> Result<u64, PrimitiveError> {
    // Do not duplicate Ethereum RLP integer canonicality in this crate.
    // `eth-valkyoth-codec` is the single source of truth for payload parsing.
    rlp_integer_payload_to_u64(bytes).map_err(map_rlp_integer_error)
}

fn canonical_u256_bytes(bytes: &[u8]) -> Result<[u8; 32], PrimitiveError> {
    // Do not duplicate Ethereum RLP integer canonicality in this crate.
    // `eth-valkyoth-codec` is the single source of truth for payload parsing.
    rlp_integer_payload_to_u256_bytes(bytes).map_err(map_rlp_integer_error)
}

fn map_rlp_integer_error(error: DecodeError) -> PrimitiveError {
    match error {
        DecodeError::Malformed => PrimitiveError::NonCanonicalInteger,
        DecodeError::LengthOverflow => PrimitiveError::IntegerTooLarge,
        // Payload helpers are expected to emit only Malformed for leading-zero
        // canonicality failures or LengthOverflow for primitive width failures.
        // OffsetOutOfBounds is structurally unreachable for the current U256
        // right-alignment helper, and any other codec error would be a
        // programming error rather than a wire-domain primitive failure.
        unexpected => {
            debug_assert!(
                matches!(unexpected, DecodeError::OffsetOutOfBounds),
                "unexpected codec error from RLP integer payload helper"
            );
            PrimitiveError::IntegerTooLarge
        }
    }
}

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

            /// Creates a value from a canonical RLP integer payload.
            ///
            /// The empty payload represents zero. Non-empty payloads must be
            /// shortest-form unsigned big-endian bytes without a leading zero.
            pub fn try_from_canonical_be_slice(bytes: &[u8]) -> Result<Self, PrimitiveError> {
                parse_canonical_u64(bytes).map(Self)
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

id_type!(
    ChainId,
    u64,
    "Ethereum chain identifier.\n\nThis type enforces canonical integer encoding only. EIP-155 assigns chain ID 0 to unsigned legacy transactions; signed transaction validation must reject chain ID 0 independently."
);
id_type!(BlockNumber, u64, "Ethereum execution-layer block number.");
id_type!(Gas, u64, "Gas quantity.");
id_type!(Nonce, u64, "Account transaction nonce.");
id_type!(UnixTimestamp, u64, "Block timestamp as Unix seconds.");

/// Primitive constructor failures.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveError {
    /// Integer bytes were not in shortest-form canonical big-endian encoding.
    NonCanonicalInteger,
    /// Integer bytes exceed the primitive's fixed-width range.
    IntegerTooLarge,
    /// Transaction type exceeds the EIP-2718 single-byte typed envelope range.
    TransactionTypeTooLarge,
    /// Zero is reserved for the legacy transaction domain, not a typed envelope.
    ReservedLegacyType,
}

/// Fixed-width Ethereum address bytes.
///
/// Equality is constant-time because recovered sender checks appear in
/// authentication and authorization paths.
///
/// The [`Hash`] implementation is for ordinary map/set use. Do not rely on
/// hash-map lookup timing for secret or side-channel-sensitive access-control
/// paths; use explicit indexed structures or constant-time scans there.
#[derive(Clone, Copy, Debug)]
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

    /// Compares two addresses in constant time.
    ///
    /// Returns [`Choice`] so compound comparisons can use `&` and `|` without
    /// short-circuiting. Convert to `bool` only at the final trust boundary.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl Eq for Address {}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
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
/// All equality for this type is constant-time because hashes appear in
/// cryptographic verification paths.
///
/// The [`Hash`] implementation is for ordinary map/set use. Do not rely on
/// hash-map lookup timing for secret or side-channel-sensitive verification
/// paths; use explicit indexed structures or constant-time scans there.
#[derive(Clone, Copy, Debug)]
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
    /// Returns [`Choice`] so compound comparisons can use `&` and `|` without
    /// short-circuiting. Convert to `bool` only at the final trust boundary.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl PartialEq for B256 {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl Eq for B256 {}

impl Hash for B256 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
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
#[derive(Clone, Copy, Debug)]
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

    /// Creates a wei amount from a canonical RLP integer payload.
    ///
    /// The empty payload represents zero. Non-empty payloads must be
    /// shortest-form unsigned big-endian bytes without a leading zero.
    pub fn try_from_canonical_be_slice(bytes: &[u8]) -> Result<Self, PrimitiveError> {
        canonical_u256_bytes(bytes).map(Self)
    }

    /// Compares two wei values in constant time.
    ///
    /// Wei is usually public, but fixed-width constant-time equality keeps
    /// proof and verification paths from accidentally choosing a weaker API.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl PartialEq for Wei {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl Eq for Wei {}

impl Hash for Wei {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
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

    /// Creates a typed EIP-2718 transaction type.
    ///
    /// `0` is reserved for the legacy transaction domain. Encoders must handle
    /// legacy transactions separately instead of prepending a zero type byte.
    pub const fn try_new_typed(value: u8) -> Result<Self, PrimitiveError> {
        match value {
            0 => Err(PrimitiveError::ReservedLegacyType),
            1..=Self::MAX_TYPED => Ok(Self(value)),
            _ => Err(PrimitiveError::TransactionTypeTooLarge),
        }
    }

    /// Creates a transaction type and accepts the legacy domain marker.
    ///
    /// Use this when an API explicitly accepts both legacy and typed
    /// transaction domains. Use [`Self::try_new_typed`] when parsing an
    /// EIP-2718 typed-envelope byte.
    pub const fn try_new_with_legacy(value: u8) -> Result<Self, PrimitiveError> {
        Self::try_new(value)
    }

    /// Returns the raw transaction type byte.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for TransactionType {
    type Error = PrimitiveError;

    /// Parses a typed EIP-2718 transaction type byte.
    ///
    /// Rejects `0` because it belongs to the legacy transaction domain. Use
    /// [`TransactionType::try_new_with_legacy`] when the caller explicitly
    /// accepts both legacy and typed transaction domains.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_new_typed(value)
    }
}

impl From<TransactionType> for u8 {
    fn from(value: TransactionType) -> Self {
        value.get()
    }
}
