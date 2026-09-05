# v0.56.0 Release-Control Remediation

Status: fixes implemented; external retest pending. Not a PASS attestation.

The incremental review of `v0.55.0` through
`4bb67147e72f6ac3168403c44e781a2246d61cb9` reported two Medium release-integrity
findings and one Low dependency-provenance finding. No arithmetic finding was
confirmed. This remediation changes no Rust, dependency versions or field APIs.

## F1: Shortened Cumulative Baseline

The preceding public checkpoint is derived from the fixed five-minor cadence,
not accepted from supplied metadata. Its tag must exist, as must every scheduled
intermediate minor. All intervening minor/patch tags must be listed, signed by
an authorized release key, ancestors of the candidate, and in chronological
ancestry order. Stale, internal, missing and future baselines fail closed.
The separate 1.0 candidate policy is not inferred from pre-1.0 arithmetic.

## F2: Unauthenticated Or Failed Prior Reports

Readiness now inspects committed reports for the entire train, including the
public baseline. Each report requires exactly one nonempty status, reviewed
commit, tester, scope and real calendar date. The status must be PASS and the
report must be a direct, single-parent, report-only child of its reviewed
commit. Internal reports bind to their actual predecessor and current range
end; public checkpoint reports use the cumulative baseline contract. The
historical 0.55 anchor retains its original report format, without newly
invented Assessment fields, but receives the same signature/lineage checks.

Signature verification uses only `security/release-allowed-signers` from the
candidate's committed HEAD, not the operator's general keyring or a dirty
policy file. The existing release key is identified by fingerprint
`SHA256:EoLRQ5k4J5pYz3UMFmkrV798gYFNkToGS2xEPvebqB4`, verified against the
published v0.55.0 tag. This file contains a public key only. Key rotation is a
reviewed policy change before the report-only commit; retain historical keys
needed to authenticate the release train. Invalid or untrusted signatures,
lightweight tags, merge report commits and unrelated ancestry are rejected.

## F3: Inherited Dependency Contracts

Cumulative change detection now compares effective Cargo dependency contracts
against an authenticated baseline snapshot in a temporary directory. It covers
requirements, features, default features, optionality, dependency kinds,
targets, renames, registry/source identities and package feature maps.
Workspace-relative paths are normalized; checkout relocation alone is not a
package change. Unsupported escaping paths or nonregular archive entries fail
closed rather than influencing a release comparison.

`cargo metadata --offline --no-deps` runs from the reviewed working directory
with an explicit snapshot manifest, without modifying the active checkout or
building the baseline. Source-path changes and internal version dependency
closure remain enforced as before. Library dependency contracts and lockfile
resolution are distinct: SBOM and transitive lockfile review remain required.

## Regression Evidence

The tests use disposable repositories and temporary SSH keys, never real tag
mutation or publication. Signed positive fixtures exercise the actual readiness
shell script for internal and public candidates, including a patch milestone.
Only SBOM generation is stubbed in these small readiness fixtures; real
signature verification and evidence/ancestry checks run unchanged.

Negative cases cover shortened and stale baselines, missing checkpoint/minor
tags, omitted patches, unsigned/untrusted/corrupted tags, dirty trust-policy
replacement, FAIL/empty/duplicate-field reports, invalid dates and reviewed
commits, wrong assessment/range/predecessor, unrelated history, merge parents
and mixed code/report commits. Cargo fixtures test root-only inherited version,
feature, default-feature and target-specific alias changes, correct dependency
release classification, and equivalent contracts across temporary paths.

```sh
python3 scripts/test-release-train.py
python3 scripts/test-release-evidence.py
python3 scripts/test-release-dependencies.py
python3 scripts/test-release-crates.py
scripts/test-release-readiness.sh
scripts/checks.sh
```

Full release admission still requires external retest, the permanent report,
green CI/CodeQL and the previously documented capable Podman host. The ignored
root PENTEST.md is consumed and removed; this document preserves the actionable
findings without claiming an independent retest has occurred.

Reference contracts: [Git SSH signature trust](https://git-scm.com/docs/git-config#Documentation/git-config.txt-gpgsshallowedSignersFile),
[Cargo configuration discovery](https://doc.rust-lang.org/cargo/reference/config.html),
and [Cargo metadata fields](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html).
