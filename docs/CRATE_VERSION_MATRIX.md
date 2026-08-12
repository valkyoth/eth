# Crate Version Matrix

Status: `v0.55.0` tagged and published after its pentest, clean retest, GitHub
CI, and CodeQL gates passed. The EVM core, EVM boundary dependency, and facade
changed.

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package or
published dependency requirements change.

Beginning after the public `v0.55.0` anchor, intermediate tags select no
crates. The `eth` source version follows each tag, support crates retain their
published versions, and all cumulative changes are classified for the next
minor divisible by five.

## Version Rules

| Change kind | Public-checkpoint version rule | Publish? |
| --- | --- | --- |
| `code` | `eth` uses the milestone version; support crates use their next independent minor. | Yes |
| `bugfix` | API-compatible support-crate fixes increment the current patch exactly once. | Yes |
| `dependency` | Patch-bump the existing line. | Yes |
| `metadata` | Use the milestone version when republishing corrected package metadata. | Yes |
| `unchanged` | Keep the previous published version. | No |

At `stage = "internal"`, `eth` follows the tag version without publication;
support crates keep their preceding published versions regardless of their
cumulative change classification. At `stage = "public"`, the table above is
applied once to the complete delta from the preceding public checkpoint.

`dependency` means the crate did not receive meaningful implementation or API
changes, but its manifest must change because a related workspace crate moved
outside the published compatible range.

`bugfix` means implementation changed to correct behavior while preserving the
public API and support-crate type identity. It must not add or remove public
API.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests, verifies the complete tag train, and refuses accidental
lockstep or internal-milestone publication.

## v0.55.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.21.0` | `0.21.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-primitives` | `0.11.4` | `0.11.4` | `unchanged` | No | No package changes. |
| `eth-valkyoth-hash` | `0.11.4` | `0.11.4` | `unchanged` | No | No package changes. |
| `eth-valkyoth-protocol` | `0.26.1` | `0.26.1` | `unchanged` | No | No package changes. |
| `eth-valkyoth-verify` | `0.27.0` | `0.27.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-derive` | `0.18.0` | `0.18.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-sanitization` | `0.8.0` | `0.8.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-evm-core` | `0.28.0` | `0.29.0` | `code` | Yes | Replaces the private 64-byte ModExp ceiling with wide-length gas admission and caller-owned arbitrary-length arithmetic workspace. |
| `eth-valkyoth-evm` | `0.12.1` | `0.12.2` | `dependency` | Yes | Updates the published EVM-core dependency requirement to `0.29.0`. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-signer` | `0.7.5` | `0.7.5` | `unchanged` | No | No package changes. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes. |
| `eth` | `0.54.0` | `0.55.0` | `code` | Yes | Exposes consensus-complete Prague-era EIP-198/EIP-2565 ModExp execution and workspace APIs. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
