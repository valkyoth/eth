# Crate Version Matrix

Status: `v0.61.0` internal candidate; implementation checks passed; ready for pentest.
Published baseline: `v0.60.0`. No package is selected before `v0.65.0`.

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

`bugfix` covers API-compatible implementation fixes, test corrections, and
README/example corrections. It preserves the public API and support-crate
type identity. It must not add or remove public API. Ordinary documentation
corrections do not trigger the exceptional `metadata` lockstep rule.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests, verifies the complete tag train, and refuses accidental
lockstep or internal-milestone publication.

## v0.61.0 Tracking Table

| Crate | Published | Candidate | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.21.1` | `0.21.1` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-primitives` | `0.11.5` | `0.11.5` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-hash` | `0.11.5` | `0.11.5` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-protocol` | `0.26.2` | `0.26.2` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-verify` | `0.27.1` | `0.27.1` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-derive` | `0.18.1` | `0.18.1` | `dependency` | No | Updates syn and trybuild; cumulative bump at v0.65.0. |
| `eth-valkyoth-sanitization` | `0.8.1` | `0.8.1` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-evm-core` | `0.30.0` | `0.30.0` | `code` | No | Adds paid G2ADD with canonical output and cross-client verification; cumulative bump at v0.65.0. |
| `eth-valkyoth-evm` | `0.13.0` | `0.13.0` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-rpc` | `0.7.1` | `0.7.1` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-signer` | `0.7.6` | `0.7.6` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-reth` | `0.7.1` | `0.7.1` | `unchanged` | No | Retains published version during the internal release train. |
| `eth-valkyoth-testkit` | `0.7.1` | `0.7.1` | `unchanged` | No | Retains published version during the internal release train. |
| `eth` | `0.60.0` | `0.61.0` | `code` | No | Internal milestone for charged BLS12-381 G2 addition; publication at v0.65.0. |

Update this table and `release-crates.toml` together. Source-only changes do not
publish support crates; classification and cumulative bumps occur at v0.65.0.
Each tag still requires its own pentest, GitHub green and explicit approval.
