# eth 0.34.0 Release Notes

Status: implementation ready; awaiting pentest.

`0.34.0` refreshes official Ethereum source and fixture pins and adds a
reproducible external reference-store workflow.

## Added

- `spec-lock.toml` now marks the source lock as required and records the
  2026-07-04 check date.
- Refreshed official Ethereum revisions:
  - execution-specs: `4f5c7d19adc916a268b7eadc196756068a325515`
  - execution tests: `c67e485ff8b5be9abc8ad15345ec21aa22e290d9`
  - EIPs: `1b4e9a44c1fd51ffd8afe4f0c404cf51d84cff64`
  - execution-apis: `f74de4b86e3b011384808c294c3d71f2854729a2`
  - consensus-specs: `bd454cb0a6cff1b210ea9de208803df4d9966655`
- `scripts/sync_spec_sources.py` for cloning, updating, and checking pinned
  upstream repositories in the external reference store.
- `docs/reference-store.md` documenting `/home/eldryoth/Work/test/eth`, the
  `ETH_REFERENCE_STORE` override, and fixture license handling.
- Release checks now validate the spec lock in lock-only mode.

## Security Notes

- No protocol behavior changes are introduced in this release.
- Large upstream repositories remain outside the crate package.
- Future consensus-sensitive claims must name exact pinned upstream revisions
  before implementation work proceeds.

## Versioning

- The facade crate publishes as `eth` `0.34.0` for packaged documentation and
  release metadata.
- Support crates are unchanged and are not republished.
