# BLS12-381 Quadratic Extension Field

Source milestone: v0.59.0, signed internal tag after clean pentest and GitHub. The next public
checkpoint is v0.60.0. This document does not claim G2 or pairing execution.

## Contract

`EvmBls12381Fp2` represents `c0 + c1*v` in `Fp[v]/(v^2 + 1)`.
The modulus and nonresidue come from
[EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#curve-parameters), checked
2026-09-10. The pinned source revision is recorded in `spec-lock.toml`.
The nonresidue is `p-1`, not a coefficient-order or curve-twist convention.

The existing exact 128-byte encoding remains `encode(c0) || encode(c1)`;
each coefficient is a canonical 64-byte big-endian Fp value with zero padding.
Wire decoding rejects out-of-range coefficients; it never silently reduces.
`from_coefficients` accepts already-canonical Fp types. `from_u64` embeds in c0.

New APIs are addition, subtraction, multiplication, square, negation,
conjugation, inversion and square root. Values own 96 fixed bytes; operations
consume/return copies, never borrow caller scratch or allocate. The private
kernel reuses six-limb Montgomery Fp arithmetic. Core has no normal dependency,
no unsafe code, no recursion, no platform service, and no new feature flag.
The facade exposes this through the existing opt-in `evm-core` feature.

All inputs must be public. Branching/reductions are variable-time, values are
Copy/Debug, and no secret-clearing guarantee is made. Do not use this for secret
scalar arithmetic or private-key handling. Arithmetic is fork-independent;
it confers no curve/subgroup validation or precompile execution authority.

## Inverse And Root Derivation

For `z=a+b*v`, conjugation is `a-b*v` and the norm is `a^2+b^2`.
Since -1 is a nonsquare in Fp, the norm vanishes only for zero.
The inverse is the conjugate times the base-field inverse of the norm.
`invert()` returns `None` exactly at zero.

For `b != 0`, a root `x+y*v` satisfies
`x^2=(a +/- sqrt(a^2+b^2))/2` and `y=b/(2*x)`.
Try both norm signs; a valid x cannot vanish for nonzero b. A nonsquare norm
rejects immediately. For `b=0`, return a real root of a if one exists,
otherwise an imaginary root of -a. This separately handles zero and avoids
division by zero. Every candidate is checked by squaring in Fp2.

The chosen root has lexicographically smaller canonical `c0 || c1` bytes
than its negative. This is an API convention, **not** a hash-to-curve sign bit,
point compression standard, or subgroup statement. Nonsquares return `None`;
zero returns zero. Worst-case work is three Fp roots plus two Fp inversions,
each with a fixed 384-bit exponent loop, and fixed extra field operations.
Hostile callers still need an outer request/work budget; this API is unmetered
arithmetic, not a gas-priced precompile.

## Example And Verification

```rust
use eth_valkyoth_evm_core::{EvmBls12381Fp as Fp, EvmBls12381Fp2 as Fp2};
let z = Fp2::from_coefficients(Fp::from_u64(3), Fp::from_u64(4));
assert_eq!(z.square().sqrt(), Some(z));
assert_eq!(z.mul_mod(z.invert().unwrap()), Fp2::from_u64(1));
```

The public API includes a compiled doctest. Reproduce the local checks:

```sh
cargo test -p eth-valkyoth-evm-core --test bls12_fp2_differential
cargo test -p eth-valkyoth-evm-core --release --test bls12_fp2_differential
cargo run -p eth-valkyoth-evm-core --release --example bls12_fp2_benchmark
scripts/materialize_fuzz_seeds.py
cargo +nightly fuzz run bls12381_fp2 -- -max_total_time=30 -max_len=193
scripts/release_0_59_0_gate.sh --implementation
```

Retained tests include the EIP-2537 H2 generator twist equation (field-only,
not G2 execution), exact small polynomial vectors, 144 small pairs,
full-width limb boundaries, 128 deterministic residue pairs, operation chains,
canonical encoding rejection, inverse-by-`p^2-2` and Frobenius-by-p checks.
The dev-only BigUint polynomial oracle uses schoolbook multiplication rather
than Montgomery/Karatsuba formulas; Euler's criterion on the norm checks root
existence independently of root extraction. Returned roots are independently
squared and their sign checked. The fuzz target exercises both malformed wire
bytes and reduced full-width coefficients with the same independent oracle.
The fixed-work host smoke measures 1,000 inverse/root pairs, not portable gas
calibration, constant-time evidence, or an external client run.

G2 point formulas are implemented in the [v0.60.0 candidate](bls12-g2-arithmetic.md), paid G2ADD is v0.61.0, subgroup validation is
v0.62.0, and later MSM/map/pairing admission runs through v0.70.0. No registry
entry changes here; existing paid G1ADD regression tests remain mandatory.

## Implementation Evidence (2026-09-10)

- `scripts/release_0_59_0_gate.sh --implementation`: passed on Rust 1.98.1,
  including workspace tests/doctests, archive verification, strict Clippy,
  all five Fp2 integration tests in debug/release, existing Fp/G1/vector tests,
  five in-process differential paths, 24 gate regressions and Cargo Deny.
- All 13 direct registry dependencies and checked Cargo/GitHub tools were
  current. Ethereum upstream monitoring ran without changing admitted forks.
- Both lockfiles passed Cargo Audit against 1,243 advisories. The first run
  stopped at the sandbox's read-only advisory cache; the complete gate was
  rerun with permission to refresh that cache, without skipping any check.
- Fp2 fuzz smoke: 5,128 executions in 31 seconds; charged G1 regression smoke:
  33,566 executions in 31 seconds, no failure. Retained seeds include exact
  canonical wire bytes and two full coefficient pairs.
- Release host smoke: 126,628 ns per inverse/root pair, 1,000 pairs; existing
  charged G1 smoke: 22,686 ns per call. These are host observations only.
- All 14 installed older compilers from 1.90.0 through 1.98.0 passed workspace
  all-feature checks. Rust 1.98.1 also passed workspace no-default checks.
- `eth --no-default-features --features evm-core` cross-checks passed for
  `thumbv7em-none-eabi`, `riscv32imac-unknown-none-elf`,
  `wasm32-unknown-unknown`, `x86_64-pc-windows-msvc`,
  `x86_64-unknown-freebsd`, `aarch64-apple-darwin`, `aarch64-apple-ios`,
  `aarch64-linux-android`, and `powerpc64-unknown-linux-gnu`. These are compile
  checks, not runtime tests on those operating systems.
- Core normal dependency tree remains empty. The packaged facade README is
  identical to the GitHub README and contains no bundled bitmap.

## External Review And Integration Obligations

The maintainer supplied clean incremental SAST for v0.58.0 through
`39228979b26a060b85a480d7124ad1de3a2f8445`: no Critical, High or Medium
findings and no remediation or retest required. The review reports workspace,
package, doctest, Clippy, debug/release differential and SBOM checks passing,
plus 2,659 additional Fp2 fuzz executions. The reviewer disabled LeakSanitizer
leak detection because ptrace was unavailable; this is not leak-check evidence.
Fresh local finalization results are recorded separately in the permanent
`security/pentest/v0.59.0.md` report.

Before remotely reachable G2/precompile integration, v0.61.0 must enforce
non-forgeable gas/work authorization before any inversion or square-root
work and collect cross-client G2 vectors. v0.60.0 remains standalone public
point arithmetic, not remote execution admission. Cross-client G2 execution,
exhaustive fuzzing and formal proof are not claimed for this field-only slice.
GitHub passed and the maintainer authorized the signed v0.59.0 tag.
