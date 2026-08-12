# Crate Version Matrix

Status: `v0.54.0` release candidate; pentest findings are remediated and the
final retest passed. The EVM core, EVM boundary dependency, and facade change.

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package or
published dependency requirements change.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | `eth` uses the milestone version; support crates use their next independent minor. | Yes |
| `bugfix` | API-compatible support-crate fixes increment the current patch exactly once. | Yes |
| `dependency` | Patch-bump the existing line. | Yes |
| `metadata` | Use the milestone version when republishing corrected package metadata. | Yes |
| `unchanged` | Keep the previous published version. | No |

`dependency` means the crate did not receive meaningful implementation or API
changes, but its manifest must change because a related workspace crate moved
outside the published compatible range.

`bugfix` means implementation changed to correct behavior while preserving the
public API and support-crate type identity. It must not add or remove public
API.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests and refuses accidental lockstep publication.

## v0.54.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.21.0` | `0.21.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-primitives` | `0.11.4` | `0.11.4` | `unchanged` | No | No package changes. |
| `eth-valkyoth-hash` | `0.11.4` | `0.11.4` | `unchanged` | No | No package changes. |
| `eth-valkyoth-protocol` | `0.26.1` | `0.26.1` | `unchanged` | No | No package changes. |
| `eth-valkyoth-verify` | `0.27.0` | `0.27.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-derive` | `0.18.0` | `0.18.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-sanitization` | `0.8.0` | `0.8.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-evm-core` | `0.27.0` | `0.28.0` | `code` | Yes | Adds exact-input gas quotes, non-forgeable paid precompile capabilities, canonical descriptor validation, and CALL-ready outcomes. |
| `eth-valkyoth-evm` | `0.12.0` | `0.12.1` | `dependency` | Yes | Updates the published EVM-core dependency requirement to `0.28.0`. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-signer` | `0.7.5` | `0.7.5` | `unchanged` | No | No package changes. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes. |
| `eth` | `0.53.0` | `0.54.0` | `code` | Yes | Exposes the metered exact-input precompile authorization and CALL outcome contract. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
