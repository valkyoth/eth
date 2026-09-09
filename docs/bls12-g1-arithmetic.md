# BLS12-381 G1 Arithmetic

Milestone: v0.57.0, internal source candidate, pentest clean; GitHub/tag approval
pending. Publication is
at v0.60.0. Owner: `eth-valkyoth-evm-core`, exposed by `eth::evm_core` under
the optional `evm-core` feature. No new runtime dependency or default feature.

## Scope And Boundaries

`EvmBls12381G1Point` remains a canonical wire-only domain. The new
`EvmBls12381G1Affine` validates y^2 = x^3 + 4 or the unique zero-encoded
infinity. `EvmBls12381G1Projective` carries that invariant through private
Jacobian coordinates. Neither type proves prime-subgroup membership.

| Operation | Contract |
| --- | --- |
| Affine `try_from_be_bytes` / `try_from_wire` | Exact 128-byte canonical encoding and curve membership; no reduction, cofactor clearing or subgroup rejection. |
| Affine `try_from_coordinates` | Finite on-curve coordinates only; `(0,0)` is rejected. Use `infinity()` for the identity. |
| `coordinates` / `to_be_bytes` | Finite coordinates or `None`; canonical 128-byte output including all-zero infinity. |
| Affine/projective conversion | No inversion into Jacobian form; one inversion for finite normalization, none for infinity. |
| `negate`, `double`, `add_point` | Closed group operations, including equal/inverse/infinity cases and on-curve points outside the prime subgroup. |
| Projective equality | Geometric equality independent of representation scale; no inversion. |

Standalone public-input arithmetic is not constant-time and has no gas token.
Callers must bound aggregate work. No scalar multiplication, MSM, G2 arithmetic,
pairing, map-to-curve, signature verification or private-key handling is added.
The existing precompile dispatcher is unchanged and BLS execution stays closed.
Charged G1 addition belongs to v0.58.0; Fp2/G2 to v0.59.0-v0.61.0; subgroup
checks to v0.62.0. These types must not be treated as signature-authorized points.

## Sources And Formulas

Reviewed 2026-09-09 against [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537)
and the [Explicit-Formulas Database](https://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html).
Use of the whole curve, rather than only the prime subgroup, is intentional.
For example `(0,2)` has order three and is accepted, not cofactor-cleared.

Coordinates satisfy affine x = X/Z^2, y = Y/Z^3. Z=0 represents infinity.
Generic addition uses EFD `add-2007-bl`; doubling uses the a=0 Jacobian formula
with Z'=2YZ (algebraically equal to `(Y+Z)^2-Y^2-Z^2`). Explicit branches
handle zero Z, equal points, inverse points and zero Y before generic formulas.
Projective fields are private: no unchecked user coordinates can enter the
closed arithmetic operations. Normalization's absent inverse means Z=0 only.

All nine official positive G1-addition vectors are retained as data from
`ethereum/EIPs` revision `582684e2d7d372c09f45777be8ea603e485e9e9d`, matching
the existing EIPs pin in `spec-lock.toml`. Source:
`assets/eip-2537/add_G1_bls.json`, CC0, SHA-256
`224b9b22ddf72efb0cd9c8ffb6cd742d68af7b062e592e8b8a5378a22d6f9d02`.
The deterministic importer preserves every input/output/name; gas fields do
not become evidence of charged execution. The packaged fixture is offline.

```sh
python3 scripts/import_bls_g1_vectors.py /home/eldryoth/Work/test/eth/eips/assets/eip-2537/add_G1_bls.json --check
```

The independent development oracle uses BigUint affine slopes and modular
exponentiation, not production Montgomery/Jacobian formulas. It also validates
raw wire input independently. Fuzzing compares decoding and group results
against that oracle, including doubled points with nonunit projective Z.
The oracle's fixed nonzero modulus and bounded canonical coordinates are its
arithmetic preconditions. No external client execution is claimed for this slice.

## Resources

- Projective value: three six-limb residues, 144 bytes, private representation.
- Affine value: optional pair of 48-byte canonical fields; no heap or recursion.
- Generic addition: 12 field multiplications and five squares; standalone
  doubling uses two field multiplications and five squares. The equal-point
  addition branch first compares scaled coordinates, then doubles, for at most
  eight multiplies and seven squares. Each field operation retains the
  v0.56.0 fixed-width contract.
- Normalization: one bounded field inversion, one square and three multiplies;
  no attacker-sized input, loop or allocation. Equality uses at most six field
  multiplies and two squares. These are source-level operation bounds, not
  guarantees of compiler stack-frame size or secret-independent timing.
- Public methods return owned values; they cannot partially overwrite a caller
  buffer on failure. Canonical/curve parsing fails before producing a capability.

## Example

Source checkout only, not yet in published `eth 0.55.0`:

```rust
use eth::evm_core::{EvmBls12381Fp as Fp, EvmBls12381G1Affine as G1};
let point = G1::try_from_coordinates(Fp::from_u64(0), Fp::from_u64(2))?;
assert_eq!(point.double(), point.negate());
assert!(point.add_point(point.negate()).is_infinity());
# Ok::<(), eth::error::EvmCoreError>(())
```

## Verification And Stop

```sh
cargo test -p eth-valkyoth-evm-core --all-features
cargo test -p eth-valkyoth-evm-core --test bls12_g1_differential --release
cargo test -p eth-valkyoth-evm-core --test bls12_g1_vectors --release
cargo run -p eth-valkyoth-evm-core --release --example bls12_g1_benchmark
scripts/materialize_fuzz_seeds.py
cargo +nightly fuzz run bls12381_g1 -- -max_total_time=30 -max_len=128
scripts/release_0_57_0_gate.sh --implementation
```

The implementation stop requires these tests and pentest of the exact commit.
No permanent PASS report or tag is generated before the maintainer's review.

The maintainer subsequently reported a clean external pentest of
`9dc582e45d78f50af4dd90d11813873fcdded55a` against v0.56.0. No remediation was
required. The review confirmed canonical curve validation, exceptional-case
arithmetic, fixed-size memory-safe operations and unchanged execution
authorization. Its independent Python oracle and additional fuzz evidence are
attributed to the supplied review, not represented as locally reproduced
artifacts. Subgroup validation, private-input timing, cross-client execution,
exhaustive fuzzing and formal verification are not claims of this milestone.

### Local Implementation Evidence, 2026-09-09

- `scripts/release_0_57_0_gate.sh --implementation` passed on Rust 1.98.1:
  workspace tests/Clippy/packaging, metadata/docs/SBOM checks, dependency/tool
  freshness, four in-process differential suites, cargo-deny and RustSec audit.
- All nine official vectors passed in debug and release; three new unit tests,
  three independent G1 differential tests and the runnable API doctest passed.
- The final 30-second G1 fuzz budget completed 15,212 inputs in 31 seconds
  without a mismatch or crash (an earlier run completed 15,364). This is smoke
  evidence, not exhaustive fuzzing or a constant-time audit.
- All 14 compatibility toolchains listed in the gate, from 1.90.0 through
  1.98.0, passed `cargo check --workspace --all-features`.
- `cargo check -p eth --no-default-features --features evm-core --target TARGET`
  passed for `thumbv7em-none-eabi`, `riscv32imac-unknown-none-elf`,
  `wasm32-unknown-unknown`, `x86_64-pc-windows-msvc`, `x86_64-unknown-freebsd`,
  `aarch64-apple-darwin`, `aarch64-apple-ios`, `aarch64-linux-android`, and
  `powerpc64-unknown-linux-gnu`. These are compile checks, not target execution.
- `cargo test -p eth-valkyoth-evm-core --release -- --ignored` also passed both
  existing BN254 timing smoke tests; ordinary suites leave them ignored.
- Fixed-work host samples (1,000 operations each): projective add 548 ns,
  projective double 298 ns, normalization 19,354 ns, affine add 19,990 ns and
  parsing 207 ns. Host measurements do not establish portable gas budgets.
- Fixture reproduction from the pinned source passed. The EVM core's normal
  dependency tree remains empty. Both facade READMEs remain identical.
- The tag gate intentionally failed with `missing pentest report` before
  handoff. No PASS evidence is fabricated from implementation tests.

The first gate attempt stopped at a sandbox-denied RustSec cache lock; the full
gate was rerun with cache permission and passed, rather than skipping audit.
Upstream monitoring reported moved execution/EIP/API/consensus/SSZ heads and
REVM 43.0.2. Those notices are not automatic pin updates or backend admission;
the reviewed G1 contract and immutable fixtures above define this slice.

## Dependency Maintenance

Freshness review found `sanitization 2.1.0` and `trybuild 1.0.121` (both MSRV
compatible). The published sanitization source diff adds `str` support and
routes String wiping through allocation-wide vector provenance; canonical
wipe/exposure APIs used by the bridge are unchanged. Its new secrecy companion
is not admitted. The trybuild source replaces `target-triple` with
`target-tuple`; compile-pass/fail fixtures remain required. These are dependency
source reviews and integration tests, not independent upstream security audits.
