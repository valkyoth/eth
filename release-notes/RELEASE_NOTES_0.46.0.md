# eth 0.46.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.46.0` adds dependency-free execution for the Frontier SHA-256 and
RIPEMD-160 precompiles. The implementations live in `eth-valkyoth-evm-core`,
preserve the existing bounded precompile planning model, and do not add any new
runtime dependencies.

This is still not full cryptographic precompile coverage. `ecrecover`, modular
exponentiation, BN254, BLAKE2F, KZG point evaluation, and BLS12-381 precompiles
remain explicit bounded fail-closed plans until their own audited release
slices are admitted.

## Added

- First-party `no_std` SHA-256 implementation for precompile address `0x02`.
- First-party `no_std` RIPEMD-160 implementation for precompile address `0x03`,
  including the Ethereum 32-byte left-padded output shape.
- `EvmPrecompileImplementation::NativeSha256`.
- `EvmPrecompileImplementation::NativeRipemd160`.
- `execute_sha256` and `EvmPrecompilePlan::execute_sha256`.
- `execute_ripemd160` and `EvmPrecompilePlan::execute_ripemd160`.
- Known-answer vector tests for empty and short inputs.
- Output-buffer no-mutation tests for too-small hash precompile buffers.
- Wrong-kind and wrong-input-length regression tests for hash precompile plans.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.9.0`.
- `eth` publishes as `0.46.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.9.0`.
- The precompile registry now reports SHA-256 and RIPEMD-160 as native
  implementations instead of backend-required descriptors.
- README, crate version matrix, EVM fork matrix, spec matrix, core
  independence audit, and implementation plan now document native hash
  precompile execution.
- The release gate is available as `scripts/release_0_46_gate.sh`.

## Security Notes

- No third-party cryptographic dependency is admitted by this release.
- Hash precompile execution rejects inputs above `EVM_PRECOMPILE_INPUT_LIMIT`.
- Hash precompile execution checks the 32-byte output buffer before writing, so
  too-small buffers remain unchanged.
- RIPEMD-160 output is explicitly left-padded to 32 bytes, matching Ethereum
  precompile semantics.
- Remaining cryptographic precompiles still fail closed with
  `PrecompileBackendUnavailable`.

## Spec References

- The Frontier SHA-256 and RIPEMD-160 precompiles are part of the canonical
  low-address precompile set.
- EIP-1352 reserves the low precompile/system-contract address range:
  <https://eips.ethereum.org/EIPS/eip-1352>.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo check -p eth --features evm-core`
- `scripts/release_crates.py --check`
- `python3 scripts/test-release-metadata.py`

## Pentest

- Pending. Permanent report will be added as `security/pentest/v0.46.0.md`
  after the release-scope pentest and retest are complete.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.9.0`.
- `eth` publishes as `0.46.0`.
- Other support crates are unchanged and are not republished.
