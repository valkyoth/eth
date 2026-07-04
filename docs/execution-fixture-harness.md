# Execution Fixture Harness

Status: v0.35.0 starts the external Ethereum execution-test harness with the
pinned RLP fixture corpus.

The fixture manifest is
[`conformance/execution-fixtures.toml`](../conformance/execution-fixtures.toml).
It names the exact `ethereum/tests` revision from `spec-lock.toml`, the fixture
groups currently claimed by this crate, and the fixture groups deliberately not
claimed yet.

## Claimed Fixtures

v0.35.0 claims `RLPTests` from `ethereum/tests` at
`c67e485ff8b5be9abc8ad15345ec21aa22e290d9`.

The Rust integration test lives in `eth-valkyoth-codec` and decodes every
fixture `out` byte sequence, re-encodes valid canonical cases, and rejects
fixtures marked `INVALID`.

This is structural RLP conformance coverage. It does not claim Ethereum integer
domain validation beyond the RLP scalar/list rules exercised by the fixture
corpus; integer canonicality remains covered by the codec integer tests and
primitive bridge tests.

## Commands

Validate the manifest without requiring a local checkout:

```sh
scripts/run_execution_fixtures.py --check
```

Run the claimed fixtures against the configured external reference store:

```sh
scripts/run_execution_fixtures.py
```

Run against an explicit pinned `ethereum/tests` checkout:

```sh
scripts/run_execution_fixtures.py --execution-tests /path/to/ethereum-tests
```

The runner verifies the checkout origin, pinned commit hash, and clean working
tree before executing the Rust fixture test.

`scripts/release_0_35_gate.sh` runs this command without `--check`, so the
pre-tag release gate fails if the pinned fixture checkout is unavailable or
dirty. General CI uses `--check` to validate the manifest without packaging the
external corpus.
