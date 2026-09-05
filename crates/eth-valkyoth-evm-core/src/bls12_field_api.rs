use crate::{EvmBls12381Fp, bls12_field::Fp};

impl EvmBls12381Fp {
    /// Creates a field value from a public integer (every `u64` is below p).
    ///
    /// ```
    /// use eth_valkyoth_evm_core::EvmBls12381Fp;
    /// let two = EvmBls12381Fp::from_u64(2);
    /// assert_eq!(two.square().sqrt(), Some(two));
    /// ```
    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        Self::from_field(Fp::from_u64(value))
    }

    /// Reduces an exact 768-bit public integer modulo p.
    ///
    /// This is arithmetic reduction, NOT canonical EIP-2537 decoding. Use
    /// [`Self::try_from_be_bytes`] to reject noncanonical wire input instead.
    /// Runs 768 bit steps without allocation; not a secret-input API.
    #[must_use]
    pub fn from_wide_be_bytes(bytes: [u8; 96]) -> Self {
        Self::from_field(Fp::reduce(bytes))
    }

    /// Adds two public field values modulo p. Not constant-time.
    #[must_use]
    pub fn add_mod(self, rhs: Self) -> Self {
        Self::from_field(self.field().add(rhs.field()))
    }

    /// Subtracts two public field values modulo p. Not constant-time.
    #[must_use]
    pub fn sub_mod(self, rhs: Self) -> Self {
        Self::from_field(self.field().sub(rhs.field()))
    }

    /// Multiplies two public field values modulo p. Not constant-time.
    #[must_use]
    pub fn mul_mod(self, rhs: Self) -> Self {
        Self::from_field(self.field().mul(rhs.field()))
    }

    /// Squares a public field value modulo p. Not constant-time.
    #[must_use]
    pub fn square(self) -> Self {
        Self::from_field(self.field().square())
    }

    /// Returns the additive inverse of a public value. Not constant-time.
    #[must_use]
    pub fn negate(self) -> Self {
        Self::from_u64(0).sub_mod(self)
    }

    /// Inverts a public nonzero value; returns `None` for zero.
    ///
    /// Uses a fixed 384-bit exponent loop and bounded scratch, but comparisons
    /// and reductions are not constant-time. Do not use for signing secrets.
    #[must_use]
    pub fn invert(self) -> Option<Self> {
        self.field().invert().map(Self::from_field)
    }

    /// Returns the numerically smaller square root, or `None` for a nonsquare.
    ///
    /// Zero has root zero. The result is checked by squaring before return.
    /// Fixed-width, bounded public-input work; not constant-time.
    #[must_use]
    pub fn sqrt(self) -> Option<Self> {
        self.field().sqrt().map(Self::from_field)
    }

    fn field(self) -> Fp {
        Fp::from_wire(self)
    }
}
