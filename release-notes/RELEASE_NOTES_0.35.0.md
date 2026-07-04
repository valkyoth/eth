# eth 0.35.0 Release Notes

Status: tagged and published.

`0.35.0` adds the first external Ethereum execution fixture harness. The
claimed fixture set is deliberately narrow: pinned `ethereum/tests` `RLPTests`
are run through `eth-valkyoth-codec`, while broader transaction, blockchain,
genesis, trie-construction, difficulty, and EOF fixture groups remain explicitly
unsupported until their matching protocol layers exist.

## Added

- `conformance/execution-fixtures.toml` records the claimed and unsupported
  execution fixture groups for this release.
- `scripts/run_execution_fixtures.py` validates the fixture manifest and can
  run claimed fixtures against a pinned `ethereum/tests` checkout.
- `eth-valkyoth-codec` now has an `execution_rlp_fixtures` integration test
  that decodes and re-encodes valid canonical RLP fixture outputs and rejects
  fixtures marked `INVALID`.
- `docs/execution-fixture-harness.md` documents the runner and external
  reference-store workflow.
- `docs/execution-fixture-report.md` records the v0.35.0 claimed pass set.
- `docs/unsupported-execution-fixtures.md` lists fixture groups not claimed by
  this crate yet.

## Security Notes

- The fixture runner verifies the `ethereum/tests` checkout origin, pinned
  commit hash, and clean working tree before running fixture tests.
- The fixture runner reuses the spec-source repository and revision validators
  and hardens git subprocess calls with disabled hooks/fsmonitor, HTTPS-only git
  transport, disabled terminal prompts, and timeouts.
- The v0.35.0 release gate materializes the pinned `execution_tests` checkout
  and runs the actual pinned corpus before tagging. Release CI validates the
  fixture manifest in `--check` mode without requiring large upstream fixture
  repositories in the crate package.
- The Rust fixture walker skips symlinks and caps directory recursion depth.
- No broad execution-test compatibility is implied by this release. Only the
  `RLPTests` fixture group is claimed, and that claim is structural RLP fixture
  coverage rather than integer-domain canonicality coverage.

## Versioning

- `eth-valkyoth-codec` publishes as `0.18.0` for the new conformance test
  package surface.
- Downstream crates with only refreshed dependency requirements use patch
  bumps.
- The facade crate publishes as `eth` `0.35.0`.
