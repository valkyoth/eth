use crate::{
    EVM_PRECOMPILE_INPUT_LIMIT, EvmCoreError, EvmPrecompileKind, EvmPrecompilePlan,
    bn254::validate_g1_point, bn254_field::Fp,
};

/// Byte length of one EIP-197 BN254 pairing tuple.
pub const EVM_BN254_PAIRING_ITEM_BYTES: usize = 192;
/// Byte length of the BN254 pairing precompile output word.
pub const EVM_BN254_PAIRING_OUTPUT_BYTES: usize = 32;

/// Validates the EIP-197 BN254 pairing input frame and returns its tuple count.
///
/// This release validates tuple segmentation, G1 points, G2 field elements, and
/// the G2 curve equation. G2 subgroup checks and non-empty pairing execution are
/// intentionally fail-closed until the dedicated pairing-algebra releases.
pub fn parse_bn254_pairing_input(input: &[u8]) -> Result<usize, EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    if !input.len().is_multiple_of(EVM_BN254_PAIRING_ITEM_BYTES) {
        return Err(EvmCoreError::PrecompileInvalidInputLength);
    }
    let mut offset = 0usize;
    let mut pairs = 0usize;
    while offset < input.len() {
        validate_g1_point(input, offset)?;
        read_g2_point(input, offset.saturating_add(64))?;
        offset = offset.saturating_add(EVM_BN254_PAIRING_ITEM_BYTES);
        pairs = pairs.saturating_add(1);
    }
    Ok(pairs)
}

/// Executes the currently admitted EIP-197 BN254 pairing frame.
///
/// Empty input is fully specified by EIP-197 and returns the 32-byte word
/// encoding one. Non-empty inputs are parsed and then fail closed until the G2
/// subgroup and pairing algebra releases are admitted.
pub fn execute_bn254_pairing(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    let target = output
        .get_mut(..EVM_BN254_PAIRING_OUTPUT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)?;
    let pairs = parse_bn254_pairing_input(input)?;
    if pairs != 0 {
        return Err(EvmCoreError::PrecompileBackendUnavailable);
    }
    target.fill(0);
    if let Some(last) = target.last_mut() {
        *last = 1;
    }
    Ok(EVM_BN254_PAIRING_OUTPUT_BYTES)
}

impl EvmPrecompilePlan {
    /// Executes the EIP-197 BN254 pairing frame into `output`.
    pub fn execute_bn254_pairing(
        self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Bn254Pairing {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_bn254_pairing(input, output)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fp2 {
    c0: Fp,
    c1: Fp,
}

impl Fp2 {
    const ZERO: Self = Self {
        c0: Fp::ZERO,
        c1: Fp::ZERO,
    };

    fn is_zero(self) -> bool {
        self.c0.is_zero() && self.c1.is_zero()
    }

    fn add(self, rhs: Self) -> Self {
        Self {
            c0: self.c0.add(rhs.c0),
            c1: self.c1.add(rhs.c1),
        }
    }

    fn mul(self, rhs: Self) -> Self {
        let ac = self.c0.mul(rhs.c0);
        let bd = self.c1.mul(rhs.c1);
        let ad = self.c0.mul(rhs.c1);
        let bc = self.c1.mul(rhs.c0);
        Self {
            c0: ac.sub(bd),
            c1: ad.add(bc),
        }
    }

    fn square(self) -> Self {
        let c0c1 = self.c0.mul(self.c1);
        Self {
            c0: self.c0.square().sub(self.c1.square()),
            c1: c0c1.double(),
        }
    }

    fn curve_b() -> Result<Self, EvmCoreError> {
        let Some(inv_82) = Fp::from_u64(82).invert() else {
            return Err(EvmCoreError::PrecompilePointNotOnCurve);
        };
        Ok(Self {
            c0: Fp::from_u64(27).mul(inv_82),
            c1: Fp::ZERO.sub(Fp::from_u64(3).mul(inv_82)),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct G2Point {
    x: Fp2,
    y: Fp2,
    infinity: bool,
}

impl G2Point {
    fn new(x: Fp2, y: Fp2) -> Result<Self, EvmCoreError> {
        if x.is_zero() && y.is_zero() {
            return Ok(Self {
                x: Fp2::ZERO,
                y: Fp2::ZERO,
                infinity: true,
            });
        }
        let point = Self {
            x,
            y,
            infinity: false,
        };
        if point.is_on_curve()? {
            Ok(point)
        } else {
            Err(EvmCoreError::PrecompilePointNotOnCurve)
        }
    }

    fn is_on_curve(self) -> Result<bool, EvmCoreError> {
        Ok(self.infinity || self.y.square() == self.x.square().mul(self.x).add(Fp2::curve_b()?))
    }
}

fn read_g2_point(input: &[u8], offset: usize) -> Result<G2Point, EvmCoreError> {
    let x = read_fp2(input, offset)?;
    let y = read_fp2(input, offset.saturating_add(64))?;
    G2Point::new(x, y)
}

fn read_fp2(input: &[u8], offset: usize) -> Result<Fp2, EvmCoreError> {
    let imaginary = read_fp(input, offset)?;
    let real = read_fp(input, offset.saturating_add(32))?;
    Ok(Fp2 {
        c0: real,
        c1: imaginary,
    })
}

fn read_fp(input: &[u8], offset: usize) -> Result<Fp, EvmCoreError> {
    Fp::from_be_bytes(read_word(input, offset))
        .ok_or(EvmCoreError::PrecompileFieldElementOutOfRange)
}

fn read_word(input: &[u8], offset: usize) -> [u8; 32] {
    let mut output = [0u8; 32];
    if let Some(available) = input.get(offset..) {
        let len = available.len().min(32);
        if let (Some(source), Some(target)) = (available.get(..len), output.get_mut(..len)) {
            target.copy_from_slice(source);
        }
    }
    output
}
