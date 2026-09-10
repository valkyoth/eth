# Release Runbook

This is the operational summary. The normative milestone requirements remain
in the [release plan](RELEASE_PLAN.md), and publication classification is
defined by the [versioning policy](VERSIONING_POLICY.md).

## Every Tag

1. Complete the roadmap goal, deliverables, verification, documentation,
   release notes, versions, lockfile, and SBOM work.
2. Set `release-crates.toml` to `stage = "internal"` unless the pre-1.0 minor
   is divisible by five. Keep `baseline` at the preceding published
   checkpoint and `review_baseline` at the immediately preceding tag.
3. List every minor and patch tag after `baseline` through the candidate in
   `cumulative_milestones`.
4. For an internal stage, retain support crates at their published versions,
   move `eth` to the tag version, and select no crate for publication.
5. Run repository and version-specific implementation checks, then explicitly
   stop and ask the maintainer to pentest. For v0.58.0, use
   `scripts/release_0_58_0_gate.sh --implementation`; no final report is needed.
6. If findings arrive in root `PENTEST.md`, fix them, add regression tests,
   update the permanent report/history, and delete the scratch file. Repeat
   review until clean. If the maintainer reports a clean pentest, document that
   result without inventing findings or an additional audit.
7. Commit the permanent `security/pentest/vX.Y.Z.md` report for the exact
   reviewed implementation commit. Internal reports record
   `Assessment: INCREMENTAL`, the preceding tag as `Baseline`, and the current
   tag as `Range-End`. Public reports use `Assessment: CUMULATIVE` and the
   preceding published checkpoint as `Baseline`.
8. Wait for GitHub CI and CodeQL. If the maintainer reports failures, fix and
   test them, update the report, commit again, and wait for the new candidate.
9. When the maintainer confirms GitHub is green and explicitly asks to tag and
   push, run only final metadata/report readiness and create/push the signed
   annotated tag. For v0.58.0, `scripts/release_0_58_0_gate.sh` (or `--tag`)
   performs that admission check, without rerunning implementation workloads.

The report-only commit is bookkeeping handled within this loop, not a new
maintainer approval stage. Commit fixes/documentation first, then update the
report's reviewed commit and commit the report alone. Every later fix requires
fresh applicable tests and report evidence; old GitHub approval does not cover
the new candidate. Never mark unresolved security findings PASS.

Internal milestones stop after their tag is pushed. Do not run the crates.io
publisher; it rejects internal stages.

## Public Checkpoint

At `v0.60.0`, `v0.65.0`, and each later scheduled checkpoint:

1. Classify the complete package delta from `baseline`, not only the newest
   implementation slice.
2. Bump each changed support crate once from its latest published version and
   update all dependent requirements.
3. Run a cumulative integration pentest over the complete range after the
   preceding published checkpoint through the candidate.
4. Authenticate every prior tag with the committed release-signer policy and
   validate its PASS report, reviewed-commit parent, report-only diff and
   assessment chain. Missing scheduled checkpoints/minors block release.
5. Complete the normal report-only commit, hosted checks, signed tag, and
   explicit tag-push authorization.
6. Run `scripts/release_crates.py --require-tag`; dependencies publish first
   and the `eth` facade publishes last.

Post-tag publishing verifies the exact signed tag, permanent evidence, SBOM,
dependency policy, audit status, package plan, and Cargo archives. It does not
rerun implementation or environment-dependent integration workloads.

## Integration Evidence

External-client/Podman runs are separate from routine tag admission, starting
with v0.56.0. Run `scripts/run_differential_tests.py` for all reference paths,
or `scripts/run_modexp_client_differential.py` for the three clients alone.
The implementation gate uses `--in-process` for portable independent oracles.

The client runner still requires rootless Podman with CPU, memory and PID
isolation and fails when those prerequisites are missing. Record such runs as
NOT RUN/host-unavailable in the permanent report; never relabel old evidence
or silently treat a failed client run as passed. The maintainer accepts this
documented limitation for the v0.56.0 field-only milestone. It is not an extra
tag blocker after clean pentest, green GitHub and explicit approval.

This workflow does not establish conformance for untested features. When a
milestone's new behavior depends on external evidence, collect it during
implementation/review or keep that capability unclaimed. Carry this phase
separation into future release gates; do not rewrite historical gate evidence.

## Failure Handling

- A failed portable implementation check, unresolved security finding, failed
  pentest/retest, CI or CodeQL check blocks the tag.
- A missing intermediate tag or pentest report blocks the next public
  checkpoint.
- A crates.io failure resumes at the first unpublished crate only after its
  dependency predecessors are visible.
- Never weaken container isolation to fit the current host. Use a capable
  host for that test or record the missing evidence and affected scope.
