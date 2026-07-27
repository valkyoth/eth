# Release Notes - eth v0.53.0

Status: implementation complete; awaiting clean retest.

## Summary

This release removes hardwired linear warm-access tracking from native state
execution, adds a bounded fixed-width radix node profile, and introduces
explicit transaction execution-resource authority.

## Added

- `EvmAccessTracker` with explicit embedded-linear and node-fixed-width-radix
  profiles.
- Pre-reserved compressed-radix address/storage indexes and bounded undo
  journals with no allocation after construction.
- Nested LIFO scope commit/rollback, atomic address+slot insertion,
  Prague-aware gas-derived hard ceilings, and sanitizing
  allocation-retaining transaction reset.
- EVM host adapters for both tracker profiles.
- `ExecutionResourceLimits`, `ExecutionGovernor`, and non-copyable
  `ExecutionWorkToken` capabilities.
- Adversarial sorted/reverse benchmark, differential profile tests, and an
  access-tracker fuzz target.

## Changed

- `EvmExecution::run_with_state` accepts any `EvmAccessTracker`.
- The compatibility `EvmAccessSet` name now aliases the explicit embedded
  profile.
- The facade adds `evm-node` and `evm-core-node` allocator-backed features;
  default `evm` and `evm-core` paths remain `no_std` and allocation-free.
- The point-in-time REVM non-admission record is refreshed against
  `revm 42.0.1` and `revm-primitives 42.0.0`; neither is admitted.

## Security Properties

- Node-scale distinct access patterns no longer incur undocumented quadratic
  membership work.
- Constructor-time fallible reservation fixes the retained allocation bound
  before untrusted execution begins.
- Capacity failure cannot leave half of an address/storage pair warm.
- Failed and reverted scopes cannot leak newly warmed entries into their
  parent or a retry.
- Governor use before destructive reset fails closed; cumulative and reusable
  high-water resources use distinct public types.
- Failed or cancelled work remains charged, and child delegation cannot create
  additional authority.
- Gas-derived tracker ceilings use exact floor division with compile-time and
  runtime drift checks.
- Generation exhaustion is validated before destructive reset mutates any
  budget state.
- Abstract work authority is capped at the reviewed one-million-step ceiling.
- Node tracker rollback, reset, and drop erase retained address and storage-key
  bytes through `eth-valkyoth-sanitization`.
- Node tracker lookup and insertion are bounded by the 160-bit address or
  416-bit address/storage key width. Rollback touches only unique insertions
  made after its checkpoint and never rebuilds retained outer-scope indexes.

The governor is an explicit capability. Integrators must route governed host
operations through it; the complete node operational binding remains assigned
to `v0.65.0`.

## Versioning

- `eth-valkyoth-evm-core` advances from `0.26.1` to `0.27.0`.
- `eth-valkyoth-evm` advances from `0.11.0` to `0.12.0`.
- `eth` advances from `0.52.7` to `0.53.0`.
- All other support crates remain unchanged.

## Pentest

Three external review rounds reported eight findings: the initial two Low
findings, then one High, three Medium, and one Low finding, followed by one
High rollback-complexity finding. They cover child-scope warmth rollback,
resource accounting, work authority, initialized warmth, retained key erasure,
and retained-index rebuild amplification. All are remediated. Tagging remains
blocked until the exact remediation commit passes clean retest, release-gate
validation, and green GitHub CI/CodeQL. The final report will be stored at
`security/pentest/v0.53.0.md`.
