# eth v0.59.0

Status: implementation checks passed; pentest pending.
Publication: DEFERRED TO v0.60.0

## Scope

- Adds first-party public-input Fp2 arithmetic on the existing canonical
  `EvmBls12381Fp2` type: coefficient constructor, integer embedding, add/subtract,
  multiply/square, negate/conjugate, inverse and deterministic checked roots.
- Freezes `v^2=-1` and `c0 || c1` conventions without changing wire validation.
- Adds independent BigUint differential tests, adversarial encoding cases,
  root-existence checks, structured fuzzing and fixed-work CPU smoke.
- Maps every yellow README capability to concrete completion and assurance
  releases; published dependency examples remain `eth = "0.55.0"`.

## Boundaries And Release State

No new runtime dependency, allocation, unsafe code or default feature.
Arithmetic is variable-time and only for public inputs. G2, subgroup checks,
MSM, maps and pairing are not enabled. Existing G1ADD stays admitted separately.
See [Fp2 contracts and executable verification](../docs/bls12-fp2-arithmetic.md)
and the [partial-capability completion map](../docs/partial-capability-completion.md).

The facade source version is 0.59.0. No crates are selected for publication;
support-crate changes accumulate for the v0.60.0 public checkpoint.

The Rust 1.98.1 implementation gate passed, including workspace/archive tests,
strict Clippy, 24 gate regressions, independent field/group checks, freshness
checks, Cargo Deny and both advisory scans. All 14 older-Rust checks from
1.90.0 through 1.98.0 and nine no-default cross-target checks passed. Fp2 and
charged-G1 fuzz smokes completed 5,128 and 33,566 executions respectively.
Detailed commands, measurements and limitations are retained in the Fp2 scope
document. Next step: exact-commit pentest. No PASS report or tag is authorized
by this implementation candidate.
