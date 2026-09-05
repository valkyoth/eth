# BLS12-381 Base Field

Milestone: `v0.56.0`, internal source release; crates.io publication at
`v0.60.0`. Pentest is pending. This is not G1, G2 or signature execution.

## Scope Manifest

Owner: `eth-valkyoth-evm-core`, exposed through `eth::evm_core` with the
optional `evm-core` facade feature. The existing canonical `EvmBls12381Fp`
wire representation and error behavior remain unchanged. New public methods:

| API | Input/output and failure contract | Verification |
| --- | --- | --- |
| `from_u64` | Every u64 becomes its exact field value. | Max-u64 and canonical round trips. |
| `from_wide_be_bytes` | Exact `[u8; 96]` reduced modulo p; no decoding validity claim. | Independent residues, all 768 single-bit inputs and carry-heavy patterns. |
| `add_mod`, `sub_mod`, `mul_mod`, `square`, `negate` | Canonical public operands and canonical modular result. No mutation or allocation. | BigUint oracle for limb-boundary Cartesian products and deterministic full-width samples. |
| `invert` | `None` exactly for zero; otherwise multiplicative inverse. | Independent modular exponentiation and product postcondition. |
| `sqrt` | Smaller canonical root; zero maps to zero, nonsquares to `None`. | Independent exponentiation, squaring check, root ordering and nonsquare rejection. |

The wire decoder must still reject p, larger values, nonzero padding and wrong
lengths. Reduction is an explicitly different arithmetic operation, never a
fallback that accepts invalid wire encodings. No mutable caller output buffer
is used, so failure cannot partially overwrite a result.

The kernel and public adapter are separate files, below 500 lines, in the
existing EVM-core ownership boundary. There are no new runtime dependencies,
unsafe blocks, allocations, recursion, I/O, locks or ambient state. Supporting
crates retain their published versions until the public checkpoint.

## Sources And Arithmetic

Reviewed 2026-09-05 against [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537),
including p, canonical 64-byte field encodings and p congruent to 3 modulo 4.
The retained implementation source is `ethereum/EIPs` revision
`582684e2d7d372c09f45777be8ea603e485e9e9d` in `spec-lock.toml`.
The current upstream monitor reports later repository revisions; the September
[source review](maintenance-review-2026-09-05.md) remains the admission record
for unrelated changes. No fork activation or source pin changes in this slice.

The wire module owns the single production modulus byte constant. Internal
limbs are little-endian Montgomery residues with R = 2^384. Multiplication
forms a full product and performs six radix-2^64 reduction steps using
`-p^-1 mod 2^64`; conversion uses R^2 mod p. Differential tests specify p
independently and exercise conversion plus all arithmetic.

Operands remain below p. Addition is below 2p < 2^384. Every multiply/add/carry
total is at most 2^128 - 1; Montgomery reduction is below 2p, so one final
subtraction suffices and the thirteenth scratch word is zero at completion.
Inversion uses p-2; square root uses (p+1)/4 followed by verification and
selection of the smaller root.

## Timing And Resources

Public data only. Branches, equality, comparisons and reductions are not
constant-time. Fixed loop bounds are a resource property, NOT evidence of
secret-independent timing. Do not route keys, nonces or other secrets through
this type. Secret BLS signing is separately assigned to `v0.309.0` and its
following audit/integration work; G1 starts at `v0.57.0`, charged G1 at
`v0.58.0`, and Fp2 starts at `v0.59.0`.

- Public field value: 48 bytes. Internal residue: six u64 limbs (48 bytes).
- Multiplication scratch: 13 u64 words (104 bytes), reused per operation;
  this excludes bounded operand/result temporaries and compiler stack spills.
- One Montgomery product has 36 product and 36 reduction multiply-adds,
  plus at most 42 carry-propagation steps, independent of operand lengths.
- Exponentiation scans 384 fixed bits, at most 384 squares and 384 multiplies;
  zero inversion returns early. No caller-supplied exponent or unbounded loop.
- Wide reduction scans 768 bits, with 768 doublings and at most 768 additions.
- No heap use; source-level temporaries are fixed-size, not a promise of a
  particular compiler-generated stack frame on every platform.

Hosts must budget these standalone arithmetic calls. They have no gas token
and do not authorize native precompile dispatch. The BLS precompiles still
fail closed; their later integration gates must assess aggregate gas/work.

## Example

Source-checkout example (not available in published `eth 0.55.0`):

```rust
use eth::evm_core::EvmBls12381Fp;

let two = EvmBls12381Fp::from_u64(2);
let four = two.square();
assert_eq!(four.sqrt(), Some(two));
assert_eq!(two.sub_mod(two), EvmBls12381Fp::from_u64(0));
```

## Verification Commands

```sh
cargo test -p eth-valkyoth-evm-core --test bls12_field_differential
cargo test -p eth-valkyoth-evm-core --all-features
cargo run -p eth-valkyoth-evm-core --release --example bls12_field_benchmark
cargo +1.90.0 check --workspace --all-features
scripts/materialize_fuzz_seeds.py
cargo +nightly fuzz run bls12381_field -- -max_total_time=30 -max_len=96
scripts/checks.sh
scripts/release_0_56_0_gate.sh
```

`num-bigint` is a test/fuzz-only independent arithmetic oracle. Fuzzing checks
modular results and inverse/root answers against that oracle as well as
algebraic invariants. It never enters the runtime dependency graph.
The benchmark uses fixed iteration counts and black-box barriers; it measures
public API cost including conversions, not constant-time behavior. Actual
reports go in `target/` and must accompany the implementation review.
The final release gate deliberately requires a completed exact-commit pentest
report; ordinary checks can pass before that report exists.

## Implementation Evidence

Local review on 2026-09-05, Rust 1.98.1, x86_64 Linux:

- All four independent field differential tests passed, as did the EVM-core
  all-feature tests, full `scripts/checks.sh`, and fuzz-workspace Clippy with
  warnings denied. This includes package verification and workspace doctests.
- Workspace all-feature compilation passed for every supported compiler from
  1.90.0 through 1.98.1 in the documented compatibility matrix.
- Direct dependency/tool/Action freshness checks, Cargo Deny and Cargo Audit
  passed. RustSec required ordinary access to its external advisory cache.
- Initial ASan/libFuzzer smoke: 19,237 runs in 31 seconds, no finding, with
  independent modular arithmetic and inverse/root checks.
- A second smoke with an explicit 96-byte carry-heavy seed completed 19,962
  runs in 31 seconds without a finding; full repository checks passed again.
- Fixed-work API benchmark: add 102, sub 100, multiply 128, square 99,
  wide reduction 3,341, inversion 22,485 and square root 22,124 ns/op.
  These are one local measurement, not portable performance guarantees.
- Existing in-process RLP and ModExp differential regressions passed.
- The inherited Geth/Besu/Nethermind regression is blocked by the host's
  missing CPU/memory cgroup delegation, both inside and outside the sandbox.
  The gate is retained unchanged and must run on a capable host before tag
  admission. No client-run success or full release-gate success is claimed.

Pentest, remediation/retest where needed, the permanent security report,
GitHub CI and CodeQL remain required before tag authorization.
