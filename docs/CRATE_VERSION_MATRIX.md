# Crate Version Matrix

Status: `v0.25.0` public RLP derives are implemented and ready for external
pentest.

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

## v0.25.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.16.0` | `0.17.0` | `code` | Yes | Adds public `RlpEncode` and `RlpDecode` traits plus derive error support. |
| `eth-valkyoth-primitives` | `0.10.1` | `0.11.0` | `code` | Yes | Implements the public RLP traits for primitive domain types. |
| `eth-valkyoth-hash` | `0.10.0` | `0.10.1` | `dependency` | Yes | Depends on `eth-valkyoth-primitives 0.11.0`. |
| `eth-valkyoth-protocol` | `0.22.0` | `0.22.1` | `dependency` | Yes | Depends on `eth-valkyoth-codec 0.17.0` and `eth-valkyoth-primitives 0.11.0`. |
| `eth-valkyoth-verify` | `0.14.1` | `0.14.2` | `dependency` | Yes | Depends on the v0.25.0-aligned codec, primitive, hash, and protocol crates. |
| `eth-valkyoth-derive` | `0.16.1` | `0.17.0` | `code` | Yes | Publishes reviewed `RlpEncode` and `RlpDecode` derive macros with trybuild diagnostics. |
| `eth-valkyoth-sanitization` | `0.7.1` | `0.7.2` | `dependency` | Yes | Depends on `eth-valkyoth-derive 0.17.0` for the optional derive feature. |
| `eth-valkyoth-evm` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.25.0. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.25.0. |
| `eth-valkyoth-signer` | `0.7.0` | `0.7.1` | `dependency` | Yes | Depends on `eth-valkyoth-primitives 0.11.0`. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.25.0. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.25.0. |
| `eth` | `0.24.2` | `0.25.0` | `code` | Yes | Re-exports the public RLP trait surface and updates packaged documentation. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
