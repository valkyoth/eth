# Precompile Authorization Contract

Status: implemented for native precompiles in `v0.54.0`; release pentest is
required before tagging.

## Purpose

Native precompile arithmetic must not be reachable through an uncharged plan,
forged descriptor, or input that changed after its gas was calculated. The
`v0.54.0` boundary therefore separates four states:

1. `EvmPrecompileDescriptor` identifies fork-scoped consensus metadata.
2. `EvmPrecompileGasQuote<'input, K>` validates the complete descriptor against
   the canonical registry and immutably borrows the exact input.
3. `authorize` borrows and admits the exact output buffer, then charges gas
   before creating a non-forgeable, one-shot, fail-closed
   `PaidPrecompile<'input, 'meter, 'output, K>`; preferred atomic methods keep
   this armed state internal.
4. Consuming the paid capability produces one `EvmPrecompileOutcome` for CALL
   integration.

`EvmPrecompilePlan` remains informational planning metadata for executable and
future fail-closed precompiles. It has no production execution methods and
cannot create paid work.

## Exact Input Binding

The quote stores `&[u8]`, not an implementation-defined checksum. Rust's
immutable borrow prevents safe code from mutating or replacing the calldata
while the quote or paid capability remains live. Authorization also stores the
exact `&mut [u8]` output borrow, so capacity cannot be asserted for one buffer
and execution redirected into another. This avoids relying on a
non-cryptographic fingerprint before first-party Keccak is available.

The quote type and paid type do not implement `Clone` or `Copy`. Their fields
and constructors are private. The paid type and terminal outcome are
`must_use`. Compile-fail tests prove that quoted input cannot be mutated, paid
authority cannot be duplicated, and a terminal outcome cannot be silently
discarded under the repository's denied-warning policy.

## Admission Order

Authorization performs these operations in order:

1. compare the entire descriptor with the canonical descriptor for its fork;
2. require the sealed marker type to match the descriptor and backend;
3. validate cheap input framing and calculate consensus gas;
4. calculate the backend's required output capacity;
5. reject insufficient output without charging or mutation;
6. charge the exact gas quote atomically;
7. expose expensive parsing, curve, subgroup, hashing, or arithmetic work.

There is no release-wide precompile calldata ceiling. Exact and tuple-framed
precompiles retain their consensus shape rules. Variable work is admitted by
its protocol gas formula and the caller's gas meter. The bounded ModExp engine
retains its separately documented operand limit until `v0.55.0`.

## CALL Gas Scope

The meter passed to `authorize` **must represent only the gas supplied to this
precompile CALL**. It may already account setup performed inside that same
child scope, but it must not be the parent transaction's unrestricted meter.

| Stage | Result | Gas | Output |
| --- | --- | --- | --- |
| Quote or output admission fails | Rust `Err` | unchanged | unchanged |
| Gas charge fails | `OutOfGas` | unchanged | unchanged |
| Paid execution succeeds | `Success` | exact quoted gas | valid prefix reported |
| Paid execution fails | `CallFailure` | all child-supplied gas | zero valid bytes; child rollback required |
| Armed capability is dropped or unwinds | no outcome available | all child-supplied gas | no valid bytes; caller must fail the interrupted CALL |

Every paid capability carries an armed drop guard. Only terminal outcome
construction disarms it. Explicit abandonment, an early Rust return, or a
panic in a caller-provided ECRECOVER backend therefore consumes all remaining
gas in the dedicated child meter. Integrators should normally call the typed
`authorize_and_execute_*` methods, which do not expose the armed intermediate
state.

`EvmPrecompileOutcome::requires_rollback` is the explicit handoff to CALL
integration. This release does not claim complete nested CALL execution; the
outcome contract prevents that later integration from guessing whether a
precompile error is a Rust control-plane error or an EVM call failure.

## Verification

- compile-fail exact-borrow, non-duplication, and must-use outcome tests;
- forged descriptor, wrong marker, output preflight, atomic out-of-gas, and
  post-payment failure tests;
- explicit-drop, partially used child-meter, and panicking ECRECOVER backend
  lifecycle tests;
- complete native precompile unit and conformance suites;
- fuzz targets migrated to quote, authorize, and outcome handling;
- release-blocking identity, SHA-256, RIPEMD-160, dense BN254 multiplication,
  non-infinity BN254 pairing, maximal bounded ModExp, and high-round BLAKE2F
  work-per-gas ceilings at `precompile_contract_benchmark`;
- default `no_std`, MSRV, strict clippy, package, SBOM, dependency-policy, and
  release-gate checks.
