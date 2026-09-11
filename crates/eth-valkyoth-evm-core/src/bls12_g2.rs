//! Public-input G2 arithmetic. Jacobian x=X/Z^2, y=Y/Z^3; Z=0 is infinity.

use crate::{EvmBls12381Fp2, EvmBls12381G2Point, EvmCoreError, bls12_fp2::Fp2};

/// An on-curve BLS12-381 G2 point, including infinity, NOT subgroup-validated.
///
/// Public inputs only. Operations are bounded but not constant-time and do
/// not authorize a precompile, signing, MSM or signature verification.
///
/// ```
/// use eth_valkyoth_evm_core::{EvmBls12381Fp2 as Fp2, EvmBls12381G2Affine as G2};
/// // Public example point. Curve validity is not subgroup validity.
/// let x = Fp2::from_u64(2);
/// let four = eth_valkyoth_evm_core::EvmBls12381Fp::from_u64(4);
/// let y = x.square().mul_mod(x).add_mod(Fp2::from_coefficients(four, four))
///     .sqrt().ok_or(eth_valkyoth_evm_core::EvmCoreError::PrecompilePointNotOnCurve)?;
/// let point = G2::try_from_coordinates(x, y)?;
/// assert_eq!(point.double(), point.add_point(point));
/// assert!(point.add_point(point.negate()).is_infinity());
/// # Ok::<(), eth_valkyoth_evm_core::EvmCoreError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmBls12381G2Affine {
    coordinates: Option<(EvmBls12381Fp2, EvmBls12381Fp2)>,
}

impl EvmBls12381G2Affine {
    /// Returns the group identity, encoded as 256 zero bytes on the wire.
    #[must_use]
    pub const fn infinity() -> Self {
        Self { coordinates: None }
    }

    /// Validates a finite point against y^2 = x^3 + 4*(1+v); no subgroup test.
    /// `(0, 0)` is not a finite curve point: use [`Self::infinity`] instead.
    pub fn try_from_coordinates(
        x: EvmBls12381Fp2,
        y: EvmBls12381Fp2,
    ) -> Result<Self, EvmCoreError> {
        let fx = Fp2::from_wire(x);
        let fy = Fp2::from_wire(y);
        if fy.square() != fx.square().mul(fx).add(Fp2::from_coefficients(4, 4)) {
            return Err(EvmCoreError::PrecompilePointNotOnCurve);
        }
        Ok(Self {
            coordinates: Some((x, y)),
        })
    }

    /// Adds curve validation to the canonical wire-only domain.
    /// Infinity is accepted; on-curve points outside the prime subgroup remain valid.
    pub fn try_from_wire(point: EvmBls12381G2Point) -> Result<Self, EvmCoreError> {
        match point.coordinates() {
            Some((x, y)) => Self::try_from_coordinates(x, y),
            None => Ok(Self::infinity()),
        }
    }

    /// Decodes exactly 256 canonical EIP-2537 bytes and validates curve membership.
    /// Does not reduce noncanonical coordinates or check subgroup membership.
    pub fn try_from_be_bytes(input: &[u8]) -> Result<Self, EvmCoreError> {
        Self::try_from_wire(EvmBls12381G2Point::try_from_be_bytes(input)?)
    }

    /// Returns finite coordinates, or `None` for infinity.
    #[must_use]
    pub const fn coordinates(self) -> Option<(EvmBls12381Fp2, EvmBls12381Fp2)> {
        self.coordinates
    }

    /// Returns whether this is the group identity.
    #[must_use]
    pub const fn is_infinity(self) -> bool {
        self.coordinates.is_none()
    }

    /// Returns the canonical 256-byte EIP-2537 encoding, including zero infinity.
    #[must_use]
    pub fn to_be_bytes(self) -> [u8; 256] {
        let mut output = [0; 256];
        if let Some((x, y)) = self.coordinates {
            let (left, right) = output.split_at_mut(128);
            left.copy_from_slice(&x.to_be_bytes());
            right.copy_from_slice(&y.to_be_bytes());
        }
        output
    }

    /// Converts to Jacobian representation without inversion.
    #[must_use]
    pub fn to_projective(self) -> EvmBls12381G2Projective {
        EvmBls12381G2Projective::from_affine(self)
    }

    /// Returns the group inverse of a public point; not constant-time.
    #[must_use]
    pub fn negate(self) -> Self {
        Self {
            coordinates: self.coordinates.map(|(x, y)| (x, y.negate())),
        }
    }

    /// Adds public points, including equal, inverse and infinity inputs.
    /// Uses at most one inversion; not constant-time or gas-authorized.
    #[must_use]
    pub fn add_point(self, rhs: Self) -> Self {
        self.to_projective()
            .add_point(rhs.to_projective())
            .to_affine()
    }

    /// Doubles a public point, using at most one inversion; not constant-time.
    #[must_use]
    pub fn double(self) -> Self {
        self.to_projective().double().to_affine()
    }
}

/// On-curve Jacobian G2 point with private coordinates; NOT subgroup-validated.
///
/// Constructed only from validated affine points or closed group operations.
/// Addition/doubling/equality need no inversion. Public inputs only, no heap,
/// no constant-time or precompile authorization claim. Equality is geometric,
/// not equality of the particular projective coordinates.
#[derive(Clone, Copy)]
pub struct EvmBls12381G2Projective {
    x: Fp2,
    y: Fp2,
    z: Fp2,
}

impl EvmBls12381G2Projective {
    /// Returns the canonical internal identity `(0, 1, 0)`.
    #[must_use]
    pub fn infinity() -> Self {
        Self {
            x: Fp2::from_u64(0),
            y: Fp2::from_u64(1),
            z: Fp2::from_u64(0),
        }
    }

    /// Converts a validated affine point without inversion.
    #[must_use]
    pub fn from_affine(point: EvmBls12381G2Affine) -> Self {
        match point.coordinates {
            Some((x, y)) => Self {
                x: Fp2::from_wire(x),
                y: Fp2::from_wire(y),
                z: Fp2::from_u64(1),
            },
            None => Self::infinity(),
        }
    }

    /// Returns whether Z is zero (the group identity).
    #[must_use]
    pub fn is_infinity(self) -> bool {
        self.z == Fp2::from_u64(0)
    }

    /// Normalizes to affine using one inversion for finite points.
    #[must_use]
    pub fn to_affine(self) -> EvmBls12381G2Affine {
        // In the extension field inversion returns None exactly for Z=0.
        let Some(inverse) = self.z.invert() else {
            return EvmBls12381G2Affine::infinity();
        };
        let squared = inverse.square();
        EvmBls12381G2Affine {
            coordinates: Some((
                self.x.mul(squared).to_wire(),
                self.y.mul(squared).mul(inverse).to_wire(),
            )),
        }
    }

    /// Negates a public point, preserving canonical infinity.
    #[must_use]
    pub fn negate(self) -> Self {
        if self.is_infinity() {
            return Self::infinity();
        }
        Self {
            y: Fp2::from_u64(0).sub(self.y),
            ..self
        }
    }

    /// Doubles a public point with fixed scratch and no inversion.
    #[must_use]
    pub fn double(self) -> Self {
        if self.is_infinity() || self.y == Fp2::from_u64(0) {
            return Self::infinity();
        }
        // EFD dbl-2009-l for a=0, with exceptional cases handled above.
        let a = self.x.square();
        let b = self.y.square();
        let c = b.square();
        let d = self.x.add(b).square().sub(a).sub(c);
        let d = d.add(d);
        let e = a.add(a).add(a);
        let x = e.square().sub(d.add(d));
        let c2 = c.add(c);
        let c4 = c2.add(c2);
        let y = e.mul(d.sub(x)).sub(c4.add(c4));
        let yz = self.y.mul(self.z);
        Self {
            x,
            y,
            z: yz.add(yz),
        }
    }

    /// Adds public projective points, including all exceptional cases.
    /// No inversion or input-dependent loop; branches are NOT constant-time.
    #[must_use]
    pub fn add_point(self, rhs: Self) -> Self {
        if self.is_infinity() {
            return rhs;
        }
        if rhs.is_infinity() {
            return self;
        }
        // EFD add-2007-bl. Compare scaled coordinates before the generic formula.
        let z1z1 = self.z.square();
        let z2z2 = rhs.z.square();
        let u1 = self.x.mul(z2z2);
        let u2 = rhs.x.mul(z1z1);
        let s1 = self.y.mul(rhs.z).mul(z2z2);
        let s2 = rhs.y.mul(self.z).mul(z1z1);
        if u1 == u2 {
            return if s1 == s2 {
                self.double()
            } else {
                Self::infinity()
            };
        }
        let h = u2.sub(u1);
        let i = h.add(h).square();
        let j = h.mul(i);
        let r = s2.sub(s1);
        let r = r.add(r);
        let v = u1.mul(i);
        let x = r.square().sub(j).sub(v.add(v));
        let s1j = s1.mul(j);
        let y = r.mul(v.sub(x)).sub(s1j.add(s1j));
        let z = self.z.add(rhs.z).square().sub(z1z1).sub(z2z2).mul(h);
        Self { x, y, z }
    }
}

impl PartialEq for EvmBls12381G2Projective {
    fn eq(&self, rhs: &Self) -> bool {
        if self.is_infinity() || rhs.is_infinity() {
            return self.is_infinity() && rhs.is_infinity();
        }
        let z1z1 = self.z.square();
        let z2z2 = rhs.z.square();
        self.x.mul(z2z2) == rhs.x.mul(z1z1)
            && self.y.mul(z2z2).mul(rhs.z) == rhs.y.mul(z1z1).mul(self.z)
    }
}

impl Eq for EvmBls12381G2Projective {}

impl core::fmt::Debug for EvmBls12381G2Projective {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EvmBls12381G2Projective")
            .field("infinity", &self.is_infinity())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "bls12_g2_tests.rs"]
mod tests;
