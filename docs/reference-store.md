# Ethereum Reference Store

Status: `v0.55.0` refreshes official Ethereum source and fixture revisions
through 2026-08-12 and retains the reproducible local sync path.

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

The sync helper accepts only official `https://github.com/ethereum/...`
repositories from `spec-lock.toml` and invokes Git with
`GIT_ALLOW_PROTOCOL=https`. Existing checkouts have their `origin` reset to the
pinned repository before fetch.

To verify an existing reference store is checked out at the pinned commits:

```sh
scripts/sync_spec_sources.py --check
```

`--check` verifies the configured origin, the exact commit hash, and a clean
working tree for every checkout.

To validate only `spec-lock.toml` without touching the external store:

```sh
scripts/sync_spec_sources.py --lock-only
```

## Pinned Sources

| Source | Repository | Revision |
| --- | --- | --- |
| Execution specs | `https://github.com/ethereum/execution-specs` | `2867859a3c19b925f7dc47dae648cca9758f4f80` |
| Execution tests | `https://github.com/ethereum/tests` | `c67e485ff8b5be9abc8ad15345ec21aa22e290d9` |
| EIPs | `https://github.com/ethereum/EIPs` | `582684e2d7d372c09f45777be8ea603e485e9e9d` |
| Execution APIs | `https://github.com/ethereum/execution-apis` | `742d45db810b31265c8d3c075af324953330d1ed` |
| Consensus specs | `https://github.com/ethereum/consensus-specs` | `6d0e95d972a90bbf79a356ded6a704d769bb67c0` |

## License Notes

The upstream repositories keep their own licenses and attribution files in the
external reference store. This crate records commit hashes and test
expectations only; it does not vendor those repositories into the package.

Before importing any fixture into this repository, copy the specific upstream
license notice into the same change and document why the fixture must be
vendored instead of read from the external store.
