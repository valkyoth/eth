use crate::{
    EvmCoreError, EvmGasMeter, EvmPrecompileKind, EvmPrecompilePlan,
    precompile::{execute_identity, execute_ripemd160, execute_sha256},
};

impl EvmPrecompilePlan {
    /// Executes the dependency-free identity precompile into `output`.
    pub fn execute_identity(
        self,
        gas_meter: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Identity {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        charge_plan(self, gas_meter, input)?;
        execute_identity(input, output)
    }

    /// Executes the dependency-free SHA-256 precompile into `output`.
    pub fn execute_sha256(
        self,
        gas_meter: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Sha256 {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        charge_plan(self, gas_meter, input)?;
        execute_sha256(input, output)
    }

    /// Executes the dependency-free RIPEMD-160 precompile into `output`.
    pub fn execute_ripemd160(
        self,
        gas_meter: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Ripemd160 {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        charge_plan(self, gas_meter, input)?;
        execute_ripemd160(input, output)
    }
}

fn charge_plan(
    plan: EvmPrecompilePlan,
    gas_meter: &mut EvmGasMeter,
    input: &[u8],
) -> Result<(), EvmCoreError> {
    let gas_cost = plan.checked_execution_cost(input)?;
    gas_meter.charge(gas_cost)
}
