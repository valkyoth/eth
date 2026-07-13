# Release Notes - eth v0.52.0

Status: implementation complete; awaiting pentest.

## Summary

This release replaces placeholder advanced-precompile planning with exact
EIP-4844 and EIP-2537 contracts. KZG and BLS12-381 arithmetic remains fail
closed, but callers can now plan canonical input shapes, fixed output lengths,
and every official gas schedule before a backend is admitted.

The release also commits to first-party dependency-free BLS12-381 and KZG
implementation sequences. External implementations may serve as independent
test oracles or optional accelerators, but not as the sole native consensus
path.

## Added

- `NonEmptyMultipleOf` precompile input policy for EIP-2537 MSM and pairing.
- EIP-2537 G1/G2 MSM and pairing gas policy domains.
- Official 128-entry G1 and G2 MSM discount tables with capped discounts.
- Exact KZG/BLS frame and fixed output metadata for addresses `0x0a..=0x11`.
- Boundary tests for fixed frames, empty/partial variable frames, output sizes,
  fixed gas, MSM discount boundaries, and pairing tuple gas.
- An independent EIP-2537 fixture oracle covering every G1/G2 MSM discount,
  every corresponding gas result, and the 129-item capped-discount boundary.
- Explicit bytecode regressions for Yellow Paper stack-top operand ordering on
  `SUB`, `LT`, and `GT`.
- Semantic SPDX drift comparison with tested `--write` and `--check` modes.
- `advanced_precompile_plan` fuzz target for all KZG/BLS planning paths.
- Advanced-precompile backend admission and conformance policy.
- Consensus regressions for false `JUMPI` destinations and zero-length
  `RETURN`, `REVERT`, and call/create memory ranges.
- `EXTCODECOPY` regressions for zero-length full-width offsets, full-width code
  offsets, and partial copies beyond the code boundary.
- Same-length BLAKE2F and ModExp plan-substitution regressions.
- A bounded 32-layer shared EIP-712 dependency-DAG regression and a hard
  64-type schema ceiling.
- Redacted `Debug` implementations for borrowed EIP-712 signing values.
- Concrete first-party BLS releases `v0.52.1..=v0.52.9` and KZG releases
  `v0.61.0..=v0.61.5`.

## Changed

- `eth-valkyoth-evm-core` is bumped from `0.24.0` to `0.25.0`.
- `eth-valkyoth-verify` is patch-bumped to `0.21.1` for `k256 0.14.0`.
- `eth-valkyoth-derive` is patch-bumped to `0.17.3` for `trybuild 1.0.118`.
- `eth-valkyoth-sanitization` is patch-bumped to `0.7.5` for `sanitization
  1.2.4` and the derive dependency update.
- `eth` is bumped from `0.51.0` to `0.52.0`.
- Prague BLS descriptors no longer use `BoundedAny`, unknown output lengths,
  or deferred dynamic gas.
- Advanced-precompile gas arithmetic is checked and split into focused modules
  so every source remains below 500 lines.
- Optional `k256` recovery uses the stable `0.14.0` API while retaining the
  project-owned scalar and low-s validation boundary.
- All public precompile execution is now available only through charged
  `EvmPrecompilePlan` methods; raw unmetered executors are crate-private.
- Every precompile plan recomputes gas from the actual execution input and
  rejects a changed content-dependent cost before charging or execution.
- Identity, SHA-256, RIPEMD-160, ECRECOVER, and BLAKE2F plan execution accepts
  the gas meter before input and output arguments, matching the other charged
  plan APIs.
- `SUB`, `LT`, and `GT` now apply stack-top `mu_s[0]` as the left operand.
- False `JUMPI` no longer converts or validates its unused destination, and
  zero-length memory ranges canonicalize their semantically irrelevant offset
  to zero without host-width conversion or memory expansion.
- `EXTCODECOPY` decodes length first, ignores offsets for empty copies, and
  zero-fills code offsets outside the release code-size domain without calling
  the state host.
- EIP-712 dependency discovery now visits each reachable type once before
  bounded lexical emission instead of recursively rediscovering shared DAGs.
- `Eip712Value` and `Eip712ValueKind` no longer implement `Copy`, `Clone`, or
  revealing derived `Debug`.
- CI and release readiness reject a committed SBOM that differs from a freshly
  generated dependency inventory.

## Security Notes

- No crypto, allocator, bigint, KZG, BLS, or trusted-setup dependency is added.
- All unimplemented KZG/BLS execution still reports backend unavailable.
- Empty BLS MSM and pairing frames are rejected as required by EIP-2537.
- Gas planning uses checked count, multiplication, and addition operations.
- The 1 MiB global precompile input limit remains in force.
- First-party execution cannot be admitted until official vectors,
  differential evidence, fuzzing, dependency review, and pentest pass.
- Every transcribed EIP-2537 discount entry is checked against a separately
  stored official-spec fixture before any later release makes the charge live.
- Precompile gas is charged before output mutation, hashing, recovery, or
  arithmetic; out-of-gas failures leave outputs and backends untouched.
- Content-dependent BLAKE2F and ModExp costs are rebound to the exact execution
  bytes, preventing a cheap plan from authorizing more expensive same-length
  work.
- Non-commutative opcode tests lock the consensus-sensitive operand order.
- Conditional jump and zero-length memory tests lock Yellow Paper operand-use
  semantics for host-width edge cases.
- EIP-712 signing values redact their payloads from formatting, and both the
  borrowed and JSON paths inherit the fixed type-count ceiling.
- The v0.52.0 pentest's High operand-order, Medium unmetered-execution, and
  Medium stale-SBOM findings were remediated before retest.
- The follow-up v0.52.0 review's four High consensus/gas/complexity findings
  and one Medium signing-value disclosure finding were remediated for retest.
- The subsequent retest's remaining High `EXTCODECOPY` host-width finding was
  remediated across empty and out-of-code copy semantics.
- Public precompile values are not treated as secret material; any future reuse
  for secret-bearing key operations requires a separate sanitization contract.

## Verification

- `cargo test -p eth-valkyoth-evm-core precompile`
- `cargo test -p eth-valkyoth-evm-core`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin advanced_precompile_plan -- -D warnings`
- `cargo fmt --all --check`
- `scripts/validate-release-metadata.sh`
- `scripts/generate-sbom.sh --check`
- `python3 scripts/test-release-metadata.py`
- `scripts/release_crates.py --check`
- `scripts/checks.sh`

## Pentest

The release must not be tagged until its independent pentest, remediation,
retest, and committed `security/pentest/v0.52.0.md` report are complete.
