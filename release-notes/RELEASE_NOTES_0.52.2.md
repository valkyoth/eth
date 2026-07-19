# Release Notes - eth v0.52.2

Status: release candidate; pentest remediation and clean retest complete.

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
- Patch-bumped `eth-valkyoth-evm-core` from `0.26.0` to `0.26.1` for the
  API-compatible consensus bug fix, preserving type identity for facade users
  with a direct compatible support-crate dependency.
- Bumped the `eth` facade from `0.52.1` to `0.52.2` and updated its optional
  `evm-core` dependency.
- Expressed the public all-zero EVM word through `u8::MIN` so CodeQL does not
  misclassify this protocol value as embedded cryptographic material; the
  value and public API are unchanged.

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
- A direct invariant test confirms `EvmWord::ZERO` remains the canonical zero
  word after the semantics-neutral CodeQL remediation.

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

The independent review found one Medium facade SemVer issue. Removing the
legacy truncated-PUSH error variant could break downstream code during a
compatible `eth` patch update. The first remediation restored the hidden
variant and stable error code while keeping it unreachable from production
execution.

The first retest found a remaining Medium public-dependency type-identity
issue because the facade patch update moved `eth-valkyoth-evm-core` from the
`0.26` line to `0.27`. The support crate now publishes as the API-compatible
bugfix `0.26.1`, allowing Cargo to unify existing direct `^0.26` dependencies
with the facade's public re-export. Release tooling now enforces exact patch
increments for support-crate `bugfix` entries.

The final retest is clean. The permanent report is
`security/pentest/v0.52.2.md` with `Status: PASS`.
