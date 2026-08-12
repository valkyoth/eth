# Release Notes - eth v0.55.0

Status: pentest findings remediated; awaiting exact-commit retest.

## Summary

This release removes the first-party ModExp engine's private 64-byte operand
ceiling. Supported Byzantium-through-Prague calls preserve all declared
lengths as 256-bit values through fork gas admission and execute every payable
EIP-198/EIP-2565 frame with streamed right-padding and caller-owned workspace.

## Added

- `EvmModExpLength` for un-narrowed 256-bit base, exponent, and modulus
  lengths.
- `EvmModExpWorkspace` and quote-specific `modexp_workspace_limbs()` sizing.
- Dependency-free arbitrary-length limb arithmetic with schoolbook
  multiplication, normalized long division, and binary exponentiation.
- Virtual segment reads for missing right-padded base, exponent, and modulus
  bytes without attacker-sized allocation.
- An independent `num-bigint 0.5.1` dev-only differential test from 1 through
  256-byte operands.
- A fail-closed Podman differential gate comparing 11 deterministic ModExp
  outputs against immutable latest-stable Geth `1.17.5`, Besu `26.7.1`, and
  Nethermind `1.39.3` images.
- Wide-length, workspace atomicity, zero-output/unrepresentable-exponent,
  80-byte KAT, and deterministic `u128` oracle coverage.
- Berlin-priced dense 64/256-byte plus sparse and zero-exponent 1,024-byte
  release-mode ModExp work-per-gas benchmarks.
- The [ModExp precompile contract](../docs/modexp-precompile.md).

## Changed

- `parse_modexp_input` no longer narrows or rejects declared lengths based on
  host `usize` or a release operand cap.
- Payable ModExp execution requires explicit caller-owned workspace. The
  atomic execution method now accepts `&mut EvmModExpWorkspace`.
- Canonical gas above `EVM_MAX_GAS_LIMIT` is represented by an unpayable
  one-over-limit quote, ensuring every constructible meter returns `OutOfGas`
  before host-size conversion or arithmetic.
- Arithmetic completes in workspace before copying the final output, keeping
  host-capacity failures output-atomic.
- The differential runner now compiles and runs both structural RLP and ModExp
  independent reference paths.
- The ModExp fuzz target exercises wide declarations and caps only its own
  execution allocations, not parser or gas coverage.
- Result serialization preserves the full declared modulus width when the
  significant modulus occupies fewer limbs, including 4/8/12-byte leading
  zero boundaries.
- Exponentiation skips leading zero bits, returns directly for exponent zero,
  and initializes from the reduced base at the highest set bit so work tracks
  the adjusted exponent length charged by EIP-198/EIP-2565.
- Payable fuzz frames now require successful execution rather than silently
  accepting unexpected call failures.
- The external-client gate verifies official latest-release metadata, runtime
  client identity, loopback-only RPC, and byte-exact output before accepting
  evidence; containers have no host mounts and no outbound network.
- Pentest remediation adds mandatory CPU, memory, PID, filesystem, tmpfs, and
  log ceilings; bounded subprocess/cleanup waits and HTTP reads; atomic
  Podman-assigned ports; proxy-free loopback RPC; and sanitized infrastructure
  failures without tracebacks or command arguments.
- Follow-up remediation verifies rootless Podman explicitly, replaces
  process-derived names with 128-bit random ownership identifiers, captures
  immutable container and network IDs before client execution, and fails the
  gate unless every owned object is confirmed removed.
- Every external-client object now carries the random run token as an ownership
  label. A timed-out or failed create triggers label-checked recovery, while an
  absent object, an inspection failure, malformed metadata, or a wrong owner
  remains a distinct fail-closed outcome.

## Scope

This release implements EIP-198 and EIP-2565 behavior through the currently
supported Prague fork table. Osaka EIP-7823 input limits and EIP-7883 gas
changes remain explicit deliverables of `v0.115.0..=v0.116.0`; this release
does not claim Osaka execution.

The arithmetic handles public EVM values and is not constant-time. It must not
be reused for private RSA exponents or other secret-dependent arithmetic.

Official sources were refreshed and cleanly synchronized on 2026-08-12:
execution-specs `2867859a3c19b925f7dc47dae648cca9758f4f80`, execution tests
`c67e485ff8b5be9abc8ad15345ec21aa22e290d9`, EIPs
`582684e2d7d372c09f45777be8ea603e485e9e9d`, execution APIs
`742d45db810b31265c8d3c075af324953330d1ed`, and consensus specs
`6d0e95d972a90bbf79a356ded6a704d769bb67c0`.

## Breaking Changes

- `EvmModExpInput::{base_len, exponent_len, modulus_len}` now return
  `EvmModExpLength` rather than `usize`.
- `authorize_and_execute_modexp` requires caller-owned workspace.
- The obsolete `EVM_MODEXP_MAX_OPERAND_BYTES` constant is removed.

These pre-1.0 changes are required to remove the private consensus boundary.

## Versioning

- `eth-valkyoth-evm-core` advances from `0.28.0` to `0.29.0` for the public
  ModExp API and arithmetic change.
- `eth-valkyoth-evm` advances from `0.12.1` to `0.12.2` for its published
  EVM-core dependency requirement.
- `eth` advances from `0.54.0` to `0.55.0`.
- All other support crates remain unchanged.

## Pentest

Tagging is blocked until the exact implementation commit is externally
pentested, every finding is remediated, a clean retest is recorded at
`security/pentest/v0.55.0.md`, and GitHub CI plus CodeQL are green.
