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
- review license compatibility with EUPL-1.2;
- inspect default features and `std` requirements;
- avoid git dependencies unless exact `rev` pinning is necessary;
- add tests for the behavior being admitted;
- document security impact in the pull request.

Core crates must not gain network, signer, filesystem, clock, TLS, Reth, or P2P
dependencies.

## Admitted Third-Party Crates

| Crate | Version | License | Default Features | Reason |
| --- | --- | --- | --- | --- |
| `subtle` | `2.6.1` | `BSD-3-Clause` | disabled, `core_hint_black_box` enabled | Constant-time equality for security-boundary byte comparisons. |
