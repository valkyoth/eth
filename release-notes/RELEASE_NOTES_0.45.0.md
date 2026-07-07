# eth 0.45.0 Release Notes

Status: release candidate; pentest clean; awaiting final GitHub checks.

`0.45.0` adds the native EVM precompile registry boundary. The release admits
precompile addresses as fork-aware descriptors, validates bounded input shape,
computes gas where the current boundary has enough audited policy, and executes
only the dependency-free identity precompile.

This is not full cryptographic precompile execution. SHA-256, RIPEMD-160,
ecrecover, modular exponentiation, BN254, BLAKE2F, KZG point evaluation, and
BLS12-381 precompiles are exposed as explicit bounded plans and fail closed
until reviewed backends or first-party implementations are admitted.

## Added

- `EvmPrecompileKind` for Frontier, Byzantium, Istanbul, Cancun, and Prague
  precompile identities.
- `EvmPrecompileRegistry` for fork-aware address lookup and descriptor
  construction.
- `EvmPrecompileDescriptor` with address, registry fork, introduction fork,
  implementation boundary, input policy, gas policy, and fixed output length
  where known.
- `EvmPrecompilePlan` with prevalidated input length and optional gas cost.
- `EvmPrecompileImplementation`, `EvmPrecompileInputPolicy`, and
  `EvmPrecompileGasPolicy` domains.
- `EVM_PRECOMPILE_INPUT_LIMIT` as the hard planning ceiling.
- Dependency-free identity execution through `execute_identity` and
  `EvmPrecompilePlan::execute_identity`.
- Deterministic precompile error codes:
  - `precompile_not_available_in_fork`;
  - `precompile_input_too_large`;
  - `precompile_invalid_input_length`;
  - `precompile_gas_overflow`;
  - `precompile_output_too_small`;
  - `precompile_backend_unavailable`.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.8.0`.
- `eth` publishes as `0.45.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.8.0`.
- The README, crate version matrix, EVM fork matrix, spec matrix, and
  implementation plan now document the precompile registry boundary.
- The release gate is available as `scripts/release_0_45_gate.sh`.

## Security Notes

- No new third-party cryptographic dependency is admitted in this release.
- Cryptographic precompile descriptors are planning surfaces only and return
  `PrecompileBackendUnavailable` for execution.
- Precompile planning rejects inputs larger than the release hard limit before
  gas calculation or execution.
- Identity execution checks output capacity before copying and does not require
  allocation.
- BN254 pairing planning enforces 192-byte input multiples and distinguishes
  Byzantium and Istanbul gas pricing.
- BLAKE2F planning enforces the 213-byte input shape and derives gas from the
  first four bytes as the round count.
- KZG point-evaluation planning enforces the 192-byte input shape and fixed
  50,000 gas, but does not verify proofs in this release.

## Spec References

- EIP-1352 reserves the low precompile/system-contract address range:
  <https://eips.ethereum.org/EIPS/eip-1352>.
- EIP-196 and EIP-197 define the Byzantium BN254 precompiles:
  <https://eips.ethereum.org/EIPS/eip-196> and
  <https://eips.ethereum.org/EIPS/eip-197>.
- EIP-198 defines modular exponentiation at address `0x05`:
  <https://eips.ethereum.org/EIPS/eip-198>.
- EIP-152 defines BLAKE2F at address `0x09`:
  <https://eips.ethereum.org/EIPS/eip-152>.
- EIP-4844 defines the KZG point-evaluation precompile:
  <https://eips.ethereum.org/EIPS/eip-4844>.
- EIP-2537 defines the BLS12-381 precompile address range:
  <https://eips.ethereum.org/EIPS/eip-2537>.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo check -p eth --features evm-core`
- `scripts/release_crates.py --check`
- `scripts/release_0_45_gate.sh`

## Pentest

- Permanent report: `security/pentest/v0.45.0.md`.
- Initial pentest reported no critical, high, or medium findings.
- Informational items covered future CALL precompile dispatch wiring,
  BLAKE2F final-block flag validation, and additional tests for deferred gas
  and known-address lookup.
- The test coverage items were remediated, and the forward-looking items were
  pinned to concrete roadmap releases.
- Retest was clean. No blocking findings remain for the v0.45.0 release scope.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.8.0`.
- `eth` publishes as `0.45.0`.
- Other support crates are unchanged and are not republished.
