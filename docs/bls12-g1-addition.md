# Charged BLS12-381 G1 Addition

Milestone: v0.58.0, internal source candidate; pentest clean, GitHub/tag approval
pending. Crates.io
publication remains v0.60.0. Owner: `eth-valkyoth-evm-core`, optional
`eth::evm_core`. No new runtime dependency, allocation or unsafe code.

## Contract

This integration reuses the tagged v0.56.0 Fp and v0.57.0 on-curve G1
implementations, not a new curve backend. The registry admits
`EvmPrecompileKind::Bls12G1Add` at address `0x0b` from Prague. The sealed
`EvmBls12G1Add` marker selects `NativeBls12G1Add`; other BLS/KZG execution
remains unavailable. On-curve membership is deliberately not prime-subgroup
membership. No cofactor clearing, scalar multiplication or subgroup rejection
is performed for this addition operation.

Reviewed against [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537) on
2026-09-10: input is exactly 256 bytes (two canonical G1 points), output is
128 bytes, and the fixed charge is 375 gas. Encoding and group arithmetic
retain the [G1 invariant and resource contract](bls12-g1-arithmetic.md).

`descriptor.quote::<EvmBls12G1Add>(input)` validates descriptor identity,
fork, kind and frame length without parsing points. The immutable borrow
binds the quoted bytes. `authorize_and_execute_bls12_g1_add` checks output
capacity and charges the meter before decoding either point. The paid
capability remains private, single-use and guarded against uncompleted work.

Both canonical wire encodings are parsed before curve validation. Both curves
are validated before group addition; the canonical result is staged in a
fixed 128-byte array and copied only after all fallible work succeeds. Success
preserves the output suffix and consumes exactly 375 gas. No caller-sized
allocation, recursion or secret-input constant-time claim is introduced.

## Failure Handling

| Failure | Return and effects |
| --- | --- |
| Wrong fork, forged descriptor, wrong marker or wrong input length | Admission `Err`; no point validation, addition, output mutation or meter charge. |
| Output capacity below 128 | Admission `Err(PrecompileOutputTooSmall)` before charge or point work. |
| Remaining gas below 375 | Admission `Err(OutOfGas)` before point work; meter and output unchanged. |
| Noncanonical field or off-curve point after payment | `CallFailure`, output length zero, remaining supplied gas consumed, rollback required; output untouched. |
| Valid points, including infinity and non-prime-subgroup points | `Success`, exactly 128 output bytes and 375 gas consumed. |

This follows the existing typed native-precompile API: admission `Err` is
**not** a successful CALL or a terminal CALL outcome. CALL integration must
translate protocol input/gas errors into failure, consume the appropriate
child gas and perform rollback. Short host output capacity and forged
descriptors are integration errors, not consensus rules permitting rejection
of otherwise valid transactions. This slice does not enable the still-planned
full interpreter CALL machinery. Nested journal/precompile integration is
owned by v0.135.0 and composed into the block transition at v0.137.0.

"No arithmetic on failure" means no group addition/normalization for malformed
points and no curve arithmetic before admission. Curve-membership rejection
itself necessarily uses bounded field arithmetic after payment. Tests count
validation and addition entry separately, with thread-local instrumentation
compiled out of production, so parallel tests cannot share counters.

## Example

Source checkout only; published `eth 0.55.0` does not contain this API:

```rust
use eth::evm_core::{EvmBls12G1Add, EvmFork, EvmGas, EvmGasMeter,
    EvmPrecompileKind, EvmPrecompileRegistry, EvmPrecompileStatus};
let descriptor = EvmPrecompileRegistry::try_new(EvmFork::PRAGUE)?
    .descriptor(EvmPrecompileKind::Bls12G1Add)?;
let input = [0u8; 256]; // infinity + infinity
let mut output = [0u8; 128];
let mut gas = EvmGasMeter::try_new(EvmGas::new(375))?;
let outcome = descriptor.quote::<EvmBls12G1Add>(&input)?
    .authorize_and_execute_bls12_g1_add(&mut gas, &mut output)?;
assert_eq!(outcome.status(), EvmPrecompileStatus::Success);
assert_eq!(outcome.output_len(), 128);
# Ok::<(), eth::error::EvmCoreError>(())
```

## Independent Evidence

All nine positive fixtures from v0.57.0 now execute through the paid API at
exact and surplus gas. Seven negative vectors from the same EIPs pin are
added, preserving even upstream's misnamed `bls_g2add_invalid_field_element`
case in the G1 file. Input lengths, invalid fields, high bytes, off-curve
points, output preservation and failure gas behavior are checked explicitly.

Source: `ethereum/EIPs` revision
`582684e2d7d372c09f45777be8ea603e485e9e9d`, CC0,
`assets/eip-2537/fail-add_G1_bls.json`, SHA-256
`92a85348a2c172d6e12ce858566febdc9f8794f542b2a18826fa3f7170a62bfd`.
The importer authenticates the complete source before JSON decoding and never
executes upstream code. Positive fixture provenance remains in the G1 document.

```sh
python3 scripts/import_bls_g1_vectors.py /home/eldryoth/Work/test/eth/eips/assets/eip-2537/fail-add_G1_bls.json --failures --check
cargo test -p eth-valkyoth-evm-core --all-features
cargo test -p eth-valkyoth-evm-core --release --test bls12_g1_vectors --test bls12_g1_rejections
cargo run -p eth-valkyoth-evm-core --release --example bls12_g1_add_benchmark
scripts/materialize_fuzz_seeds.py
cargo +nightly fuzz run bls12381_g1_add -- -max_total_time=30 -max_len=257
scripts/release_0_58_0_gate.sh --implementation
```

The charged fuzzer compares generated full-width points against the independent
BigUint affine oracle, also exercising arbitrary wire input, malformed frames,
zero/insufficient/exact/surplus gas, output capacity and untouched suffixes.
The existing standalone G1 fuzzer remains in the gate. CPU evidence measures
1,000 complete charged finite-point calls at 375 gas each, with a one-second
host smoke ceiling. This catches gross local regressions, not portable gas
calibration, exhaustive resource proof or a constant-time guarantee.

No Geth/Besu/Nethermind run or formal proof is claimed. Independent arithmetic
and immutable official results provide this slice's differential evidence;
external-client infrastructure remains a separate explicit command. Further
BLS owners remain v0.59.0 (Fp2), v0.60.0-v0.61.0 (G2), v0.62.0 (subgroups),
and subsequent MSM/pairing/map milestones through v0.70.0.

Implementation stop: run pentest for the exact implementation commit. A clean
report, GitHub CI/CodeQL approval and explicit tag authorization remain required.

The maintainer subsequently confirmed a clean pentest and supplied incremental
SAST evidence for v0.57.0 through `e5b4afaeabbcb17f4a61ac63c148edaa66beda48`.
No Critical, High or Medium security finding or remediation was reported, and
no retest was requested. The review confirmed the charged G1ADD contract and
public-input restrictions. Fresh local release verification is recorded
separately from the external review's test, benchmark and fuzz results.

## Local Verification, 2026-09-10

- Full `--implementation` gate passed on Rust 1.98.1: package verification,
  workspace tests/doctests and strict Clippy, release-mode field/G1 differential
  suites, all 16 official G1ADD frames, benchmark, both fuzz targets, Deny,
  both lockfile audits, tool/dependency freshness and compatibility checks.
- All 14 older compilers listed in the gate passed all-feature workspace
  checks, retaining the 1.90.0 floor. No dependency/tool updates were needed.
- Charged fuzzing completed 31,791 executions in 31 seconds; standalone G1
  completed 15,358 in 31 seconds. Neither found a mismatch or crash. These
  short runs are smoke evidence, not exhaustive fuzzing.
- Fixed-work charged addition: 22,880 ns/call, 22.881 ms for 1,000 calls at
  375 gas each on this x86_64 Linux host, below the one-second smoke ceiling.
- Four new admission/atomicity/fork tests and 19 release-phase regression
  tests passed. The first gate caught test-only unchecked indexing; checked
  accessors replaced it, and the full gate was rerun without weakening lints.
- `cargo check -p eth --no-default-features --features evm-core --target TARGET`
  passed for `thumbv7em-none-eabi`, `riscv32imac-unknown-none-elf`,
  `wasm32-unknown-unknown`, `x86_64-pc-windows-msvc`, `x86_64-unknown-freebsd`,
  `aarch64-apple-darwin`, `aarch64-apple-ios`, `aarch64-linux-android` and
  `powerpc64-unknown-linux-gnu`. These are compile checks, not target execution.
- Both fixture importer modes passed `--check`. Both repository READMEs match
  the packaged facade README, with no bitmap asset packaged. The core normal
  dependency tree remains empty; all changed code files remain below 500 lines.
- Ethereum upstream monitoring still reports source drift as advisory metadata,
  not automatic fork/backend admission. No external client run is claimed.
- Default tag admission failed as expected with the missing v0.58.0 pentest
  report. Implementation testing does not fabricate a PASS report.
