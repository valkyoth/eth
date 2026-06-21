# Crate Version Matrix

Status: release matrix for `v0.5.0`

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

## v0.5.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.4.0` | `0.5.0` | `code` | Yes | Adds mandatory proof-node and cumulative item budgets plus checked length and offset helpers. |
| `eth-valkyoth-primitives` | `0.4.0` | `0.5.0` | `code` | Yes | Adds non-exhaustive primitive errors, transaction type TryFrom documentation, and Address hash timing documentation. |
| `eth-valkyoth-protocol` | `0.4.0` | `0.5.0` | `code` | Yes | Marks protocol error enums non-exhaustive and removes the dead-code typestate entry point from normal builds. |
| `eth-valkyoth-verify` | `0.4.0` | `0.5.0` | `code` | Yes | Marks verification error enums non-exhaustive. |
| `eth-valkyoth-derive` | `0.4.0` | `0.5.0` | `code` | Yes | Rejects enum sanitization derives and avoids bounds for skipped generic fields. |
| `eth-valkyoth-sanitization` | `0.4.0` | `0.5.0` | `code` | Yes | Adds hardened-mode evidence and deployment guidance while updating sanitization to 1.1.1. |
| `eth-valkyoth-evm` | `0.3.0` | `0.3.0` | `unchanged` | No | EVM boundary remains unchanged for the v0.5.0 decode-budget milestone. |
| `eth-valkyoth-rpc` | `0.3.0` | `0.3.0` | `unchanged` | No | RPC boundary remains unchanged for the v0.5.0 decode-budget milestone. |
| `eth-valkyoth-signer` | `0.3.1` | `0.3.2` | `dependency` | Yes | Updates the primitives dependency range after eth-valkyoth-primitives moved to 0.5.0. |
| `eth-valkyoth-reth` | `0.3.0` | `0.3.0` | `unchanged` | No | Reth boundary remains unchanged for the v0.5.0 decode-budget milestone. |
| `eth-valkyoth-testkit` | `0.3.0` | `0.3.0` | `unchanged` | No | Testkit boundary remains unchanged for the v0.5.0 decode-budget milestone. |
| `eth` | `0.4.0` | `0.5.0` | `code` | Yes | Re-exports the expanded and remediated v0.5.0 API through the facade crate. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
