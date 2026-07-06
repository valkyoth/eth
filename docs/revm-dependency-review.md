# REVM Dependency Review

Status: v0.37.1 adds the upstream recheck automation; REVM remains
non-admitted.

## Decision

REVM is not part of the `eth` dependency graph in v0.40.0.

The review was performed on 2026-07-05. It must be refreshed before
2026-10-05, or before any REVM/external execution backend is admitted,
whichever is sooner. First-party boundary-only work in `eth-valkyoth-evm` does
not admit REVM. The review checked the latest registry line and the newest line
compatible with the workspace's Rust `1.90.0` floor:

| Candidate | Version | Rust requirement | Result |
| --- | --- | --- | --- |
| `revm` latest | `41.0.0` | `1.91.0` | rejected for current MSRV range |
| `revm` newest MSRV-compatible line | `36.0.0` | `1.88.0` | rejected by dependency policy |
| `revm-primitives` matching `revm 36.0.0` | `22.1.0` | `1.88.0` | rejected by dependency policy |

## Policy Findings

`revm 36.0.0`, even with default features disabled, pulls the execution,
precompile, handler, interpreter, context, database, state, and primitive graph.
That graph failed the current dependency policy because it introduced:

- a `CC0-1.0` transitive dependency through `aurora-engine-modexp`;
- duplicate crypto/hash lines such as `digest`, `block-buffer`,
  `crypto-common`, `cpufeatures`, and `hashbrown`;
- the unmaintained `paste` advisory through `alloy-primitives`.

The narrower `revm-primitives 22.1.0` candidate avoids the precompile license
issue but still introduces duplicate crypto/hash lines and the unmaintained
`paste` advisory.

## Release Boundary

`eth-valkyoth-evm` publishes a first-party `RevmDependencyReview` value in
v0.37.0 so downstream users can see the reviewed decision from code and docs.
`v0.38.0` adds first-party execution boundary types, `v0.39.0` adds
first-party bounded gas-estimation boundary types, and `v0.40.0` starts the
dependency-free native EVM core type crate. No REVM feature or dependency is
exposed until a future review can pass:

- `cargo deny check`;
- Rust `1.90.0` through `1.96.1` compatibility;
- no default feature expansion into the core facade graph;
- the normal pentest-before-tag flow.

## Recheck

The v0.37.1 release adds `scripts/check_ethereum_upstream.py` as the follow-up
dependency recheck before any temporary execution adapter is implemented.
`v0.40.0` through `v0.47.0` reserve the first-party native EVM engine path, so
a future REVM adapter is reference or compatibility infrastructure rather than
the trusted 1.0 execution core.

The advisory upstream-monitoring script is used for maintenance planning. It
checks the latest REVM registry line and official Ethereum hardfork/spec and
fixture revisions, then reports whether a maintenance release may be needed for
fork, opcode, gas, precompile, transaction-type, or fixture changes.

The upstream checker must fetch metadata only and compare it to pinned local
state. It must not pipe network responses to a shell, use `eval`, or execute
fetched content. Official sources become implementation inputs only after exact
commit or tag SHAs are pinned.
