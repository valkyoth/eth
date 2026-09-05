# Maintenance Review: 2026-09-05

Scope: maintenance after published `v0.55.0`, before starting `v0.56.0`.

Version assignments below are the source-review snapshot before the subsequent
[scope review](ROADMAP_REVIEW_2026_09_05.md). Consult its
[version map](roadmap-version-map.json) for current planned numbers; source
hashes and completed test evidence in this dated review remain unchanged.
No tag is replaced and no crates are published by this update. The five-minor
publication cadence, exact-commit pentests and signed milestone tags remain.

## Toolchain And Dependencies

The [official stable manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml)
reports Rust `1.98.1`, dated 2026-09-03. The workspace and active gate now use
that compiler; `rust-version = "1.90"` remains unchanged. Historical release
reports and older gate scripts retain their original toolchain evidence.

| Item | Reviewed result |
| --- | --- |
| `sanitization` | `2.0.3` -> `2.0.4`; Rust 1.90 compatible |
| `syn` | `3.0.3` -> `3.0.5` |
| Other direct crates | All 11 others current, including trybuild `1.0.120` and libfuzzer-sys `0.4.13` |
| Workspace transitive updates | cpufeatures `0.3.1`, indexmap `2.14.2`, toml `1.1.5+spec-1.1.0`, wnaf `0.14.1` |
| Fuzz transitive updates | cc `1.4.5`, find-msvc-tools `0.1.12`, hybrid-array `0.4.14`, cpufeatures `0.3.1`, wnaf `0.14.1` |
| CI security tools | cargo-deny `0.20.2`, cargo-audit `0.22.2`, cargo-sbom `0.10.0`: current |
| Fuzz tool | cargo-fuzz `0.13.2`: current |
| Optional local release tool | cargo-release upgraded `1.1.2` -> `1.1.5`; not a replacement for the project's publisher |
| GitHub checkout | `v7.0.1`, SHA `3d3c42e5aac5ba805825da76410c181273ba90b1`: current |
| CodeQL | GitHub Default setup retained; no custom action/workflow introduced |

Registry versions were checked with `scripts/check_latest_crates.py`,
`scripts/check_latest_tools.sh`, and official crates.io metadata. The
[sanitization package](https://crates.io/crates/sanitization/2.0.4) source diff
changes protected mapping replacement/growth to retain established preferred
protections or fail. Its canonical wipe API and our bridge require no migration.
This is a dependency review and integration test, not a new independent audit
of the upstream crate. Core runtime dependency/no_std policies are unchanged.

CI now fetches full history so publication-train validation can inspect tags.
The SHA-pinned checkout action was already current; changing its SHA without
an upstream release would not improve freshness. The existing live Rust/tool
and dependency checks remain mandatory before each release.

Rust 1.98 adds `clippy::chunks_exact_to_as_chunks`. That style preference is
explicitly allowed in the workspace to retain reviewed slice-iterator item
types, including existing typed wrappers; security lints remain unchanged.
The standalone hex-fixture reader and fuzz operation loop use `as_chunks`
where no public iterator contract is affected (available at the MSRV).

## External Execution References

The official release APIs report Geth `v1.17.5`, Besu `26.8.1`, and Nethermind
`1.39.3`. Only Besu changed. Its Docker registry manifest was resolved using
`skopeo inspect` and pinned to:

```text
docker.io/hyperledger/besu@sha256:6f3f21ce533383fcc8db3bce02252b59d5a9e776b72b5a1c8ecd2db011600042
```

The runner still checks the release tag, runtime identity and isolation.
The [previous differential report](differential-test-report.md) records Besu
26.7.1 and is retained as historical evidence, not relabeled as a 26.8.1 run.
The host exposes only the `pids` cgroup controller to rootless Podman. A new
three-client run requires delegated `cpu`, `memory`, and `pids`; this update
does not relax those security checks or claim a successful new container run.

REVM and revm-primitives are now `43.0.0`, requiring Rust `1.91.0`.
The newest MSRV-compatible candidates remain `36.0.0` / `22.1.0`. Neither is
admitted. The [review](revm-dependency-review.md) distinguishes refreshed registry
metadata from the prior dependency-policy rejection and its existing deadline.

## Protocol Sources

These are observed official HEADs, not substituted implementation evidence.
The original five `spec-lock.toml` pins stay fixed until applicable fixtures
are imported and rerun. SSZ has a new separate source-only pin and is now
included in synchronization and live drift monitoring.

| Official repository | Observed revision | Changes since existing pin |
| --- | --- | --- |
| [execution-specs](https://github.com/ethereum/execution-specs/compare/2867859a3c19b925f7dc47dae648cca9758f4f80...903b48f152c932f6e47a615f0f7f009c56f1d92b) | `903b48f152c932f6e47a615f0f7f009c56f1d92b` | 70 commits |
| [tests](https://github.com/ethereum/tests/tree/c67e485ff8b5be9abc8ad15345ec21aa22e290d9) | `c67e485ff8b5be9abc8ad15345ec21aa22e290d9` | unchanged |
| [EIPs](https://github.com/ethereum/EIPs/compare/582684e2d7d372c09f45777be8ea603e485e9e9d...9207c6011f526bd40abd79649484a1a342585bd4) | `9207c6011f526bd40abd79649484a1a342585bd4` | 91 commits |
| [execution-apis](https://github.com/ethereum/execution-apis/compare/742d45db810b31265c8d3c075af324953330d1ed...2ab543851a206ec2836cb387b3aa9cb33c646938) | `2ab543851a206ec2836cb387b3aa9cb33c646938` | 9 commits |
| [consensus-specs](https://github.com/ethereum/consensus-specs/compare/6d0e95d972a90bbf79a356ded6a704d769bb67c0...6805f6a66e831f909f468fa595782e5cc8e5bd9f) | `6805f6a66e831f909f468fa595782e5cc8e5bd9f` | 59 commits |
| [ssz-specs](https://github.com/ethereum/ssz-specs/tree/9ff5170ab7701540b008e12ff3d42fe0c6d35cf9) | `9ff5170ab7701540b008e12ff3d42fe0c6d35cf9` | newly tracked |

The [SSZ move](https://github.com/ethereum/consensus-specs/pull/5523) removes
generic SSZ sources/vectors from consensus-specs. Future vector admission must
follow the new source, rather than silently skipping disappeared fixtures.

Additional official interface sources were reviewed for changes since
2026-08-12, without prematurely adding their future implementations to the
active fixture lock:

| Source / inspected HEAD | Roadmap consequence |
| --- | --- |
| [Beacon APIs](https://github.com/ethereum/beacon-APIs/tree/ef98d512c03c8ca6b9d7cbdc45b9293ec2b24722) | Four commits: Gloas production/forwarding, builder headers and progressive payload-attestation response; `v0.262.0`, `v0.264.0`. |
| [Keymanager APIs](https://github.com/ethereum/keymanager-APIs/tree/d1c9bb46914be4e80f0cd7d5a225695ba94d8751) | Five commits: OpenAPI 3.1, per-key builder configuration/authentication and payment semantics; `v0.275.0`. |
| [Builder specs](https://github.com/ethereum/builder-specs/tree/38f11441c194d150386f567b4d7087ec86d4118c) | Three commits: Gloas gas-limit and bid/payment clarification; `v0.279.0`. |
| [DevP2P](https://github.com/ethereum/devp2p/tree/2c19a28b25c29487773ad6b07243e290fa8d65ec) | Seven commits including post-Merge message deprecation and discv5 challenge/admission changes; `v0.185.0`, `v0.186.0`, `v0.246.0`. |

[Glamsterdam's meta EIP](https://eips.ethereum.org/EIPS/eip-7773) is in Review;
the reviewed activation section does not establish a mainnet activation.
[Hegota's meta EIP](https://eips.ethereum.org/EIPS/eip-8081) is Draft and lists
FOCIL and Frame Transaction as scheduled work. Proposed entries are not
automatically admitted. The
[Plataberget announcement](https://blog.ethereum.org/2026/08/17/plataberget-testnet)
describes testnet integration, not permission to enable those rules on mainnet.

The observed changes require roadmap clarification, not an unreviewed switch
of today's Prague-oriented native execution behavior. The
[release plan](RELEASE_PLAN.md#september-2026-upstream-admission-requirements)
now assigns gas/state-gas, BALs, transfer logs, frame transactions, SSZ,
Engine transport/status, networking, ePBS, FOCIL and consensus upgrades to
explicit existing versions, with required verification and pentest exits.
It also assigns new historical/delegation/signature fixtures to their admission
gates. A later proposed EIP must receive an owner before it is enabled.

## Verification

Passed locally on Rust 1.98.1:

- `scripts/checks.sh`: documentation/roadmap/policy and script regressions,
  SBOM, archive verification, workspace all-feature tests/doctests and Clippy,
  plus fuzz compilation;
- `cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings`;
- `cargo check --workspace --no-default-features --locked`;
- `cargo deny check`, `cargo audit`, and live direct-crate/tool freshness;
- pinned official RLP fixtures and in-process differential oracles;
- `scripts/sync_spec_sources.py --check`, including the newly downloaded SSZ
  source-only checkout, and the live Ethereum/REVM drift report.

`cargo check --workspace --all-features --locked` passed on Rust 1.90.0,
1.91.0, 1.91.1, 1.92.0, 1.93.0, 1.93.1, 1.94.0, 1.94.1, 1.95.0, 1.96.0,
1.96.1, 1.97.0, 1.97.1 and 1.98.0. Both main READMEs remain identical.

The full differential command passed its in-process oracles, then correctly
failed the external-client isolation prerequisite on this host. Therefore the
full pre-tag release gate is **not** declared passed. This maintenance change
is not a release-readiness or pentest attestation and does not reuse the old
report to approve new dependency behavior. The next milestone/checkpoint must
include these changes in its normal review and collect fresh client evidence.
