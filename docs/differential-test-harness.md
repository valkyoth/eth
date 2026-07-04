# Differential Test Harness

Status: v0.36.0 starts differential testing with a dev-only `alloy-rlp`
reference path.

The first differential target is structural RLP handling in
`eth-valkyoth-codec`. The harness compares this crate's canonical scalar/list
acceptance and exact re-encoding against `alloy-rlp` header decoding for a
curated corpus of valid and invalid RLP items.

## Scope

v0.36.0 claims one independent reference path:

| Area | Local path | Independent reference | Claim |
| --- | --- | --- | --- |
| Structural RLP | `eth-valkyoth-codec::differential_rlp_reference` | `alloy-rlp` `0.3.16` | Valid/invalid structural RLP decisions match for the curated corpus, and accepted local cases re-encode to identical bytes. |

This is structural RLP coverage. Integer-domain canonicality remains covered by
the codec integer tests, primitive bridge tests, and fuzz targets because a
generic RLP reference cannot distinguish every Ethereum integer-domain rule
from ordinary byte-string validity.

## Commands

Validate the harness configuration:

```sh
scripts/run_differential_tests.py --check
```

The check mode compiles the actual differential integration test with
`--no-run`; it does not rely on a constant success message.

Run the differential harness:

```sh
scripts/run_differential_tests.py
```

The runner executes:

```sh
cargo test -p eth-valkyoth-codec --test differential_rlp_reference --features testing
```

The fuzz workspace also includes `rlp_differential`, which feeds arbitrary
bytes to both `eth-valkyoth-codec` and `alloy-rlp`. The target asserts
structural accept/reject agreement and exact byte-for-byte re-encoding for
accepted values. Explicit `DecodeLimits` resource rejections are treated as
local policy differences, not reference mismatches.

## Mismatch Reporting

The Rust test accumulates all mismatches before failing. Reports include the
case name and whether the mismatch came from the reference rejecting a
claimed-valid case, this crate rejecting a reference-accepted case, this crate
accepting a reference-rejected case, or canonical re-encoding producing
different bytes.
