# Crate Version Matrix

Status: `v0.13.0` implementation, pentest remediation, and clean retest
complete; waiting for final GitHub checks before tag.

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package
contents need a new crates.io version.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | Use the milestone version, for example `0.5.0`. | Yes |
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

## v0.13.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.10.0` | `0.10.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-primitives` | `0.10.0` | `0.10.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-hash` | `0.10.0` | `0.10.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-protocol` | `0.12.0` | `0.13.0` | `code` | Yes | Adds the unvalidated EIP-2930 access-list transaction field decoder. |
| `eth-valkyoth-verify` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-derive` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-sanitization` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-evm` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-signer` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.13.0. |
| `eth` | `0.12.0` | `0.13.0` | `code` | Yes | Updates facade documentation, error re-exports, and dependency ranges for the v0.13.0 access-list transaction decoder. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
