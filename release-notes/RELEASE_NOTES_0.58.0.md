# eth v0.58.0

Status: implementation checks and pentest passed; GitHub/tag approval pending.
Publication: DEFERRED TO v0.60.0

## Scope

- Adds sealed `EvmBls12G1Add` paid execution and `NativeBls12G1Add` registry
  admission at Prague address `0x0b`, reusing the tagged Fp/G1 implementations.
- Exactly 256 input bytes, canonical 128-byte output and 375 gas. Infinity
  and on-curve non-prime-subgroup points are valid, as required by EIP-2537.
- Output capacity and gas admission precede point work; both curves are
  validated before addition and the staged result is committed atomically.
- Malformed paid input returns CALL failure with supplied gas consumed and
  unchanged output. Earlier admission errors remain caller-handled errors,
  not successful CALL outcomes; see the [integration contract](../docs/bls12-g1-addition.md).

## Verification And Boundaries

All nine official positive and seven official negative vectors exercise the
typed API. Thread-local tests verify no point work before payment and no
addition on malformed points. New differential execution fuzzing compares
against the independent BigUint oracle; fixed-gas CPU smoke includes the full
quote/charge/validate/add/encode path. Existing group/field tests remain active.

No runtime dependency, allocation, unsafe code or default feature is added.
No subgroup validation, private-key arithmetic, G2/MSM/pairing/map execution,
full CALL interpreter or external-client conformance is implied.

This source milestone selects no crates for publication. The main crate
version is 0.58.0; support crate bumps accumulate for v0.60.0. README dependency
examples retain published `eth 0.55.0`. The external pentest/SAST of
`e5b4afaeabbcb17f4a61ac63c148edaa66beda48` is clean, with no Critical, High or
Medium findings and no remediation required. Next step: GitHub CI/CodeQL wait,
then explicit maintainer tag authorization.

The Rust 1.98.1 implementation gate, 14 older-Rust compatibility checks and
nine no-default-feature cross-target builds passed. Pre-pentest charged/standalone G1 fuzz
smokes completed 31,791/15,358 inputs without a finding. See the integration
document for commands, scope limitations and fixed-work CPU measurements.
