# Crate Version Matrix

Status: `v0.36.0` adds the first differential test harness against the
independent `alloy-rlp` RLP implementation; implementation ready for pentest.

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package
contents need a new crates.io version.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | `eth` uses the milestone version, for example `0.20.0`; support crates use their next independent minor, for example `0.8.0` to `0.9.0`. | Yes |
| `dependency` | Patch-bump the existing line, for example `0.3.1` to `0.3.2`. | Yes |
| `metadata` | Use the milestone version when republishing corrected package metadata. | Yes |
| `unchanged` | Keep the previous published version. | No |

`dependency` means the crate did not receive meaningful implementation or API
changes, but its manifest must change because a related workspace crate changed
in a way that the published dependency range cannot cover.

`metadata` means the crate did not receive meaningful implementation changes,
but must be republished so immutable crates.io package metadata is corrected.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests before release. The script refuses accidental lockstep
publication when a crate is marked `unchanged`.

## v0.36.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.18.0` | `0.19.0` | `code` | Yes | Adds the v0.36.0 alloy-rlp differential RLP reference integration test harness. |
| `eth-valkyoth-primitives` | `0.11.1` | `0.11.2` | `dependency` | Yes | Refreshes the published eth-valkyoth-codec dependency to 0.19.0. |
| `eth-valkyoth-hash` | `0.11.1` | `0.11.2` | `dependency` | Yes | Refreshes the published eth-valkyoth-primitives dependency to 0.11.2. |
| `eth-valkyoth-protocol` | `0.25.1` | `0.25.2` | `dependency` | Yes | Refreshes published codec, primitives, and hash dependency versions for v0.36.0. |
| `eth-valkyoth-verify` | `0.20.1` | `0.20.2` | `dependency` | Yes | Refreshes published codec, primitives, hash, and protocol dependency versions for v0.36.0. |
| `eth-valkyoth-derive` | `0.17.1` | `0.17.2` | `dependency` | Yes | Refreshes published codec and primitives dependency versions for v0.36.0. |
| `eth-valkyoth-sanitization` | `0.7.3` | `0.7.4` | `dependency` | Yes | Refreshes the optional published derive dependency to 0.17.2. |
| `eth-valkyoth-evm` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.36.0. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.36.0. |
| `eth-valkyoth-signer` | `0.7.2` | `0.7.3` | `dependency` | Yes | Refreshes the published eth-valkyoth-primitives dependency to 0.11.2. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.36.0. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.36.0. |
| `eth` | `0.35.0` | `0.36.0` | `code` | Yes | Exposes v0.36.0 documentation and refreshed dependency graph for the differential test harness. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
