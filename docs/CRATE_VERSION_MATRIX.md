# Crate Version Matrix

Status: active for `v0.4.0` development

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package
contents need a new crates.io version.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | Use the milestone version, for example `0.4.0`. | Yes |
| `dependency` | Patch-bump the existing line, for example `0.3.0` to `0.3.1`. | Yes |
| `unchanged` | Keep the previous published version. | No |

`dependency` means the crate did not receive meaningful implementation or API
changes, but its manifest must change because a related workspace crate changed
in a way that the published dependency range cannot cover.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests before release. The script refuses accidental lockstep
publication when a crate is marked `unchanged`.

## v0.4.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.3.0` | `0.3.0` | `unchanged` | No | No 0.4.0 codec implementation change has landed yet. |
| `eth-valkyoth-primitives` | `0.3.0` | `0.3.0` | `unchanged` | No | No 0.4.0 primitive implementation change has landed yet. |
| `eth-valkyoth-protocol` | `0.3.0` | `0.3.0` | `unchanged` | No | No 0.4.0 protocol implementation change has landed yet. |
| `eth-valkyoth-verify` | `0.3.0` | `0.3.0` | `unchanged` | No | No 0.4.0 verification implementation change has landed yet. |
| `eth-valkyoth-derive` | `0.3.0` | `0.3.0` | `unchanged` | No | Derive macros have no 0.4.0 implementation change yet. |
| `eth-valkyoth-sanitization` | `0.3.0` | `0.3.0` | `unchanged` | No | Sanitization bridge has no 0.4.0 implementation change yet. |
| `eth-valkyoth-evm` | `0.3.0` | `0.3.0` | `unchanged` | No | EVM boundary remains unchanged for the start of 0.4.0. |
| `eth-valkyoth-rpc` | `0.3.0` | `0.3.0` | `unchanged` | No | RPC boundary remains unchanged for the start of 0.4.0. |
| `eth-valkyoth-signer` | `0.3.0` | `0.3.0` | `unchanged` | No | Signer boundary remains unchanged for the start of 0.4.0. |
| `eth-valkyoth-reth` | `0.3.0` | `0.3.0` | `unchanged` | No | Reth boundary remains unchanged for the start of 0.4.0. |
| `eth-valkyoth-testkit` | `0.3.0` | `0.3.0` | `unchanged` | No | Testkit boundary remains unchanged for the start of 0.4.0. |
| `eth` | `0.3.0` | `0.3.0` | `unchanged` | No | The facade will move to 0.4.0 when the 0.4.0 public API changes land. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
