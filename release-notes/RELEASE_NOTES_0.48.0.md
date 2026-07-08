# eth 0.48.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.48.0` adds bounded native ModExp precompile execution to
`eth-valkyoth-evm-core`. The implementation keeps the crate dependency-free and
`no_std` by parsing EIP-198 ModExp calldata, computing fork-aware EIP-198 and
EIP-2565 gas, and executing modular exponentiation with fixed-size stack
buffers.

This release intentionally uses an explicit operand cap:
`EVM_MODEXP_MAX_OPERAND_BYTES`. Inputs that declare larger base, exponent, or
modulus lengths are rejected before execution. That cap keeps the first
first-party bigint pass reviewable and prevents hostile calldata from causing
unbounded CPU or memory use.

## Added

- `EVM_MODEXP_HEADER_BYTES` for the 96-byte ModExp length header.
- `EVM_MODEXP_MAX_OPERAND_BYTES` for the release operand cap.
- `EvmModExpInput` and `parse_modexp_input` for EIP-198 length parsing with
  right-padding semantics.
- `execute_modexp` and `EvmPrecompilePlan::execute_modexp`.
- Fork-aware ModExp gas calculation:
  - Byzantium through Istanbul use the EIP-198 formula.
  - Berlin and later use the EIP-2565 formula and 200 gas floor.
- `fuzz/fuzz_targets/modexp_frame.rs` for ModExp header, length, and bounded
  execution fuzz coverage.
- Tests for the EIP-198 Fermat example, Berlin EIP-2565 gas, zero modulus,
  short-exponent gas, empty modulus length, right-padding, oversized operands,
  short output, and wrong-plan dispatch.

## Changed

- `eth-valkyoth-evm-core` publishes as `0.11.0`.
- `eth` publishes as `0.48.0` and points its optional `evm-core` feature at
  `eth-valkyoth-evm-core 0.11.0`.
- The precompile registry now reports ModExp as native bounded execution
  instead of deferred dynamic gas/backend-unavailable execution.
- README, crate version matrix, EVM fork matrix, spec matrix, implementation
  plan, and release plan now document bounded ModExp execution.
- The release gate is available as `scripts/release_0_48_gate.sh`.

## Security Notes

- No bigint, crypto, or allocator dependency is added.
- ModExp checks declared operand lengths before execution and rejects operands
  above `EVM_MODEXP_MAX_OPERAND_BYTES`.
- ModExp checks the output buffer before executing the bigint loop.
- ModExp treats calldata as right-padded, matching EIP-198.
- ModExp gas calculation now has targeted coverage for short declared exponent
  widths, including one-byte exponent and zero-exponent cases.
- Zero modulus returns a zero-filled output of the declared modulus length.
- `execute_modexp` is documented as public-EVM-calldata arithmetic and not
  constant-time secret-key arithmetic.
- Remaining cryptographic precompiles still fail closed with
  `PrecompileBackendUnavailable`.

## Spec References

- EIP-198 defines the ModExp precompile input shape, right-padding behavior,
  output length, and Byzantium gas formula:
  <https://eips.ethereum.org/EIPS/eip-198>.
- EIP-2565 defines the Berlin ModExp gas formula and 200 gas floor:
  <https://eips.ethereum.org/EIPS/eip-2565>.

## Verification

- `cargo fmt --all --check`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo check --manifest-path fuzz/Cargo.toml --bin modexp_frame`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin modexp_frame -- -D warnings`
- `cargo check -p eth --features evm-core`
- `scripts/release_crates.py --check`
- `python3 scripts/test-release-metadata.py`

## Pentest

- Pending. Permanent report will be added as `security/pentest/v0.48.0.md`
  after the release-scope pentest and retest are complete.

## Versioning

- `eth-valkyoth-evm-core` publishes as `0.11.0`.
- `eth` publishes as `0.48.0`.
- Other support crates are unchanged and are not republished.
