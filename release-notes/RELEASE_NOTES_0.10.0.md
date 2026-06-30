# eth 0.10.0 Release Notes

Status: follow-up pentest remediation complete; retest required before tag

## Summary

`0.10.0` establishes the RLP fuzz harness baseline before transaction envelope
parsing expands the untrusted byte surface.

The release keeps live fuzz corpus growth out of git, commits reviewed fixture
seeds as hexadecimal text, and documents crash reproduction so every future RLP
parser change has an explicit fuzz target to update.

## Included So Far

- Added a combined `rlp` fuzz target that drives scalar, integer, bounded
  integer, and list decoders through exact and partial decode paths.
- Kept the existing focused fuzz targets for scalars, lists, integers, encoding,
  primitive RLP bridge helpers, and decode-budget accounting.
- Added committed hex seed corpus files under `fuzz/seed-corpus/`.
- Added `scripts/materialize_fuzz_seeds.py` to validate committed seeds and
  materialize them into ignored `fuzz/corpus/` directories for local campaigns.
- Added `docs/fuzzing.md` with target scope, seed handling, campaign commands,
  and crash reproduction.
- Added `scripts/release_0_10_gate.sh`.
- Updated release metadata so `eth-valkyoth-codec`,
  `eth-valkyoth-primitives`, `eth-valkyoth-hash`, and `eth` publish for this
  release.
- Addressed initial pentest findings by documenting and bounding list
  iteration recursion, fuzzing list traversal to the decoder hard cap, adding
  signed `ChainId` decode helpers, adding a Keccak empty-input KAT helper,
  making unexpected primitive codec errors release-visible, simplifying
  transaction type constructors, removing the list iterator
  `ExactSizeIterator` contract, and adding explicit reviewed decode-policy
  construction.
- Addressed follow-up pentest findings by giving
  `Keccak256ConformanceError` standard formatting/error behavior and adding
  `verify_empty_digest_with` for configured hashers that cannot implement
  `Default`.

## Known Limitations

- CI builds the fuzz workspace but does not run long fuzz campaigns.
- Transaction-envelope parsers are still planned for later releases.
- Crash artifacts remain local until a minimized case is converted into a
  reviewed hex seed or deterministic regression test.

## Still Required Before Tag

- Maintainer retest must be run for the exact remediation commit.
- Any follow-up pentest findings must be fixed and retested.
- A permanent report must be written at `security/pentest/v0.10.0.md`.
- GitHub checks must pass on the final release report commit.

## Verification

```bash
scripts/materialize_fuzz_seeds.py --check
cargo check --manifest-path fuzz/Cargo.toml
scripts/checks.sh
scripts/release_0_10_gate.sh
cargo deny check
cargo deny --manifest-path fuzz/Cargo.toml check
cargo audit
scripts/release_crates.py --check
scripts/release_crates.py --dry-run --skip-checks --yes
```
