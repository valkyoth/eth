# Ethereum Reference Store

Status: v0.34.0 pins official Ethereum source and fixture revisions and adds a
reproducible local sync path.

`eth` keeps large upstream Ethereum repositories outside this crate. The
default reference store path is recorded in `spec-lock.toml` as
`../../test/eth`, which resolves from this checkout to:

```text
/home/eldryoth/Work/test/eth
```

Set `ETH_REFERENCE_STORE` to use a different location.

## Sync Command

To clone or update all pinned repositories:

```sh
scripts/sync_spec_sources.py
```

To verify an existing reference store is checked out at the pinned commits:

```sh
scripts/sync_spec_sources.py --check
```

To validate only `spec-lock.toml` without touching the external store:

```sh
scripts/sync_spec_sources.py --lock-only
```

## Pinned Sources

| Source | Repository | Revision |
| --- | --- | --- |
| Execution specs | `https://github.com/ethereum/execution-specs` | `4f5c7d19adc916a268b7eadc196756068a325515` |
| Execution tests | `https://github.com/ethereum/tests` | `c67e485ff8b5be9abc8ad15345ec21aa22e290d9` |
| EIPs | `https://github.com/ethereum/EIPs` | `1b4e9a44c1fd51ffd8afe4f0c404cf51d84cff64` |
| Execution APIs | `https://github.com/ethereum/execution-apis` | `f74de4b86e3b011384808c294c3d71f2854729a2` |
| Consensus specs | `https://github.com/ethereum/consensus-specs` | `bd454cb0a6cff1b210ea9de208803df4d9966655` |

## License Notes

The upstream repositories keep their own licenses and attribution files in the
external reference store. This crate records commit hashes and test
expectations only; it does not vendor those repositories into the package.

Before importing any fixture into this repository, copy the specific upstream
license notice into the same change and document why the fixture must be
vendored instead of read from the external store.
