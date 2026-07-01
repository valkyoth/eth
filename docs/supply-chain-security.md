# eth Supply-Chain Security

`eth` uses overlapping supply-chain controls because each catches a different
class of problem.

## Required Checks

- `cargo deny check` for license, source, advisory, and duplicate policy.
- `cargo audit` for RustSec advisories.
- `scripts/generate-sbom.sh` for SBOM evidence.
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

## Admitted Third-Party Crates

| Crate | Version | License | Default Features | Reason |
| --- | --- | --- | --- | --- |
| `proc-macro2` | `1.0.106` | `MIT OR Apache-2.0` | enabled | Token handling for optional derive macros. |
| `quote` | `1.0.46` | `MIT OR Apache-2.0` | enabled | Code generation for optional derive macros. |
| `libfuzzer-sys` | `0.4.13` | `(MIT OR Apache-2.0) AND NCSA` | fuzz-only | LLVM libFuzzer runtime for `fuzz/` targets. |
| `k256` | `0.13.4` | `MIT OR Apache-2.0` | disabled, `ecdsa` enabled | secp256k1 ECDSA recovery for digest-level sender recovery. |
| `sanitization` | `1.2.2` | `MIT OR Apache-2.0` | disabled | Optional best-effort secret memory clearing bridge. |
| `sha3` | `0.10.9` | `MIT OR Apache-2.0` | disabled | Test-only Ethereum Keccak-256 backend for independent sender-recovery vectors; kept on the RustCrypto `digest 0.10` line to avoid duplicate-version policy violations. |
| `subtle` | `2.6.1` | `BSD-3-Clause` | disabled, `core_hint_black_box` enabled | Constant-time equality for security-boundary byte comparisons. |
| `syn` | `2.0.118` | `MIT OR Apache-2.0` | enabled, `full` enabled | Syntax parsing for optional derive macros. |
