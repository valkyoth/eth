# Core Dependency Independence Audit

Status: updated by `v0.37.4` constant-time and reference dependency policy
work.

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
the newer `0.14.0-rc.15` is a release candidate. `v0.37.3` removes direct
`sha3` use from `eth-valkyoth-verify` tests in favor of the project hash
boundary with the optional `tiny-keccak` test backend.

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
| `subtle 2.6.1` | `eth -> eth-valkyoth-primitives -> subtle` | Reviewed exception | `v0.37.4` keeps `subtle` as the narrow reviewed exception for fixed-width constant-time equality composition. See `docs/constant-time-reference-policy.md`. |

`k256` and its transitive cryptographic crates no longer enter the default
facade graph. They are admitted only through the explicit `secp256k1-k256`
feature.

## Optional Runtime Dependencies

| Crate | Feature path | Classification | Follow-up |
| --- | --- | --- | --- |
| `tiny-keccak 2.0.2` | `eth/keccak-tiny -> eth-valkyoth-hash/tiny-keccak` | Optional backend | Keep outside default. Backend admission remains documented in `docs/keccak-boundary.md`; future native Keccak work belongs to the hashing track. |
| `k256 0.13.4` | `eth/secp256k1-k256 -> eth-valkyoth-verify/secp256k1-k256` | Optional backend | Keep outside default. Backend admission remains documented in `docs/dependency-admission-k256.md` and `docs/signature-backend-boundary.md`. |
| `sanitization 1.2.2` | `eth/sanitization -> eth-valkyoth-sanitization` | Optional backend | `v0.37.5` documents and gates the optional sanitization bridge path. |
| `serde 1.0.228` | `eth/eip712-json -> eth-valkyoth-verify/json` | Optional backend | `v0.37.5` documents and gates the optional JSON parser boundary. |
| `serde_json 1.0.150` | `eth/eip712-json -> eth-valkyoth-verify/json` | Optional backend | `v0.37.5` documents and gates the optional JSON parser boundary plus existing duplicate-key and limit checks. |

Optional dependencies must remain absent from the default `eth` graph. Any
release that changes their feature path must include a cargo-tree check.

## Dev, Fuzz, And Reference Dependencies

| Crate | Path | Classification | Follow-up |
| --- | --- | --- | --- |
| `alloy-rlp 0.3.16` | `eth-valkyoth-codec` dev-dependency and `fuzz/` dependency | Reference-only | `v0.37.4` documents and gates the quarantine rule for third-party reference oracles. |
| `serde_json 1.0.150` | `eth-valkyoth-codec` dev-dependency | Reference-only | `v0.37.4` documents and gates the dev-only fixture parser rule. |
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
| Sender recovery | First-party `RecoverableSecp256k1` policy boundary | `k256` is optional adapter only. |
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

`v0.37.3` resolves the primary actionable finding from this audit by moving
`k256` behind the explicit `secp256k1-k256` backend feature and adding
backend-aware sender, EIP-712, transaction, and set-code authorization APIs.

`v0.37.4` resolves the constant-time and reference-only follow-up by accepting
`subtle` as the narrow reviewed exception and adding
`scripts/check_runtime_dependency_policy.py` to keep `alloy-rlp`,
fixture-only `serde_json`, optional parser crates, optional backends,
sanitization, REVM, and direct hash/signature implementation crates out of the
default runtime graph.

`v0.37.5` resolves the optional parser and sanitization bridge follow-up by
adding `scripts/check_optional_boundary_policy.py`, documenting the exact
`eip712-json` and `sanitization` feature paths, and capturing separate
dependency-tree evidence for default, JSON, sanitization, and all-feature
graphs.

## Follow-Up Register

| Release | Dependency work |
| --- | --- |
| `v0.37.3` | Completed: secp256k1 recovery moved behind explicit backend/API boundaries and direct `sha3` verify test usage removed. |
| `v0.37.4` | Completed: retained `subtle` as a narrow reviewed exception and added executable quarantine checks for `alloy-rlp` plus dev fixture `serde_json`. |
| `v0.37.5` | Completed: documented and gated optional parser and sanitization bridges so `serde`, `serde_json`, and `sanitization` cannot become accidental defaults. |
| `v0.40.0` through `v0.54.0` | Build first-party EVM execution phases; REVM remains reference or compatibility only if admitted. |
| `v0.46.0` through `v0.52.0` | Admit or implement cryptographic precompile backends only with conformance vectors, dependency review, fuzzing where applicable, and pentest gates. `v0.46.0` begins this with first-party dependency-free SHA-256 and RIPEMD-160. `v0.47.0` adds ECRECOVER execution through caller-provided secp256k1 and Keccak traits without adding default crypto dependencies. `v0.48.0` adds bounded first-party ModExp execution without a bigint dependency. `v0.49.0` adds dependency-free first-party BN254 add/mul execution. `v0.50.0` adds the dependency-free BN254 pairing frame boundary with empty-input execution, `v0.50.1` adds G2 subgroup validation, `v0.50.2` adds the Fp6/Fp12 tower foundation, and `v0.50.3` adds validated tuple streaming while non-empty algebra remains fail-closed for the explicit pairing releases. |
| `v0.61.0` | Admit or reject KZG/blob cryptography backends before blob consensus validation is claimed. |
| `v0.94.0` | Add Kani proof harnesses as extra assurance for selected critical invariants. |

The exit criteria for this release are documentation-only: no core Ethereum
dependency remains accidental or undocumented, and every remaining third-party
core dependency has a boundary, classification, or versioned follow-up.
