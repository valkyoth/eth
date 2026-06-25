# Crate Version Matrix

Status: active for `v0.6.0` development

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package
contents need a new crates.io version.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | Use the milestone version, for example `0.5.0`. | Yes |
| `dependency` | Patch-bump the existing line, for example `0.3.1` to `0.3.2`. | Yes |
| `unchanged` | Keep the previous published version. | No |

`dependency` means the crate did not receive meaningful implementation or API
changes, but its manifest must change because a related workspace crate changed
in a way that the published dependency range cannot cover.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests before release. The script refuses accidental lockstep
publication when a crate is marked `unchanged`.

## v0.6.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.5.0` | `0.5.0` | `unchanged` | No | Codec remains on the published v0.5.0 API until the v0.6.0 RLP scalar decoder lands. |
| `eth-valkyoth-primitives` | `0.5.0` | `0.5.0` | `unchanged` | No | Primitive domains remain unchanged for the initial v0.6.0 dependency refresh. |
| `eth-valkyoth-protocol` | `0.5.0` | `0.5.0` | `unchanged` | No | Protocol state remains unchanged for the initial v0.6.0 dependency refresh. |
| `eth-valkyoth-verify` | `0.5.0` | `0.5.0` | `unchanged` | No | Verification boundaries remain unchanged for the initial v0.6.0 dependency refresh. |
| `eth-valkyoth-derive` | `0.5.0` | `0.6.0` | `code` | Yes | Starts v0.6.0 by refreshing derive macro dependencies, including `quote` 1.0.46. |
| `eth-valkyoth-sanitization` | `0.5.0` | `0.6.0` | `code` | Yes | Starts v0.6.0 by refreshing optional sanitization support to `sanitization` 1.2.2. |
| `eth-valkyoth-evm` | `0.3.0` | `0.3.0` | `unchanged` | No | EVM boundary remains unchanged for the initial v0.6.0 dependency refresh. |
| `eth-valkyoth-rpc` | `0.3.0` | `0.3.0` | `unchanged` | No | RPC boundary remains unchanged for the initial v0.6.0 dependency refresh. |
| `eth-valkyoth-signer` | `0.3.2` | `0.3.2` | `unchanged` | No | Signer remains on the published v0.3.2 dependency range. |
| `eth-valkyoth-reth` | `0.3.0` | `0.3.0` | `unchanged` | No | Reth boundary remains unchanged for the initial v0.6.0 dependency refresh. |
| `eth-valkyoth-testkit` | `0.3.0` | `0.3.0` | `unchanged` | No | Testkit boundary remains unchanged for the initial v0.6.0 dependency refresh. |
| `eth` | `0.5.0` | `0.6.0` | `code` | Yes | Starts v0.6.0 by exposing the refreshed optional sanitization dependency range. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
