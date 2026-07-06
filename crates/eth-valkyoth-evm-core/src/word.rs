/// A canonical 256-bit EVM stack word.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EvmWord([u8; Self::LEN]);

impl EvmWord {
    /// Number of bytes in an EVM word.
    pub const LEN: usize = 32;
    /// The zero word.
    pub const ZERO: Self = Self([0u8; Self::LEN]);

    /// Constructs an EVM word from big-endian bytes.
    #[must_use]
    pub const fn from_be_bytes(bytes: [u8; Self::LEN]) -> Self {
        Self(bytes)
    }

    /// Returns the word as big-endian bytes.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; Self::LEN] {
        self.0
    }

    /// Borrows the word bytes.
    #[must_use]
    pub const fn as_be_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }
}
