# Release Notes - eth v0.52.1

Status: implementation complete; awaiting independent pentest and retest.

## Summary

This release adds the first dependency-free BLS12-381 implementation layer:
canonical EIP-2537 field, scalar, point-coordinate, infinity, and complete
precompile-frame parsing. The API rejects malformed encodings before arithmetic
without allocating and preserves the full 256-bit EIP multiplication-scalar
domain.

Curve membership, subgroup validation, map-to-curve, arithmetic, MSM, pairing,
and precompile execution remain fail closed. Those capabilities are assigned to
the following `v0.52.2..=v0.52.9` releases.

## Added

- Canonical 64-byte EIP-2537 Fp parsing with zero-padding and modulus checks.
- Canonical 32-byte Fr parsing below the subgroup order.
- A separate exact 32-byte multiplication-scalar domain accepting all 256-bit
  values, including `q` and larger values.
- Canonical Fp2 parsing in EIP-2537 `c0 || c1` order.
- Exact 128-byte G1 and 256-byte G2 coordinate parsing with unique all-zero
  infinity values.
- Allocation-free parsers for all `0x0b..=0x11` BLS precompile inputs.
- Eager validation plus exact-size fused iterators for G1 MSM, G2 MSM, and
  pairing frames.
- Boundary and round-trip tests for field limits, padding, coefficient order,
  infinity, scalar ranges, exact frames, partial frames, and malformed later
  items.
- The `bls12381_wire` fuzz target and committed seed corpus.
- `docs/bls12-381-wire-encodings.md` defining the release's security boundary.
- `docs/current-status.md` with explicit available, partial, and planned
  capability boundaries.

## Changed

- `eth-valkyoth-evm-core` is minor-bumped from `0.25.0` to `0.26.0` for the new
  public wire/frame APIs and tightened execution lifecycle contract.
- `eth-valkyoth-verify` is minor-bumped from `0.22.0` to `0.23.0` for
  fail-closed EIP-712 identifier, uniqueness, and partial-output behavior.
- `eth` is bumped from `0.52.0` to `0.52.1` and exposes the new domains through
  the optional `evm-core` feature while consuming the hardened verification
  boundary.
- Existing advanced-precompile policies now share the public BLS wire-size
  constants instead of duplicating frame literals.
- The pinned stable toolchain and full release gate now use Rust `1.97.0`;
  Rust `1.90.0` through `1.96.1` retain individual all-feature compatibility
  checks.
- The facade README now leads with the crate's concrete purpose, installation,
  a working bounded transaction-envelope example, and a compact support table
  instead of release-note-style implementation inventory.

## Security Notes

- No crypto, allocator, bigint, BLS, or other runtime dependency is added.
- Fp rejects any non-zero byte in the 16-byte EIP padding and every value at or
  above the base-field modulus.
- Fr and EIP multiplication scalars are distinct types so callers cannot
  accidentally reject valid full-width precompile scalars or admit a non-Fr
  value where canonical scalar-field membership is required.
- Variable frames are non-empty, exact multiples of their item size, bounded by
  the existing 1 MiB precompile input ceiling, and fully validated before a
  borrowed frame is returned.
- G1/G2 point types represent canonical wire coordinates only. They are not
  evidence of curve or subgroup validity, and arithmetic execution remains
  unavailable.
- Public precompile bytes are not treated as secret material. Secret-bearing
  reuse requires a separate constant-time and sanitization contract.
- Caller-provided EVM memory is zeroed on construction. An `EvmExecution` can
  run only once until explicit destructive reset clears its stack, memory, and
  program counter.
- Failed or reverted stateful execution restores the caller-provided warm/cold
  access checkpoint, preventing retries or reverted scopes from inheriting
  newly warmed entries.
- Borrowed and JSON EIP-712 schemas reject invalid struct/field identifiers.
  Atomic-looking aliases, invalid widths, and fixed-point spellings cannot be
  reinterpreted as custom structs.
- Borrowed schemas and values reject duplicate names before hashing and cap
  each struct at 64 fields and 64 named values before quadratic duplicate
  checks begin.
- `encode_eip712_data` clears its selected output region if a later field
  fails, so partial signing material is not left in pooled buffers.

## Verification

- Official EIP-2537 encoding rules checked on 2026-07-16.
- Official EIP-712 type and identifier rules checked on 2026-07-16.
- `cargo test -p eth-valkyoth-evm-core bls12_wire_tests --all-features`
- `cargo test -p eth-valkyoth-evm-core --all-features`
- `cargo test -p eth-valkyoth-verify --all-features`
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`
- `cargo clippy --manifest-path fuzz/Cargo.toml --bin bls12381_wire -- -D warnings`
- `scripts/materialize_fuzz_seeds.py --check`
- `scripts/release_crates.py --check`
- `scripts/checks.sh`
- `cargo +<toolchain> check --workspace --all-features` for every Rust release
  from `1.90.0` through `1.96.1`
- Full release gate on pinned Rust `1.97.0`

## Pentest

Tagging is blocked until the independent pentest findings are remediated, the
retest passes, and `security/pentest/v0.52.1.md` records `Status: PASS`.
