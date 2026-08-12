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
5. Run repository and version-specific gates, then stop for pentest.
6. Remediate findings, delete root `PENTEST.md`, rerun tests, and repeat review
   until clean.
7. Commit the permanent `security/pentest/vX.Y.Z.md` report for the exact
   reviewed implementation commit. Internal reports record
   `Assessment: INCREMENTAL`, the preceding tag as `Baseline`, and the current
   tag as `Range-End`. Public reports use `Assessment: CUMULATIVE` and the
   preceding published checkpoint as `Baseline`.
8. Wait for green GitHub CI and CodeQL, rerun release readiness, and create the
   signed annotated tag only after explicit maintainer authorization.

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
4. Verify permanent pentest evidence exists in every intervening signed tag.
5. Complete the normal report-only commit, hosted checks, signed tag, and
   explicit tag-push authorization.
6. Run `scripts/release_crates.py --require-tag`; dependencies publish first
   and the `eth` facade publishes last.

Post-tag publishing verifies the exact signed tag, permanent evidence, SBOM,
dependency policy, audit status, package plan, and Cargo archives. It does not
rerun environment-dependent integration workloads already required before
tagging.

## Failure Handling

- A failed implementation, pentest, retest, CI, or CodeQL check blocks the tag.
- A missing intermediate tag or pentest report blocks the next public
  checkpoint.
- A crates.io failure resumes at the first unpublished crate only after its
  dependency predecessors are visible.
- Never weaken a security gate to fit the current host. Move the gate to a
  capable host or retain the fail-closed result as an unresolved blocker.
