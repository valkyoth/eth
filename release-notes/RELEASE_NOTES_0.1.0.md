# eth 0.1.0 Release Notes

Status: planned

## Summary

`0.1.0` establishes the repository foundation for a security-oriented,
`no_std`-first Ethereum execution-layer toolkit.

## Added

- Rust workspace with focused first-party crates.
- EUPL-1.2 license.
- Security, threat model, modularity, supply-chain, implementation, and release
  planning docs.
- Local check and release-gate scripts.
- Granular release plan through `v0.44.0` before `v1.0.0`, with explicit exit
  criteria for each milestone.
- Release-readiness validation for permanent pentest evidence before tags.
- GitHub CI, Dependabot, CODEOWNERS, funding, issue, and pull request metadata.

## Security Notes

- No third-party runtime dependencies are admitted yet.
- Network, signer, EVM, Reth, and P2P functionality are placeholders only.
- This release is not production-ready Ethereum protocol software.

## Verification

```bash
scripts/checks.sh
scripts/release_0_1_gate.sh
```
