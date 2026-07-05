# Crate Version Matrix

Status: `v0.37.4` adds constant-time and reference dependency policy evidence.
The default facade graph keeps `subtle` as a reviewed exception and gates
reference-only crates out of runtime paths.

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

## v0.37.4 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.19.0` | `0.19.0` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-primitives` | `0.11.2` | `0.11.2` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-hash` | `0.11.2` | `0.11.2` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-protocol` | `0.25.2` | `0.25.2` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-verify` | `0.21.0` | `0.21.0` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-derive` | `0.17.2` | `0.17.2` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-sanitization` | `0.7.4` | `0.7.4` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-evm` | `0.8.0` | `0.8.0` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-signer` | `0.7.3` | `0.7.3` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.37.4. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.37.4. |
| `eth` | `0.37.3` | `0.37.4` | `metadata` | Yes | Publishes facade documentation for constant-time and reference dependency policy evidence. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
