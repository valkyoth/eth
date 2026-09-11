# BLS12-381 G2 Group Operations

Source milestone: v0.60.0 public-checkpoint candidate. Cumulative pentest
against v0.55.0 is pending. This slice implements standalone public-point
arithmetic; it does not authorize G2 precompile execution.

## Contract

`EvmBls12381G2Affine` owns either infinity or canonical Fp2 coordinates
satisfying `y^2=x^3+4*(1+v)`, with `v^2=-1`.
`try_from_coordinates` rejects finite (0,0) and off-curve points;
`try_from_wire` adds curve validation to the existing wire-only domain.
`try_from_be_bytes` requires exactly 256 canonical EIP-2537 bytes:
`x.c0 || x.c1 || y.c0 || y.c1`, each a zero-padded 64-byte Fp value.
The unique all-zero frame denotes infinity. Out-of-range coordinates are
rejected, never reduced. Swapping coefficient order is not an alternate format.

`EvmBls12381G2Projective` has private Jacobian coordinates with
`x=X/Z^2`, `y=Y/Z^3`; Z=0 denotes infinity. Only validated affine points
and closed group operations construct it. Conversion, negation, complete
addition/doubling and geometric equality cover infinity, equal and inverse
points, including non-unit/non-real projective rescalings. Finite normalization
uses one Fp2 inversion. Addition/doubling and geometric equality use none.

The formulas are the a=0 EFD `add-2007-bl` and `dbl-2009-l`, with exceptional
cases handled before the generic formulas. They apply over Fp2 as well as Fp.
Private coordinate storage is 288 bytes; scratch is fixed-size. There is no
heap, recursion, unsafe code, platform service, new runtime dependency or
attacker-sized loop. The default core normal dependency tree remains empty.
The facade exposes both point types through the existing `evm-core` feature.

**Public inputs only.** Values are Copy and operations are variable-time.
No secret clearing, constant-time scalar operation, signing, subgroup
validation, pairing, gas authorization or fork validity is implied.
Curve points outside the prime subgroup are deliberately accepted, as required
for EIP-2537 addition. Direct callers must bound repeated requests themselves.
G2ADD admission and work-before-arithmetic controls are v0.61.0; subgroup
checks are v0.62.0, followed by MSM/maps/pairing through v0.70.0.

## Sources And Fixtures

Reviewed 2026-09-11:

- [EIP-2537 curve parameters and G2 addition](https://eips.ethereum.org/EIPS/eip-2537).
- [Explicit-Formulas Database, a=0 Jacobian formulas](https://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html).

Fixture source: Ethereum EIPs revision
`582684e2d7d372c09f45777be8ea603e485e9e9d`, also in `spec-lock.toml`.

| Input | SHA-256 | Cases |
| --- | --- | --- |
| `assets/eip-2537/add_G2_bls.json` | `c4e5efd1d487ccb11aa171e44b40d20228cd6e251a83d967d30cdd0f45c06bf7` | 9 positive |
| `assets/eip-2537/fail-add_G2_bls.json` | `e7ee1a0d2e67febb0da69310bf0cdd3f3b07cbe55fcf5045852e1025d98ac976` | 7 rejection |

The importer checks each complete source hash before parsing JSON and emits
the committed line fixtures. The test-only 512-byte addition adapter compares
canonical output with upstream results; it is not a dispatcher implementation.

```sh
python3 scripts/import_bls_g2_vectors.py /home/eldryoth/Work/test/eth/eips/assets/eip-2537/add_G2_bls.json
python3 scripts/import_bls_g2_vectors.py /home/eldryoth/Work/test/eth/eips/assets/eip-2537/fail-add_G2_bls.json --failures
cargo test -p eth-valkyoth-evm-core --test bls12_g2_vectors --test bls12_g2_differential
cargo test -p eth-valkyoth-evm-core --release --test bls12_g2_vectors --test bls12_g2_differential
cargo run -p eth-valkyoth-evm-core --release --example bls12_g2_benchmark
scripts/materialize_fuzz_seeds.py
cargo +nightly fuzz run bls12381_g2 -- -max_total_time=30 -max_len=257
scripts/release_0_60_0_gate.sh --implementation
```

The independent dev-only BigUint oracle uses polynomial Fp2 arithmetic and
affine slopes, not the production Montgomery/Jacobian formulas. Tests include
generator chains, independently generated full-curve points, geometric
equality, inverse/doubling/associativity, malformed lengths, every coefficient's
padding/modulus boundary, and coefficient-order rejection. Fuzzing compares
wire decisions and group results against that oracle, not only self-identities.

The README includes a compiled finite-point example. The fixed-work benchmark
times 1,000 additions and enforces a five-second host ceiling; this is neither
portable gas calibration nor side-channel evidence. Fresh Geth/Besu/Nethermind
G2 execution is explicitly assigned to v0.61.0; the official offline vectors
and independent oracle are not represented as a fresh cross-client run.

## Publication Scope

v0.60.0 bundles v0.56.0 Fp, v0.57.0 G1, v0.58.0 charged G1ADD,
v0.59.0 Fp2 and this G2 slice. All README changes and cumulative dependency
updates are included in the same publication review. Package versions and
dependency order are recorded in [the version matrix](CRATE_VERSION_MATRIX.md).
No final PASS report, tag or publication is authorized by implementation tests.

## Implementation Evidence (2026-09-11)

- `scripts/release_0_60_0_gate.sh --implementation` passed on Rust 1.98.1:
  760 workspace tests/doctests passed, 2 intentionally ignored benchmark tests,
  strict workspace/fuzz Clippy, all 14 package archives, 29 release-gate
  regressions, six independent in-process reference suites, freshness checks,
  Cargo Deny and both Cargo Audit scans.
- All 42 README examples passed with no ignored examples across all 14
  packages. The checker fetches the locked workspace graph before its offline
  example builds so a cold CI cache is supported. Six README-policy and three
  archive/dependency-closure regression tests passed. After this CI portability
  correction, `scripts/checks.sh` was rerun successfully.
- Final targeted fuzz smokes: G2 9,990 executions, charged G1ADD 32,243,
  Fp2 5,139, each in 31 seconds without a failure. This is smoke coverage,
  not an exhaustive campaign.
- Fixed-work observations: G2 affine addition 23,543 ns/call, Fp2 inverse/root
  127,202 ns/pair, charged G1ADD 21,689 ns/call. Each benchmark used 1,000
  operations; these are host measurements, not portable gas/side-channel proof.
- Both complete official fixture source hashes reproduced the committed
  positive/rejection files using the importer with `--check`.
- All 14 older-Rust workspace/all-feature checks from 1.90.0 through 1.98.0
  passed. Rust 1.98.1 also passed workspace/no-default checks.
- `eth --no-default-features --features evm-core` compiled for
  `thumbv7em-none-eabi`, `riscv32imac-unknown-none-elf`,
  `wasm32-unknown-unknown`, `x86_64-pc-windows-msvc`,
  `x86_64-unknown-freebsd`, `aarch64-apple-darwin`, `aarch64-apple-ios`,
  `aarch64-linux-android` and `powerpc64-unknown-linux-gnu`.
  These are compile checks, not runtime testing on those systems.
- All 13 direct registry dependencies and checked Cargo/GitHub tooling were
  current. RustSec scanned the root/fuzz lockfiles (81/69 dependencies) against
  1,243 advisories without findings. The first full run stopped at the
  sandbox's read-only advisory cache; the complete gate passed on rerun with
  permission to refresh that cache.
- The SBOM matches the graph. The default core normal dependency tree is empty.
  Every package archive contains its current README and no logo bitmap.

The fresh ModExp client runner stopped before client startup because this host
does not delegate all required Podman `cpu`, `memory` and `pids` controllers.
No reduced-isolation fallback was used. Historical v0.55.0 client results are
not claimed as a fresh v0.60.0 run. This infrastructure limitation is recorded
under the [integration-evidence workflow](RELEASE_RUNBOOK.md#integration-evidence);
G2 cross-client execution is still a mandatory v0.61.0 deliverable.

Implementation stop reached. Request cumulative pentest of the exact committed
candidate against v0.55.0, including all intermediate milestones, package
requirements, README examples and publication tooling. No external v0.60.0
pentest result is asserted by this document.
