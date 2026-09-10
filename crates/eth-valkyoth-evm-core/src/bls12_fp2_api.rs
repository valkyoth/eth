use crate::{EvmBls12381Fp, EvmBls12381Fp2, bls12_fp2::Fp2};

impl EvmBls12381Fp2 {
    /// Embeds a public integer as the constant coefficient, with zero linear part.
    ///
    /// ```
    /// use eth_valkyoth_evm_core::{EvmBls12381Fp as Fp, EvmBls12381Fp2 as Fp2};
    /// let v = Fp2::from_coefficients(Fp::from_u64(0), Fp::from_u64(1));
    /// assert_eq!(v.square(), Fp2::from_u64(1).negate());
    /// assert_eq!(v.mul_mod(v.invert().unwrap()), Fp2::from_u64(1));
    /// assert_eq!(v.square().sqrt(), Some(v));
    /// ```
    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        Self::from_coefficients(EvmBls12381Fp::from_u64(value), EvmBls12381Fp::from_u64(0))
    }

    /// Adds public values modulo `v^2 + 1`. Not constant-time.
    #[must_use]
    pub fn add_mod(self, rhs: Self) -> Self {
        self.field().add(rhs.field()).to_wire()
    }

    /// Subtracts public values modulo `v^2 + 1`. Not constant-time.
    #[must_use]
    pub fn sub_mod(self, rhs: Self) -> Self {
        self.field().sub(rhs.field()).to_wire()
    }

    /// Multiplies public values modulo `v^2 + 1`. Not constant-time.
    #[must_use]
    pub fn mul_mod(self, rhs: Self) -> Self {
        self.field().mul(rhs.field()).to_wire()
    }

    /// Squares a public value. Not constant-time.
    #[must_use]
    pub fn square(self) -> Self {
        self.field().square().to_wire()
    }

    /// Negates both coefficients of a public value. Not constant-time.
    #[must_use]
    pub fn negate(self) -> Self {
        self.field().negate().to_wire()
    }

    /// Maps `c0 + c1*v` to `c0 - c1*v` (also the p-power Frobenius).
    /// Public inputs only; not constant-time.
    #[must_use]
    pub fn conjugate(self) -> Self {
        self.field().conjugate().to_wire()
    }

    /// Inverts a public nonzero value, returning `None` exactly for zero.
    /// Fixed-size storage and one base-field inversion; not constant-time.
    #[must_use]
    pub fn invert(self) -> Option<Self> {
        self.field().invert().map(Fp2::to_wire)
    }

    /// Returns the root with lexicographically smaller `c0 || c1` wire bytes.
    ///
    /// Returns `None` for a nonsquare; zero has root zero. The result is checked
    /// by squaring. Uses at most three base-field square roots and two inversions,
    /// without allocation or recursion. Public inputs only; not constant-time.
    /// This root convention is not a point-compression or hash-to-curve sign rule.
    #[must_use]
    pub fn sqrt(self) -> Option<Self> {
        self.field().sqrt().map(Fp2::to_wire)
    }

    fn field(self) -> Fp2 {
        Fp2::from_wire(self)
    }
}
