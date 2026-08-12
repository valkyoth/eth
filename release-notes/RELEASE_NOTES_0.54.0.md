# Release Notes - eth v0.54.0

Status: implementation complete; pentest and clean retest required.

## Summary

This release replaces directly executable precompile plans with immutable
exact-input gas quotes, canonical descriptor validation, non-forgeable
one-shot paid capabilities, and precise CALL-ready outcomes.

## Added

- Sealed marker types for every native precompile admitted in this release.
- `EvmPrecompileGasQuote<'input, K>` with an immutable exact-input borrow.
- `PaidPrecompile<'input, 'meter, 'output, K>` as the only production
  native-execution authority, borrowing the exact admitted output buffer.
- `EvmPrecompileOutcome` and `EvmPrecompileStatus` for success, call failure,
  gas consumed, output length, bounded error, and rollback decisions.
- Compile-fail tests for quoted-input mutation and paid-authority duplication.
- Forged-descriptor, marker mismatch, output preflight, atomic out-of-gas,
  post-payment failure, redacted-debug, and gas-derived input-bound tests.
- Public-input identity, SHA-256, and BN254 multiplication work-per-gas
  benchmark evidence.
- A dedicated [precompile authorization contract](../docs/precompile-authorization.md).

## Changed

- `EvmPrecompilePlan` is informational only and has no production execution
  methods.
- Native identity, SHA-256, RIPEMD-160, ECRECOVER, bounded ModExp, BN254
  add/mul/pairing, and BLAKE2F execution consumes a paid capability.
- The complete descriptor is compared with the canonical fork registry before
  quote admission, preventing caller-modified gas or backend metadata.
- Output capacity is admitted before gas charging or output mutation.
- Post-payment execution failure consumes all gas in the dedicated child-call
  meter and reports that CALL rollback is required.
- Protocol gas replaces the former release-wide precompile calldata ceiling.
  Exact frame and tuple rules remain enforced, and bounded ModExp retains its
  separate operand cap until `v0.55.0`.
- Native precompile fuzz targets use the quote/authorize/outcome path.
- The compile-fail test harness uses `trybuild 1.0.120`; compatible transitive
  lockfile patches and the generated SBOM are current at the implementation
  stop.

## Integration Requirement

The meter passed to `authorize` must be scoped to gas supplied to that
precompile CALL. A transaction-global unrestricted meter would be consumed in
full on a post-payment execution failure and is therefore not a valid input to
this API.

## Breaking Changes

Direct `EvmPrecompilePlan::execute_*` methods are removed. Integrators must
request the matching typed quote from a canonical descriptor, authorize it
against a dedicated gas meter and output capacity, then consume the returned
paid capability exactly once.

## Versioning

- `eth-valkyoth-evm-core` advances from `0.27.0` to `0.28.0` for the public
  execution-authority API change.
- `eth-valkyoth-evm` advances from `0.12.0` to `0.12.1` to update its published
  core dependency requirement.
- `eth` advances from `0.53.0` to `0.54.0`.
- All other support crates remain unchanged.

## Pentest

The implementation stop must be pentested at its exact commit. Every finding
must be remediated and independently retested before the permanent report-only
commit and release tag.
