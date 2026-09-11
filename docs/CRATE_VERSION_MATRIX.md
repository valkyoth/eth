# Crate Version Matrix

Status: `v0.60.0` public-checkpoint candidate; pentest complete, GitHub and release approval pending.
All 14 packages have changes since the published `v0.55.0` baseline.

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

## v0.60.0 Tracking Table

| Crate | Published | Candidate | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.21.0` | `0.21.1` | `bugfix` | Yes | Corrects README examples and test maintenance without API changes. |
| `eth-valkyoth-primitives` | `0.11.4` | `0.11.5` | `dependency` | Yes | Updates codec requirement and current domain examples. |
| `eth-valkyoth-hash` | `0.11.4` | `0.11.5` | `dependency` | Yes | Updates primitives requirement and backend examples. |
| `eth-valkyoth-protocol` | `0.26.1` | `0.26.2` | `dependency` | Yes | Updates internal requirements and current bounded transaction examples. |
| `eth-valkyoth-verify` | `0.27.0` | `0.27.1` | `dependency` | Yes | Updates internal requirements and verification guidance. |
| `eth-valkyoth-derive` | `0.18.0` | `0.18.1` | `dependency` | Yes | Updates macro dependencies and executable derive examples. |
| `eth-valkyoth-sanitization` | `0.8.0` | `0.8.1` | `dependency` | Yes | Updates sanitization dependency and current wipe guidance. |
| `eth-valkyoth-evm-core` | `0.29.0` | `0.30.0` | `code` | Yes | Adds cumulative public-input BLS12-381 Fp/Fp2/G1/G2 arithmetic and charged G1 addition. |
| `eth-valkyoth-evm` | `0.12.2` | `0.13.0` | `code` | Yes | Updates public backend review metadata and native core dependency. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.1` | `bugfix` | Yes | Clarifies trust-policy-only scope and adds a tested example. |
| `eth-valkyoth-signer` | `0.7.5` | `0.7.6` | `dependency` | Yes | Updates primitives requirement and identity-only documentation. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.1` | `bugfix` | Yes | Clarifies placeholder scope and adds a tested boundary example. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.1` | `bugfix` | Yes | Clarifies corpus metadata scope and adds a tested example. |
| `eth` | `0.55.0` | `0.60.0` | `code` | Yes | Publishes the complete v0.56.0 through v0.60.0 train and refreshed documentation. |

Update this table and `release-crates.toml` together. Publication follows the
listed dependency order after per-tag pentest, local verification, GitHub
green, and explicit signed-tag approval. No crates have been published yet.
