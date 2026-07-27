# eth Supply-Chain Security

`eth` uses overlapping supply-chain controls because each catches a different
class of problem.

## Required Checks

- `cargo deny check` for license, source, advisory, and duplicate policy.
- `cargo audit` for RustSec advisories.
- `scripts/generate-sbom.sh --check` for exact committed SBOM evidence.
- `scripts/check_latest_tools.sh` for latest stable Rust, Cargo
  security/SBOM/fuzz tools, and semantic YAML validation of full-SHA current
  GitHub Action pins across every workflow.
- `scripts/check_latest_crates.py` for MSRV-aware direct dependency freshness.
- Dependabot for Cargo and GitHub Actions updates.
- Manual current-version review before dependency edits.

Action-pin validation uses Ruby's standard-library YAML parser and fails
closed on malformed documents, non-string `uses` values, mutable or
expression-based remote references, and unsupported YAML aliases.
Checkout freshness compares the parsed `uses` value directly with the
upstream release SHA. Semver comments are separately linted and cross-checked
as non-authoritative annotation metadata.
Checkout owner/repository spelling must be canonical lowercase so
case-insensitive repository addressing cannot evade freshness checks. Docker
actions, job containers, and service containers must use explicit
`@sha256:<64-hex>` image digests; mutable tags and expressions fail closed.

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
| `alloy-rlp` | `0.3.16` | `MIT OR Apache-2.0` | disabled | Dev-only independent RLP differential reference. |
| `crunchy` | `0.2.4` | `MIT` | enabled | Tiny macro helper pulled by `tiny-keccak`; no direct public API dependency. |
| `libfuzzer-sys` | `0.4.13` | `(MIT OR Apache-2.0) AND NCSA` | fuzz-only | LLVM libFuzzer runtime for `fuzz/` targets. |
| `k256` | `0.14.0` | `MIT OR Apache-2.0` | disabled, `ecdsa` enabled | Optional `secp256k1-k256` backend for digest-level sender recovery. |
| `proc-macro2` | `1.0.107` | `MIT OR Apache-2.0` | enabled | Token handling for optional derive macros. |
| `quote` | `1.0.47` | `MIT OR Apache-2.0` | enabled | Code generation for optional derive macros. |
| `sanitization` | `2.0.3` | `MIT OR Apache-2.0` | enabled | Optional canonical wiping, protected-container, and runtime protection-report bridge. |
| `serde` | `1.0.229` | `MIT OR Apache-2.0` | enabled | Optional EIP-712 JSON parser data model. |
| `serde_json` | `1.0.151` | `MIT OR Apache-2.0` | enabled | Optional EIP-712 JSON parser; excluded from default and core decode paths. |
| `subtle` | `2.6.1` | `BSD-3-Clause` | disabled, `core_hint_black_box` enabled | Constant-time equality for security-boundary byte comparisons. |
| `syn` | `3.0.3` | `MIT OR Apache-2.0` | enabled, `full` enabled | Syntax parsing for optional derive macros. |
| `tiny-keccak` | `2.0.2` | `CC0-1.0` | disabled, `keccak` enabled | Optional non-default software Keccak-256 backend admitted in v0.27.0. |
| `trybuild` | `1.0.118` | `MIT OR Apache-2.0` | enabled | Dev-only compile-fail diagnostics for public derive macros. |

`CC0-1.0` is not a globally allowed license in `deny.toml`. The release policy
uses a scoped cargo-deny license exception for `tiny-keccak 2.0.2` only, so any
future CC0 dependency must receive a separate review and exception.
