use crate::{EvmCoreError, EvmGas, EvmGasMeter, EvmPrecompileKind, EvmPrecompilePlan};

/// Proof that a precompile plan's gas cost has been charged.
///
/// The fields are private so dispatcher integrations cannot forge this token.
/// Construct it with [`EvmPrecompilePlan::charge_gas`] immediately before
/// executing the matching precompile plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmPrecompileGasCharge {
    kind: EvmPrecompileKind,
    input_len: usize,
    gas_cost: EvmGas,
}

impl EvmPrecompileGasCharge {
    /// Returns the precompile kind this charge token belongs to.
    #[must_use]
    pub const fn kind(self) -> EvmPrecompileKind {
        self.kind
    }

    /// Returns the planned input length this charge token belongs to.
    #[must_use]
    pub const fn input_len(self) -> usize {
        self.input_len
    }

    /// Returns the gas charged to create this token.
    #[must_use]
    pub const fn gas_cost(self) -> EvmGas {
        self.gas_cost
    }
}

impl EvmPrecompilePlan {
    /// Charges this plan's gas cost and returns the execution token.
    ///
    /// Precompile dispatcher integrations should call this before reaching any
    /// plan-level execution method. Precompiles with deferred gas formulas
    /// cannot be executed through this release's plan boundary.
    pub fn charge_gas(
        self,
        gas_meter: &mut EvmGasMeter,
    ) -> Result<EvmPrecompileGasCharge, EvmCoreError> {
        let gas_cost = self
            .gas_cost()
            .ok_or(EvmCoreError::PrecompileBackendUnavailable)?;
        gas_meter.charge(gas_cost)?;
        Ok(EvmPrecompileGasCharge {
            kind: self.descriptor().kind,
            input_len: self.input_len(),
            gas_cost,
        })
    }
}
