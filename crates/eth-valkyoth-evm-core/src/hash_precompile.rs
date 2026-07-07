use crate::{EVM_PRECOMPILE_INPUT_LIMIT, EvmCoreError, ripemd160, sha256};

const HASH_OUTPUT_BYTES: usize = 32;
const RIPEMD160_PADDING_BYTES: usize = 12;

pub(crate) fn execute_sha256(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    prepare_hash_output(input, output)?;
    let digest = sha256::digest(input);
    copy_digest(&digest, output)?;
    Ok(HASH_OUTPUT_BYTES)
}

pub(crate) fn execute_ripemd160(input: &[u8], output: &mut [u8]) -> Result<usize, EvmCoreError> {
    prepare_hash_output(input, output)?;
    let digest = ripemd160::digest(input);
    let target = hash_output(output)?;
    target.fill(0);
    if let Some(suffix) = target.get_mut(RIPEMD160_PADDING_BYTES..) {
        suffix.copy_from_slice(&digest);
    }
    Ok(HASH_OUTPUT_BYTES)
}

fn prepare_hash_output(input: &[u8], output: &[u8]) -> Result<(), EvmCoreError> {
    if input.len() > EVM_PRECOMPILE_INPUT_LIMIT {
        return Err(EvmCoreError::PrecompileInputTooLarge);
    }
    hash_output_read(output)?;
    Ok(())
}

fn copy_digest(digest: &[u8; HASH_OUTPUT_BYTES], output: &mut [u8]) -> Result<(), EvmCoreError> {
    let target = hash_output(output)?;
    target.copy_from_slice(digest);
    Ok(())
}

fn hash_output(output: &mut [u8]) -> Result<&mut [u8], EvmCoreError> {
    output
        .get_mut(..HASH_OUTPUT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)
}

fn hash_output_read(output: &[u8]) -> Result<&[u8], EvmCoreError> {
    output
        .get(..HASH_OUTPUT_BYTES)
        .ok_or(EvmCoreError::PrecompileOutputTooSmall)
}
