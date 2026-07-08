# eth 0.47.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.47.0` adds native ECRECOVER precompile execution to
`eth-valkyoth-evm-core`. The implementation keeps the core crate free of
default secp256k1 and Keccak dependencies by requiring explicit caller-provided
backend traits for public-key recovery and address derivation.

This release deliberately does not apply transaction low-s policy to the
precompile. EIP-2 made high-s transaction signatures invalid, but left the
ECDSA recover precompile unchanged. The native ECRECOVER implementation
therefore accepts full nonzero secp256k1 scalar `s` values below the curve
order, while transaction validation continues to enforce low-s elsewhere.

## Added

- `EVM_ECRECOVER_INPUT_BYTES` for the canonical 128-byte ECRECOVER frame.
- `EVM_ECRECOVER_PUBLIC_KEY_BYTES` for the 64-byte uncompressed public-key
  payload.
- `EvmEcRecoverSignature`, exposing parsed `r`, `s`, and normalized y parity.
- `EvmEcRecoverBackend` for caller-provided recoverable secp256k1 backends.
- `EvmPrecompileKeccak256` for caller-provided Ethereum Keccak-256 address
  derivation.
- `execute_ecrecover` and `EvmPrecompilePlan::execute_ecrecover`.
- Tests for valid backend execution, high-s acceptance, invalid-v handling,
  invalid scalar handling, output-buffer safety, and wrong-plan rejection.
- `fuzz/fuzz_targets/ecrecover_frame.rs` to exercise the untrusted calldata
  frame parser with deterministic caller-provided backend stubs.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.10.0`.
- `eth` publishes as `0.47.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.10.0`.
- The precompile registry now reports ECRECOVER as a native implementation
  requiring caller-provided backends instead of a fail-closed backend-required
  descriptor.
- README, crate version matrix, EVM fork matrix, spec matrix, implementation
  plan, and release plan now document native ECRECOVER execution.
- The release gate is available as `scripts/release_0_47_gate.sh`.

## Security Notes

- No default secp256k1 or Keccak dependency is added.
- ECRECOVER checks the output buffer before parsing or calling backends.
- ECRECOVER canonicalizes calldata using the Ethereum precompile frame shape:
  the first 128 bytes are used, shorter input is right-padded with zeroes, and
  extra input bytes are ignored after planning.
- Invalid `v`, invalid `r`, invalid `s`, or backend recovery failure returns
  zero-length output without mutating the output buffer.
- ECRECOVER accepts high-s signatures by design because EIP-2 explicitly kept
  the precompile behavior unchanged.
- The ECRECOVER backend trait documents that precompile implementations must
  not reuse transaction-sender recovery helpers that reject high-s signatures.
- Remaining cryptographic precompiles still fail closed with
  `PrecompileBackendUnavailable`.

## Spec References

- EIP-2 distinguishes transaction low-s validity from unchanged ECRECOVER
  precompile behavior:
  <https://eips.ethereum.org/EIPS/eip-2>.
- EIP-1352 reserves the low precompile/system-contract address range:
  <https://eips.ethereum.org/EIPS/eip-1352>.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo check --manifest-path fuzz/Cargo.toml --bin ecrecover_frame`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin ecrecover_frame -- -D warnings`
- `cargo check -p eth --features evm-core`
- `scripts/release_crates.py --check`
- `python3 scripts/test-release-metadata.py`

## Pentest

- Pending. Permanent report will be added as `security/pentest/v0.47.0.md`
  after the release-scope pentest and retest are complete.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.10.0`.
- `eth` publishes as `0.47.0`.
- Other support crates are unchanged and are not republished.
