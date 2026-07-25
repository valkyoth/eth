# Crate Version Matrix

Status: `v0.52.5` adds canonical account decoding and cryptographically
composed account/storage proof authority. Only the verify support crate and
facade change; all other published package versions remain unchanged.

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

## v0.52.5 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.21.0` | `0.21.0` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-primitives` | `0.11.4` | `0.11.4` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-hash` | `0.11.4` | `0.11.4` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-protocol` | `0.26.1` | `0.26.1` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-verify` | `0.25.0` | `0.26.0` | `code` | Yes | Adds canonical account decoding, non-forgeable account authority, composed storage verification, and absence/zero semantics. |
| `eth-valkyoth-derive` | `0.17.5` | `0.18.0` | `code` | Yes | Aligns generated sanitization derives with the 2.0 drop-safety contract. |
| `eth-valkyoth-sanitization` | `0.7.7` | `0.8.0` | `code` | Yes | Migrates to `sanitization 2.0.3`, canonical wipe APIs, and runtime protection reports. |
| `eth-valkyoth-evm-core` | `0.26.1` | `0.26.1` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-evm` | `0.10.2` | `0.10.2` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-signer` | `0.7.5` | `0.7.5` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.5. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.5. |
| `eth` | `0.52.4` | `0.52.5` | `code` | Yes | Exposes complete bounded account-to-storage proof composition. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
