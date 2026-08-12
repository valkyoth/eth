# Versioning And Publication Policy

## Tagged Milestones

Every roadmap version keeps its ordinary `vX.Y.Z` GitHub tag. Before any tag,
the exact candidate must complete its implementation stop, automated release
gate, pentest, remediation and clean retest when needed, permanent report,
GitHub CI, and CodeQL review. Patch milestones receive the same treatment and
do not alter the publication schedule.

Beginning after the published `v0.55.0` baseline, pre-1.0 tags form
five-minor publication trains:

- `v0.56.0` through `v0.59.x` are tagged internal source milestones;
- `v0.60.0` is the next public crates.io checkpoint;
- this repeats at `v0.65.0`, `v0.70.0`, and every later minor divisible by
  five;
- intermediate patch tags stay inside their current train;
- `v1.0.0-rc.N` and `v1.0.0` follow their separately planned production
  admission process.

No intermediate tag publishes crates. The signed tag and permanent pentest
report remain public GitHub evidence and make each implementation slice easy
to review independently.

## Pentest Scope

An internal milestone receives an incremental pentest against the immediately
preceding tag. A public checkpoint receives a cumulative integration pentest
covering every change after the preceding published checkpoint through the
exact candidate. The checkpoint gate also verifies that every intervening
minor and patch tag is represented in `cumulative_milestones`; no internal
slice may disappear from the publication range.

Incremental reports reduce review size but do not replace the cumulative
checkpoint assessment. Findings are remediated and retested under the normal
project workflow before either kind of tag is created.

## Crate Versions

The `eth` facade source version always follows the GitHub tag. Supporting
crates retain their latest published versions during internal milestones even
when their source changes. At a public checkpoint:

- cumulative code changes receive one appropriate independent minor bump;
- API-compatible bug fixes receive one patch bump;
- dependency-only changes receive one patch bump;
- unchanged crates keep their published version and are not uploaded;
- changed dependencies publish before dependants and `eth` publishes last.

This prevents unpublished support-crate versions from entering dependency
requirements while preserving independent crate versioning. Package changes
are compared cumulatively against the preceding public checkpoint.

## Enforced Metadata

`release-crates.toml` records:

- `stage`: `internal` or `public`;
- `baseline`: the preceding public checkpoint;
- `review_baseline`: the immediately preceding GitHub tag;
- `cumulative_milestones`: every minor and patch tag after `baseline` through
  the current candidate.

`scripts/release_train.py` validates the cadence and complete tag chain.
`scripts/release_crates.py` refuses all crates.io publication when
`stage = "internal"` and verifies cumulative package and dependency changes at
public checkpoints. Release readiness also requires internal release notes to
record `Publication: DEFERRED TO v0.N.0`, public checkpoint notes to record
`Publication: PENDING`, and matching incremental or cumulative report fields.
