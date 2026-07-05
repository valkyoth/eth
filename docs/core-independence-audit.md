# Core Dependency Independence Audit

Status: v0.37.2 implementation ready for pentest.

This audit records every third-party dependency that can influence core
Ethereum behavior before execution work continues. The long-term project goal
is first-party protocol logic for wire formats, validation rules, trie/state
behavior, execution semantics, and node-facing boundaries. Third-party crates
may still be used when they are explicitly classified as an optional backend,
reference oracle, compile-time tool, or reviewed cryptographic exception.

Evidence was gathered on 2026-07-05 with:

```text
cargo tree -p eth -e features --no-default-features
cargo tree -p eth -e features --all-features
cargo tree -e features --workspace
```

The same pass checked current crates.io metadata with `cargo info` for
`k256`, `sha3`, `tiny-keccak`, `subtle`, `alloy-rlp`, `serde`, `serde_json`,
`sanitization`, `proc-macro2`, `quote`, `syn`, and `trybuild`. The notable
version findings are that `k256 0.13.4` remains the selected stable line while
the newer `0.14.0-rc.15` is a release candidate, and `sha3 0.10.9` has a newer
`0.12.0` line but is currently dev-only and assigned to `v0.37.3` review.

## Classifications

| Class | Meaning |
| --- | --- |
| First-party | Implemented in `eth-valkyoth-*` crates and controlled by this workspace. |
| Optional backend | Not in the default facade graph; callers opt in by feature. |
| Reference-only | Used for tests, fuzzing, or differential evidence only. |
| Compile-time only | Used to build macros or diagnostics, not runtime protocol behavior. |
| Reviewed exception | Default runtime dependency accepted because replacing it now would increase risk. |
| Temporary debt | Default runtime dependency that conflicts with the first-party-core goal and has a follow-up release. |

## Default Facade Graph

`eth` with no features still depends on these first-party crates:

- `eth-valkyoth-codec`
- `eth-valkyoth-hash`
- `eth-valkyoth-primitives`
- `eth-valkyoth-protocol`
- `eth-valkyoth-verify`

The default third-party runtime dependencies visible from that graph are:

| Crate | Path | Classification | Follow-up |
| --- | --- | --- | --- |
| `k256 0.13.4` | `eth -> eth-valkyoth-verify -> k256` | Temporary debt | `v0.37.3` moves secp256k1 recovery behind an explicit backend boundary, compatibility adapter, or documented cryptographic exception. |
| `subtle 2.6.1` | `eth -> eth-valkyoth-primitives -> subtle` | Reviewed exception | `v0.37.4` reviews constant-time utility policy and decides whether to retain `subtle`, wrap it more narrowly, or replace the public use with first-party code. |

`k256` currently brings transitive cryptographic crates such as `ecdsa`,
`elliptic-curve`, `sha2`, `digest`, `crypto-bigint`, `ff`, `group`,
`rand_core`, `rfc6979`, and `zeroize`. They are classified under the same
`k256` debt because they enter through the same default signature path.

## Optional Runtime Dependencies

| Crate | Feature path | Classification | Follow-up |
| --- | --- | --- | --- |
| `tiny-keccak 2.0.2` | `eth/keccak-tiny -> eth-valkyoth-hash/tiny-keccak` | Optional backend | Keep outside default. Backend admission remains documented in `docs/keccak-boundary.md`; future native Keccak work belongs to the hashing track. |
| `sanitization 1.2.2` | `eth/sanitization -> eth-valkyoth-sanitization` | Optional backend | `v0.37.5` reviews optional sanitization bridge wording before execution/signing state grows. |
| `serde 1.0.228` | `eth/eip712-json -> eth-valkyoth-verify/json` | Optional backend | `v0.37.5` reviews JSON/parser boundary policy and ensures no default parser dependency creep. |
| `serde_json 1.0.150` | `eth/eip712-json -> eth-valkyoth-verify/json` | Optional backend | `v0.37.5` reviews JSON/parser boundary policy and checks duplicate-key and limit gates remain documented. |

Optional dependencies must remain absent from the default `eth` graph. Any
release that changes their feature path must include a cargo-tree check.

## Dev, Fuzz, And Reference Dependencies

| Crate | Path | Classification | Follow-up |
| --- | --- | --- | --- |
| `alloy-rlp 0.3.16` | `eth-valkyoth-codec` dev-dependency and `fuzz/` dependency | Reference-only | `v0.37.4` documents the quarantine rule for third-party reference oracles and keeps `alloy-rlp` out of runtime crates. |
| `sha3 0.10.9` | `eth-valkyoth-verify` dev-dependency | Reference-only | `v0.37.3` replaces direct test use with the project hash boundary where practical, or records why it remains a dev-only independent KAT oracle. |
| `serde_json 1.0.150` | `eth-valkyoth-codec` dev-dependency | Reference-only | `v0.37.4` documents reference-only fixture parsing rules. |
| `trybuild 1.0.117` | `eth-valkyoth-derive` dev-dependency | Compile-time only | No runtime core impact; keep dev-only. |

Reference-only crates must never be re-exported by runtime crates or used as
the implementation source of consensus behavior. Their job is to catch
mismatches against independent implementations and fixture formats.

## Compile-Time Macro Dependencies

| Crate | Path | Classification | Follow-up |
| --- | --- | --- | --- |
| `proc-macro2 1.0.106` | `eth-valkyoth-derive` | Compile-time only | Keep under derive crate; no runtime protocol behavior. |
| `quote 1.0.46` | `eth-valkyoth-derive` | Compile-time only | Keep under derive crate; no runtime protocol behavior. |
| `syn 2.0.118` | `eth-valkyoth-derive` | Compile-time only | Keep under derive crate; no runtime protocol behavior. |

The derive crate is optional at the workspace layer. Macro dependencies do not
enter the default runtime facade graph.

## Core Behavior Inventory

| Behavior area | Current implementation source | Third-party impact |
| --- | --- | --- |
| RLP scalar/list/integer codec | First-party `eth-valkyoth-codec` | `alloy-rlp` is dev/fuzz reference only. |
| Primitive domains | First-party `eth-valkyoth-primitives` | `subtle` supplies constant-time equality helpers. |
| Keccak hashing boundary | First-party `eth-valkyoth-hash` trait | `tiny-keccak` is optional backend only. |
| Transaction decode/encode | First-party `eth-valkyoth-protocol` | No runtime third-party parser dependency. |
| Fork/context validation | First-party `eth-valkyoth-protocol` and `eth-valkyoth-verify` | No runtime third-party source. |
| MPT decode/proofs | First-party `eth-valkyoth-verify` | Caller-provided Keccak backend only. |
| Sender recovery | First-party policy around `k256` backend | `k256` is current default temporary debt. |
| EIP-712 JSON parsing | First-party limits over optional `serde_json` parser | Optional backend parser support only. |
| EVM execution | First-party plan, REVM not admitted | No REVM dependency in graph. |
| RPC, Reth, networking | Placeholder first-party crates | No third-party network dependency admitted. |

## Cryptographic Backend Policy

Cryptographic primitives are not automatically safer when hand-written. For
curve arithmetic, signature verification, KZG, hash backends, and future
hardware/HSM integrations, the project uses this order:

1. Keep the public API first-party and domain-specific.
2. Prefer caller-provided trait boundaries for cryptographic engines.
3. Admit software backends only behind explicit features or compatibility
   crates unless a reviewed exception is documented.
4. Require KATs, malformed-input tests, feature review, license review, MSRV
   review, and cargo-deny/audit evidence for every backend.
5. Schedule first-party or formally audited replacements only when doing so
   lowers risk compared with a reviewed backend.

`k256` currently violates item 3 because it is a default runtime dependency of
`eth-valkyoth-verify`. That is the primary actionable finding from this audit
and is assigned to `v0.37.3`.

## Follow-Up Register

| Release | Dependency work |
| --- | --- |
| `v0.37.3` | Move secp256k1 recovery behind explicit backend/API boundaries and review `sha3` dev-test usage. |
| `v0.37.4` | Review `subtle` constant-time utility policy and quarantine rules for reference-only crates such as `alloy-rlp` and dev `serde_json`. |
| `v0.37.5` | Review optional parser and sanitization bridges so `serde`, `serde_json`, and `sanitization` cannot become accidental defaults. |
| `v0.40.0` through `v0.47.0` | Build first-party EVM execution phases; REVM remains reference or compatibility only if admitted. |
| `v0.54.0` | Admit or reject KZG/blob cryptography backends before blob consensus validation is claimed. |
| `v0.87.0` | Add Kani proof harnesses as extra assurance for selected critical invariants. |

The exit criteria for this release are documentation-only: no core Ethereum
dependency remains accidental or undocumented, and every remaining third-party
core dependency has a boundary, classification, or versioned follow-up.
