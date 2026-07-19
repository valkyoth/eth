# Crate Version Matrix

Status: `v0.52.3` adds an operation-wide, non-copyable decode session across
RLP, every supported transaction family and nested substructure, and MPT
syntax parsing. Support-crate minor bumps expose the public APIs;
dependency-only patches keep the published graph on one codec, primitive, and
protocol type identity.

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

## v0.52.3 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.19.0` | `0.20.0` | `code` | Yes | Public shared decode-session policy, ledger, exact pre-charged nested recounts, and RLP APIs. |
| `eth-valkyoth-primitives` | `0.11.2` | `0.11.3` | `dependency` | Yes | Accepts codec `0.20` without duplicate public error types. |
| `eth-valkyoth-hash` | `0.11.2` | `0.11.3` | `dependency` | Yes | Accepts primitives `0.11.3`. |
| `eth-valkyoth-protocol` | `0.25.2` | `0.26.0` | `code` | Yes | Public shared-session transaction and charged borrowed-traversal APIs. |
| `eth-valkyoth-verify` | `0.23.0` | `0.24.0` | `code` | Yes | Public shared-session MPT syntax and charged inline-node traversal APIs. |
| `eth-valkyoth-derive` | `0.17.3` | `0.17.4` | `dependency` | Yes | Accepts codec `0.20` and primitives `0.11.3`. |
| `eth-valkyoth-sanitization` | `0.7.5` | `0.7.6` | `dependency` | Yes | Accepts derive `0.17.4`. |
| `eth-valkyoth-evm-core` | `0.26.1` | `0.26.1` | `unchanged` | No | No package changes for v0.52.3. |
| `eth-valkyoth-evm` | `0.10.0` | `0.10.1` | `dependency` | Yes | Accepts the new codec, primitives, and protocol lines. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.3. |
| `eth-valkyoth-signer` | `0.7.3` | `0.7.4` | `dependency` | Yes | Accepts primitives `0.11.3`. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.3. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.3. |
| `eth` | `0.52.2` | `0.52.3` | `code` | Yes | Exposes shared-session parsing and clarifies the complete Ethereum-stack goal. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
