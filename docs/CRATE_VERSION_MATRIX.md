# Crate Version Matrix

Status: `v0.52.4` adds strict canonical MPT proof preflight and session-aware
inclusion verification. Support-crate minor bumps expose the codec and proof
APIs; dependency-only patches keep the published graph on one codec,
primitive, hash, and protocol type identity.

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

## v0.52.4 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.20.0` | `0.21.0` | `code` | Yes | Adds nibble/value ceilings and noncommitting hash-capacity preflight. |
| `eth-valkyoth-primitives` | `0.11.3` | `0.11.4` | `dependency` | Yes | Accepts codec `0.21` without duplicate public error types. |
| `eth-valkyoth-hash` | `0.11.3` | `0.11.4` | `dependency` | Yes | Accepts primitives `0.11.4`. |
| `eth-valkyoth-protocol` | `0.26.0` | `0.26.1` | `dependency` | Yes | Accepts codec `0.21`, hash `0.11.4`, and primitives `0.11.4`. |
| `eth-valkyoth-verify` | `0.24.0` | `0.25.0` | `code` | Yes | Adds canonical proof preflight, session-aware inclusion APIs, and pre-hash charging. |
| `eth-valkyoth-derive` | `0.17.4` | `0.17.5` | `dependency` | Yes | Accepts codec `0.21` and primitives `0.11.4` in derive verification. |
| `eth-valkyoth-sanitization` | `0.7.6` | `0.7.7` | `dependency` | Yes | Accepts derive `0.17.5`. |
| `eth-valkyoth-evm-core` | `0.26.1` | `0.26.1` | `unchanged` | No | No package changes for v0.52.4. |
| `eth-valkyoth-evm` | `0.10.1` | `0.10.2` | `dependency` | Yes | Accepts the new codec, primitives, and protocol lines. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.4. |
| `eth-valkyoth-signer` | `0.7.4` | `0.7.5` | `dependency` | Yes | Accepts primitives `0.11.4`. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.4. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.4. |
| `eth` | `0.52.3` | `0.52.4` | `code` | Yes | Exposes strict canonical proof preflight and session-aware inclusion verification. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
