use crate::{EvmCoreError, EvmPrecompileKind, EvmPrecompilePlan, execute_modexp};

impl EvmPrecompilePlan {
    /// Executes the bounded first-party ModExp precompile into `output`.
    pub fn execute_modexp(self, input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Modexp {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_modexp(input, output)
    }
}
