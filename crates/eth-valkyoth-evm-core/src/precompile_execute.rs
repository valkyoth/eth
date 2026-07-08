use crate::{
    EvmCoreError, EvmPrecompileKind, EvmPrecompilePlan, execute_identity, execute_ripemd160,
    execute_sha256,
};

impl EvmPrecompilePlan {
    /// Executes the dependency-free identity precompile into `output`.
    pub fn execute_identity(self, input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Identity {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_identity(input, output)
    }

    /// Executes the dependency-free SHA-256 precompile into `output`.
    pub fn execute_sha256(self, input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Sha256 {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_sha256(input, output)
    }

    /// Executes the dependency-free RIPEMD-160 precompile into `output`.
    pub fn execute_ripemd160(self, input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
        if self.descriptor().kind != EvmPrecompileKind::Ripemd160 {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        execute_ripemd160(input, output)
    }
}
