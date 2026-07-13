# eth Supply-Chain Security

`eth` uses overlapping supply-chain controls because each catches a different
class of problem.

## Required Checks

- `cargo deny check` for license, source, advisory, and duplicate policy.
- `cargo audit` for RustSec advisories.
- `scripts/generate-sbom.sh --check` for exact committed SBOM evidence.
- Dependabot for Cargo and GitHub Actions updates.
- Manual current-version review before dependency edits.

## Dependency Admission

Before adding a third-party crate:

- confirm the latest release version;
- review license compatibility with `MIT OR Apache-2.0`;
- inspect default features and `std` requirements;
- avoid git dependencies unless exact `rev` pinning is necessary;
- add tests for the behavior being admitted;
- document security impact in the pull request.

Core crates must not gain network, signer, filesystem, clock, TLS, Reth, or P2P
dependencies.

Use `scripts/generate-sbom.sh --write` only when intentionally refreshing
`sbom/eth.spdx.json`. CI and release readiness use `--check`, which generates a
fresh document and compares all stable SPDX content. The comparator ignores
only cargo-sbom's per-run timestamp, random document namespace, and collection
ordering; package versions, licenses, checksums, references, and relationships
must match.

`docs/core-independence-audit.md` is the release-level inventory for
dependencies that can affect Ethereum hashing, signatures, RLP, trie/proof
behavior, execution, consensus, networking, or RPC semantics. Update that
audit, or the follow-up release that supersedes it, whenever a core-impacting
dependency changes classification.

## Admitted Third-Party Crates

| Crate | Version | License | Default Features | Reason |
| --- | --- | --- | --- | --- |
| `crunchy` | `0.2.4` | `MIT` | enabled | Tiny macro helper pulled by `tiny-keccak`; no direct public API dependency. |
| `proc-macro2` | `1.0.106` | `MIT OR Apache-2.0` | enabled | Token handling for optional derive macros. |
| `quote` | `1.0.46` | `MIT OR Apache-2.0` | enabled | Code generation for optional derive macros. |
| `libfuzzer-sys` | `0.4.13` | `(MIT OR Apache-2.0) AND NCSA` | fuzz-only | LLVM libFuzzer runtime for `fuzz/` targets. |
| `k256` | `0.14.0` | `MIT OR Apache-2.0` | disabled, `ecdsa` enabled | Optional `secp256k1-k256` backend for digest-level sender recovery. |
| `sanitization` | `1.2.4` | `MIT OR Apache-2.0` | disabled | Optional best-effort secret memory clearing bridge. |
| `subtle` | `2.6.1` | `BSD-3-Clause` | disabled, `core_hint_black_box` enabled | Constant-time equality for security-boundary byte comparisons. |
| `syn` | `2.0.118` | `MIT OR Apache-2.0` | enabled, `full` enabled | Syntax parsing for optional derive macros. |
| `tiny-keccak` | `2.0.2` | `CC0-1.0` | disabled, `keccak` enabled | Optional non-default software Keccak-256 backend admitted in v0.27.0. |

`CC0-1.0` is not a globally allowed license in `deny.toml`. The release policy
uses a scoped cargo-deny license exception for `tiny-keccak 2.0.2` only, so any
future CC0 dependency must receive a separate review and exception.
| `trybuild` | `1.0.118` | `MIT OR Apache-2.0` | enabled | Dev-only compile-fail diagnostics for public derive macros. |
