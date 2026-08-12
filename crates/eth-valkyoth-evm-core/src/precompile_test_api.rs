use crate::precompile_authorization::PaidPrecompile;
use crate::{
    EvmBlake2F, EvmBn254Add, EvmBn254Mul, EvmBn254Pairing, EvmCoreError, EvmEcRecover,
    EvmEcRecoverBackend, EvmExecutablePrecompile, EvmGasMeter, EvmIdentity, EvmModExpWorkspace,
    EvmModexp, EvmPrecompileKeccak256, EvmPrecompileOutcome, EvmPrecompilePlan,
    EvmPrecompileStatus, EvmRipemd160, EvmSha256,
};

impl EvmPrecompilePlan {
    fn test_authorize<'input, 'meter, 'output, K: EvmExecutablePrecompile>(
        self,
        gas: &'meter mut EvmGasMeter,
        input: &'input [u8],
        output: &'output mut [u8],
    ) -> Result<PaidPrecompile<'input, 'meter, 'output, K>, EvmCoreError> {
        if self.descriptor().kind != K::KIND {
            return Err(EvmCoreError::PrecompileBackendUnavailable);
        }
        if input.len() != self.input_len() {
            return Err(EvmCoreError::PrecompileInvalidInputLength);
        }
        let quote = self.descriptor().quote::<K>(input)?;
        if self.gas_cost() != Some(quote.gas_cost()) {
            return Err(EvmCoreError::PrecompilePlanInputMismatch);
        }
        quote.authorize_internal(gas, output)
    }

    pub(crate) fn execute_identity(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        outcome(
            self.test_authorize::<EvmIdentity>(gas, input, output)?
                .execute_identity(),
        )
    }

    pub(crate) fn execute_sha256(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        outcome(
            self.test_authorize::<EvmSha256>(gas, input, output)?
                .execute_sha256(),
        )
    }

    pub(crate) fn execute_ripemd160(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        outcome(
            self.test_authorize::<EvmRipemd160>(gas, input, output)?
                .execute_ripemd160(),
        )
    }

    pub(crate) fn execute_modexp(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        let required = crate::modexp_workspace_limbs(input)?;
        let mut storage = std::vec![0_u32; required];
        let mut workspace = EvmModExpWorkspace::new(&mut storage);
        outcome(
            self.test_authorize::<EvmModexp>(gas, input, output)?
                .execute_modexp(&mut workspace),
        )
    }

    pub(crate) fn execute_bn254_add(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        outcome(
            self.test_authorize::<EvmBn254Add>(gas, input, output)?
                .execute_bn254_add(),
        )
    }

    pub(crate) fn execute_bn254_mul(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        outcome(
            self.test_authorize::<EvmBn254Mul>(gas, input, output)?
                .execute_bn254_mul(),
        )
    }

    pub(crate) fn execute_bn254_pairing(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        outcome(
            self.test_authorize::<EvmBn254Pairing>(gas, input, output)?
                .execute_bn254_pairing(),
        )
    }

    pub(crate) fn execute_blake2f(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, EvmCoreError> {
        outcome(
            self.test_authorize::<EvmBlake2F>(gas, input, output)?
                .execute_blake2f(),
        )
    }

    pub(crate) fn execute_ecrecover<B, H>(
        self,
        gas: &mut EvmGasMeter,
        input: &[u8],
        output: &mut [u8],
        backend: B,
        hasher: H,
    ) -> Result<usize, EvmCoreError>
    where
        B: EvmEcRecoverBackend,
        H: EvmPrecompileKeccak256,
    {
        outcome(
            self.test_authorize::<EvmEcRecover>(gas, input, output)?
                .execute_ecrecover(backend, hasher),
        )
    }
}

fn outcome(outcome: EvmPrecompileOutcome) -> Result<usize, EvmCoreError> {
    if outcome.status() == EvmPrecompileStatus::Success {
        Ok(outcome.output_len())
    } else {
        Err(outcome
            .error()
            .unwrap_or(EvmCoreError::PrecompileBackendUnavailable))
    }
}
