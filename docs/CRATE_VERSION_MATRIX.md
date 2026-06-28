# Crate Version Matrix

Status: ready for `v0.7.0` external pentest

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

## v0.7.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.6.0` | `0.7.0` | `code` | Yes | Adds bounded canonical RLP list decoding with nesting, item-count enforcement, and immediate child iteration. |
| `eth-valkyoth-primitives` | `0.5.0` | `0.5.0` | `unchanged` | No | Primitive domains remain unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-protocol` | `0.5.0` | `0.5.0` | `unchanged` | No | Protocol state remains unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-verify` | `0.5.0` | `0.5.0` | `unchanged` | No | Verification boundaries remain unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-derive` | `0.6.0` | `0.6.0` | `unchanged` | No | Derive macro support remains unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-sanitization` | `0.6.0` | `0.6.0` | `unchanged` | No | Sanitization support remains unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-evm` | `0.3.0` | `0.3.0` | `unchanged` | No | EVM boundary remains unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-rpc` | `0.6.0` | `0.6.0` | `unchanged` | No | RPC trust-model support remains unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-signer` | `0.3.2` | `0.3.2` | `unchanged` | No | Signer remains on the published v0.3.2 dependency range. |
| `eth-valkyoth-reth` | `0.3.0` | `0.3.0` | `unchanged` | No | Reth boundary remains unchanged for the v0.7.0 RLP list milestone. |
| `eth-valkyoth-testkit` | `0.3.0` | `0.3.0` | `unchanged` | No | Testkit boundary remains unchanged for the v0.7.0 RLP list milestone. |
| `eth` | `0.6.0` | `0.7.0` | `code` | Yes | Exposes bounded canonical RLP list decoding and item iteration through the facade crate. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
