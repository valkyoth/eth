# Release Notes - eth v0.53.0

Status: implementation complete; awaiting pentest.

## Summary

This release removes hardwired linear warm-access tracking from native state
execution, adds a bounded logarithmic node profile, and introduces explicit
transaction execution-resource authority.

## Added

- `EvmAccessTracker` with explicit embedded-linear and node-logarithmic
  profiles.
- Pre-reserved AVL address/storage tables with no allocation after
  construction and worst-case `O(log n)` operations.
- Root attempt commit/rollback, atomic address+slot insertion, gas-derived hard
  ceilings, and allocation-retaining transaction reset.
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
- Failed and reverted root attempts cannot leak warmth into a retry.
- Governor use before destructive reset fails closed; reusable capacity uses a
  high-water observation rather than cumulative-call accounting.
- Failed or cancelled work remains charged, and child delegation cannot create
  additional authority.

The governor is an explicit capability. Integrators must route governed host
operations through it; the complete node operational binding remains assigned
to `v0.65.0`.

## Versioning

- `eth-valkyoth-evm-core` advances from `0.26.1` to `0.27.0`.
- `eth-valkyoth-evm` advances from `0.11.0` to `0.12.0`.
- `eth` advances from `0.52.7` to `0.53.0`.
- All other support crates remain unchanged.

## Pentest

Tagging is blocked until the exact implementation commit passes external
pentesting, remediation, clean retest, release-gate validation, and green
GitHub CI/CodeQL. The final report will be stored at
`security/pentest/v0.53.0.md`.
