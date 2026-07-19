# Release Notes - eth v0.52.2

Status: implementation complete; awaiting pentest.

## Summary

This release corrects a consensus divergence in the native EVM bootstrap:
truncated `PUSH1..=PUSH32` instructions now read missing immediate bytes as
trailing zeros instead of failing. Execution and jump-destination analysis use
one instruction-advance rule and both consume the full declared PUSH width.

The release does not broaden the admitted opcode set or claim complete EVM
execution. It closes the specific PUSH-at-EOF milestone before later execution
work builds on the bytecode scanner.

## Changed

- Replaced truncated-immediate rejection with allocation-free right-zero
  padding for every PUSH width.
- Shared checked instruction advancement between the interpreter and
  jump-destination map construction.
- Retained `EvmCoreError::PushImmediateOutOfBounds` as a hidden legacy
  compatibility variant while ensuring no production execution path returns
  it; truncated PUSH is valid Ethereum bytecode, not an execution error.
- Minor-bumped `eth-valkyoth-evm-core` from `0.26.0` to `0.27.0` for the public
  error-domain and consensus-behavior change.
- Bumped the `eth` facade from `0.52.1` to `0.52.2` and updated its optional
  `evm-core` dependency.

## Security Notes

- Execution and static jump analysis cannot disagree about a PUSH instruction's
  declared boundary because both paths use the same helper.
- Program-counter arithmetic remains checked. Missing code bytes are modeled
  only as EVM-defined zero bytes and never accessed out of bounds.
- No dependency, allocation, unsafe code, host call, or additional opcode is
  introduced.
- A present `JUMPDEST` byte inside a truncated PUSH immediate remains data and
  cannot be selected as a dynamic jump target.
- The facade patch release preserves the legacy public error variant and its
  stable category code, avoiding an automatic Cargo update breaking downstream
  matches or references.

## Verification

- Exhaustive execution tests cover all 528 combinations of PUSH width and
  truncated available length.
- Boundary tests check the padded word, gas, step count, and declared
  program-counter advance.
- Jump-destination tests cover every truncated immediate length and attempted
  jumps into present immediate bytes.
- Conformance was compared with the Yellow Paper, pinned Ethereum
  `execution-specs`, and Geth's independent PUSH implementation.
- `cargo test -p eth-valkyoth-evm-core --all-features`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin truncated_push -- -D warnings`
- `scripts/materialize_fuzz_seeds.py --check`
- `scripts/release_crates.py --check`
- `scripts/checks.sh`
- Full release gate on pinned Rust `1.97.1`; compatibility checks remain
  assigned to Rust `1.90.0` through `1.97.0`.

## Pentest

Pending. The release cannot become tag-ready until the implementation commit
has an independent pentest, all findings are resolved, the retest passes, and
the permanent report is stored as `security/pentest/v0.52.2.md`.
