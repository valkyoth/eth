use crate::{EvmCoreError, EvmGasMeter, EvmPrecompileKind, EvmPrecompilePlan, execute_modexp};

impl EvmPrecompilePlan {
    /// Executes the bounded first-party ModExp precompile into `output`.
    ///
    /// This method charges this exact plan's precompile gas before ModExp
    /// validation or exponentiation is reachable.
    pub fn execute_modexp(
        self,
        gas_meter: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Modexp {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        let gas_cost = self
            .gas_cost()
            .ok_or(EvmCoreError::PrecompileBackendUnavailable)?;
        gas_meter.charge(gas_cost)?;
        execute_modexp(input, output)
    }
}
