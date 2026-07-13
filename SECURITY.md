# Security Policy

`eth` is security-sensitive protocol software. Treat parsing, fork validation,
proof verification, RPC, signing, EVM execution, Reth integration, P2P, release
scripts, CI, and dependency updates as high-risk until reviewed and tested.

## Routine Checks

Run these regularly and before releases:

```bash
scripts/checks.sh
scripts/check_latest_tools.sh
scripts/release_0_1_gate.sh
cargo deny check
cargo audit
scripts/generate-sbom.sh --check
```

GitHub Actions run CI. GitHub CodeQL default setup should be enabled in the
repository security settings. Do not add an advanced CodeQL workflow while
default setup is active.
The verification steps are documented in
[GitHub Security Settings](docs/github-security-settings.md).

## Release Gate

Every release tag must point at a final pentest-report commit. The matching
`security/pentest/vX.Y.Z.md` report must have `Status: PASS`, and
`scripts/validate-release-readiness.sh vX.Y.Z` must pass before the tag is
pushed. Tag-time readiness is enforced by the local release gate scripts before
tag creation; the GitHub release workflow is metadata-only and manual so a
pushed tag is not blocked by a post-tag check that necessarily sees the tag
already exists.

The pentest-report commit must be the direct, linear child of the reviewed
commit. Do not squash-merge or rewrite the release branch between the reviewed
implementation commit and the final report commit.

## Dependency Policy

The dependency policy lives in `deny.toml`. Unknown registries and git sources
are denied by default. Git dependencies require exact `rev` pinning and a
documented exception before use.

New third-party crates require:

- current version check before admission;
- license and maintenance review;
- feature impact review;
- no hidden `std`, network, signer, or native-code expansion in core crates;
- tests for the behavior being admitted;
- `cargo deny check` and `cargo audit` evidence.

## Reporting

Do not publish exploitable security details before a fix is available. Open a
private security advisory or contact the maintainers directly once the public
repository security channels are configured.
