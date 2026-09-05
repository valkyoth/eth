# Security Policy

Release-train tags are authenticated against the public keys in the candidate's
committed `security/release-allowed-signers`, not arbitrary operator keyrings.
Trust-policy edits require review before the report-only release commit.
See [release-control evidence](docs/release-control-remediation-0.56.0.md).

`eth` is security-sensitive protocol software. Treat parsing, fork validation,
proof verification, RPC, signing, EVM execution, Reth integration, P2P, release
scripts, CI, and dependency updates as high-risk until reviewed and tested.

## Routine Checks

Run these regularly and before releases:

```bash
scripts/checks.sh
scripts/check_latest_tools.sh
scripts/release_0_56_0_gate.sh --implementation
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

The maintainer workflow is implementation/testing, pentest and remediation,
permanent report, commit, GitHub checks, then explicit permission to tag/push.
GitHub failures return to fixes/tests/report updates and a new commit.
Consume and delete root `PENTEST.md` after preserving findings and resolutions;
a clean pentest is documented directly. See the [runbook](docs/RELEASE_RUNBOOK.md).

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

The active gate's `--implementation` mode runs portable tests, freshness,
audits, fuzzing and compiler compatibility before review. Default/`--tag` mode
checks final metadata and report readiness only; it does not start clients.
External-client runs remain explicit, fully isolated tests. An unavailable
host is recorded as missing evidence with its scope, not a hidden success or
an automatic additional tag-stage prerequisite for v0.56.0.

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
