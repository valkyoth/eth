# Constant-Time And Reference Dependency Policy

Status: `v0.37.4` policy implemented; awaiting pentest before tagging.

This document closes the `v0.37.4` dependency-policy slice from the core
independence audit. It covers the default `subtle` runtime dependency and the
reference-only `alloy-rlp` plus dev fixture `serde_json` paths.

## Constant-Time Helper Policy

`eth-valkyoth-primitives` keeps `subtle 2.6.1` as a reviewed exception for
fixed-width equality composition. This is intentional for `v0.37.4`:

- the crate is `no_std` compatible;
- default features are disabled;
- only `core_hint_black_box` is enabled;
- public fixed-width primitives can return `subtle::Choice` so callers can
  compose equality checks without forcing an early branch;
- replacing it with an unreviewed first-party constant-time helper right now
  would increase risk more than it would reduce dependency surface.

The policy is deliberately narrow. `subtle` is accepted only for constant-time
helper behavior around fixed-width security domains. It must not become a
source of Ethereum consensus logic, cryptographic algorithms, parsing, hashing,
or signature recovery.

If the public `subtle::Choice` exposure becomes too broad, a later release may
add a first-party wrapper type. That wrapper must preserve composition behavior
and include timing-focused review before replacing the current API.

## Reference-Only RLP Oracle Policy

`alloy-rlp` is admitted only as an independent reference oracle for tests,
fuzzing, and differential evidence. It must not appear in runtime dependencies,
normal crate dependencies, public APIs, or production encoding/decoding paths.

Allowed locations:

- `eth-valkyoth-codec` dev-dependencies for differential integration tests;
- `fuzz/Cargo.toml` for the RLP differential fuzz harness.

Every production RLP behavior remains first-party in `eth-valkyoth-codec`.
Reference mismatches must be reviewed as either local policy differences or
bugs; the reference crate is not allowed to define consensus behavior.

## Dev Fixture Parser Policy

`serde_json` is admitted in `eth-valkyoth-codec` dev-dependencies only for
test fixture parsing. Fixture parsing must stay outside runtime crate paths and
must not be re-exported.

The optional EIP-712 JSON parser boundary in `eth-valkyoth-verify` is a
separate feature-gated parser path. Its detailed parser and sanitization policy
is scheduled for `v0.37.5`.

## Executable Gate

`scripts/check_runtime_dependency_policy.py` enforces this policy by checking:

- the default `eth` runtime tree excludes reference, parser, backend,
  sanitization, REVM, and direct hash/signature implementation crates;
- `alloy-rlp` is absent from normal and build dependencies;
- `alloy-rlp` dev usage is limited to `eth-valkyoth-codec`;
- codec fixture `serde_json` dev usage stays limited to
  `eth-valkyoth-codec`;
- any `serde_json` normal dependency outside the scheduled optional verify
  parser path fails the gate.

The `v0.37.4` release gate captures default and all-feature cargo-tree evidence
under `target/release_0_37_4_*_tree.txt`.
