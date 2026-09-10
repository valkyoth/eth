# eth Release Plan To 1.0

Status: planning document

This plan is intentionally granular. `eth` is security-sensitive Ethereum
protocol software, so each milestone must be small enough to review, test,
pentest, and stop cleanly before tagging. Every milestone keeps a signed GitHub
tag; crates.io publication is batched at five-minor checkpoints.

The list below is not a maximum. Add patch releases or split a milestone before
implementation if the work no longer fits in one safe review pass.

The [September scope review](ROADMAP_REVIEW_2026_09_05.md) preserves all 296
previously planned workstream contracts, extracts 98 implementation passes and
promotes 11 planned patch milestones to minors. Unpublished work now extends
through `v0.449.0`; published history through `v0.55.0` is unchanged. The
[version map](roadmap-version-map.json) records every previous assignment.
The current candidate is `v0.59.0`, BLS12-381 quadratic-extension arithmetic
on the tagged field foundation. v0.58.0 charged G1 addition is tagged.
Pentest is clean for this new slice; GitHub/tag approval remains pending.
It enables no G2 or other precompile.
The [partial-capability completion map](partial-capability-completion.md)
traces all five yellow README rows to implementation and acceptance releases.

Tags use:

```text
v0.N.0      tagged milestone; public when N is divisible by five
v0.N.P      tagged patch/fix milestone; never advances publication cadence
v1.0.0-rc.N exact 1.0-versioned production candidate
v1.0.0      first serious production-ready Ethereum crate
```

## Tagged Release Trains

`v0.55.0` is the publication-cadence anchor. Beginning with `v0.56.0`, every
minor and patch version completes the existing pentest, report, GitHub, CodeQL,
and signed-tag workflow, but only `v0.60.0`, `v0.65.0`, and later pre-1.0
minors divisible by five publish crates.io packages. See the
[versioning policy](VERSIONING_POLICY.md) and
[release runbook](RELEASE_RUNBOOK.md).

Internal tags receive an incremental pentest against the immediately preceding
tag. Public checkpoints receive a cumulative integration pentest over every
change after the preceding published checkpoint and verify the complete
intervening tag/report chain. Patch tags remain in the same train and do not
move the next checkpoint.

The `eth` source version follows every tag. Supporting crates retain their
latest published versions at internal milestones and receive at most one
cumulative bump per change class at a public checkpoint. No internal milestone
may select a crate for publication, and `scripts/release_crates.py` enforces
that rule.

## Release Principles

### September 2026 Upstream Admission Requirements

The [2026-09-05 source review](maintenance-review-2026-09-05.md) adds the
following mandatory deliverables and verification to the named planned
releases. These are not implemented-feature claims. Each owner must implement
its part, not only a descriptor, and its exit requires the listed evidence plus
its existing exact-commit pentest, remediation and stop below.

The table is an integration-coverage index. The dedicated implementation
passes below own their narrower tasks; an integration owner cannot absorb
additional implementations merely because its row names several EIPs.

`v0.154.0` must recheck scheduled versus proposed status and activation against
pinned official sources. Networking upgrades have separate negotiation; testnet
activation is not mainnet admission. Reconcile this table at `v0.285.0` and the
final `v0.445.0` acceptance matrix. Split an oversized owner into new minor
milestones before implementation; do not hide feature work under patch tags.

| Version owner | Required implementation deliverables | Verification and exit evidence |
| --- | --- | --- |
| `v0.154.0` | Classify EIP-7773 Glamsterdam and EIP-8081 Hegota manifests, execution/consensus names, activations and experimental branches. Track EIP-7610's removal from Glamsterdam. | Pinned scheduled/proposed/removed matrix; no testnet-only rule enabled on mainnet by default. |
| `v0.165.0` | EIP-2780 intrinsic/runtime gas, EIP-7778 pre-refund block accounting, EIP-7976 calldata floor, EIP-7981 access lists, EIP-8037 state-gas reservoirs/rollback/child merges and EIP-8038 state access, alongside assigned Osaka rules. | Fork-edge fixtures and independent clients for halts, reverts, delegation spills, refund placement and funded accounts; no conflation of execution and state gas. |
| `v0.165.0` | EIP-7843 SLOTNUM, EIP-7954 contract/initcode limits, EIP-7997 factory predeploy, EIP-8024 stack opcodes and EIP-8246 SELFDESTRUCT balance behavior. | Opcode/deployment vectors, funded-remnant CREATE2 and historical regressions; only admitted forks change behavior. |
| `v0.116.0`, `v0.140.0`, `v0.165.0` | EIP-7708 transfer-log models, execution emission and receipt/bloom integration, including system calls. | Revert, zero-value, system-address and receipt-gas fixtures; no omitted or duplicated logs. |
| `v0.117.0`, `v0.137.0`, `v0.165.0` | EIP-7928 canonical block access lists, ordering/indexes, reverted reads, recreated accounts, roots and state integration. | Malformed/non-minimal encodings, max nonce and missing/extra account/slot differentials; no promotion based on syntax alone. |
| `v0.129.0`, `v0.166.0` | New execution-specs fixtures for pre-Berlin pricing, EIP-7702 delegation clearing/precompile dispatch, ECRECOVER inverse points, CREATE collisions and receipt accounting. | Every applicable case has a runnable first-party test and independent oracle; out-of-scope cases retain named future owners. |
| `v0.115.0`, `v0.165.0`, `v0.186.0` | EIP-8141 Frame Transaction models, signing/authorization and builders, then execution, approval scopes, rollback, precompile dispatch, floors and state-gas limits. | Official frame-family vectors, negative signature/scope and frame gas tests; decoding alone is not Hegota support. |
| `v0.176.0`, `v0.238.0`, `v0.417.0`, `v0.420.0` | Current execution-apis schemas: BAL getters, hash-based raw receipts, net methods, simulation and admitted REST-SSZ Engine transport. Isolate testing_commitBlockV1. | Wire/error and cross-client server fixtures; production RPC defaults expose no test mutation endpoint. |
| `v0.238.0`, `v0.355.0` | Amsterdam BAL validity, custody columns, SSZ payloads and Bogota inclusion-list sequencing/non-zero response constraints. | Undecodable BAL gets INVALID only from authoritative evidence; timeout/missing-data remain distinct; coordinator sequencing/size tests pass. |
| `v0.235.0`, `v0.308.0` | Standalone ethereum/ssz-specs source/vectors; EIP-7688 progressive structures, roots/proofs and mutable caches required by admitted forks. | Standalone SSZ/consensus differentials, progressive soft-limit/encoding and cache rollback tests; no dependency on deleted generic vector paths. |
| `v0.260.0`, `v0.263.0`, `v0.268.0`, `v0.354.0` | Separately negotiate eth/70 partial receipts (7975), eth/71 BAL (8159), eth/72 sparse blobpool (8070), snap/2 BAL healing (8189), and cell-level column deltas (8136). | Mixed-version peers, custodyColumns null/empty-list cases, malformed deltas and bounded reconstruction; negotiation is not consensus activation. |
| `v0.316.0`, `v0.319.0`, `v0.327.0` | EIP-8045 slashed-proposer exclusion, EIP-8061 churn, Gloas upgrades and EIP-8261 gas-limit schedule configuration. | Official proposer/churn/upgrade and empty-parent gas-limit vectors; historical state unchanged. |
| `v0.325.0`, `v0.334.0`, `v0.363.0`, `v0.382.0` | Full EIP-7732 ePBS and EIP-8282 builder requests: bids, registry/payments, payload timeliness committee, withholding/fallback and fork choice, not just a relay adapter. | Same-key slot reuse, exited builders, parent/hash equality, equivocation, unavailable parents and fork-transition differentials; signing/payment safety survives failure. |
| `v0.322.0`, `v0.334.0`, `v0.345.0`, `v0.346.0`, `v0.363.0` | EIP-7805 FOCIL list formation, gossip/timeliness, dependent-root storage, validation, block-production and fork-choice enforcement. | Reorg, wrong dependent root, withheld/conflicting lists and max-size tests; builders cannot bypass inclusion constraints. |
| `v0.362.0`, `v0.364.0`, `v0.376.0`, `v0.381.0` | Current Beacon/Keymanager/Builder APIs: produceBlockV4, builder preferences/forwarding headers, progressive payload-attestation responses, execution_requests_root, per-key builder configuration, BuilderRequestAuth and bid-payment valuation. | Pinned OpenAPI 3.1 and builder fixtures, gas-limit/payment validation, forwarding authentication, key-isolation and mixed-version tests; no unchecked builder route or payment field. |
| `v0.257.0`, `v0.260.0`, `v0.343.0` | Updated discv5 admission controls and WHOAREYOU challenge retransmission; post-Merge NewBlockHashes/NewBlock deprecation. | Handshake replay/amplification and duplicate-challenge tests, pre/post-Merge message profiles; no legacy announcement treated as PoS fork-choice authority. |
| `v0.285.0`, `v0.305.0`, `v0.327.0` | Reassess experimental EIP-8148 sweep thresholds, EIP-8205 credentials, EIP-8321 RANDAO, proof-engine changes and proposed Hegota additions. | Admitted/rejected/proposed inventory with implementation owners before enabling selected proposals; experimental branches do not imply scheduled inclusion. |

Every owner retains its Goal, Deliverables, Verification, Exit criteria and
pentest below. Final acceptance reconciles this inventory with implementation
and integration reports; source drift never silently enables new rules.

### Scope And Completeness Rules

- The target is both a reusable Rust library family and complete execution,
  beacon, validator and integrated-node products. A shipped executable does
  not prove that downstream developers can build their own client from public
  APIs; independent consumers are mandatory at `v0.442.0..=v0.444.0`.
- Each implementation pass owns one algorithm family, wire contract, state
  machine or integration boundary. New cryptography, a new wire protocol and
  a new persistent schema must not be introduced together. Split again before
  coding when the detailed source inventory crosses these boundaries.
- The `Scope` paragraph limits the release even when the retained workstream
  contract lists more coverage. Completion passes state their remaining
  implementation explicitly. Previously extracted pieces are consumed and
  retested, not implemented again. Ordinary milestones must apply the same
  limit; a short bullet containing many protocol families is not a small task.
- `Depends on` identifies the preceding baseline, not permission to use a
  future implementation. Forward references are requirements or later product
  acceptance unless explicitly fulfilled already. Test doubles can test a
  contract but cannot establish production capability. A concrete dependency
  discovered to be later must be moved or split before starting the consumer.
- Each release begins with a checked-in scope manifest naming exact methods,
  types, fork revisions, crate ownership, public API changes, inputs, outputs,
  errors and excluded adjacent work. Each behavior maps to a named runnable
  fixture/property/differential/negative test and an observable expected result.
  Its release gate command and numeric work/memory limits are recorded before
  the implementation stop, not invented after a pentest has passed.
- Mandatory integration evidence is cumulative: every retained contract bullet
  must point to an implementing pass and its test/report. Missing, skipped or
  mock-only behavior blocks the corresponding support claim and final 1.0 gate.
- Every parser/validator tests malformed inputs and budget exhaustion; every
  mutation tests failure atomicity; every concurrent/persistent component tests
  cancellation, restart and fault recovery; secret-bearing code requires
  independent cryptographic and side-channel evidence. All code stays below
  500 lines per file, in focused crates, with no first-party unsafe code.
- `no_std`/no-alloc core, alloc convenience and std/service adapters are
  separately tested products. Networking, storage engines, runtimes and key
  custody remain opt-in. First-party Ethereum rules remain authoritative;
  reviewed generic OS/transport/crypto adapters cannot decide consensus validity.
- Platform claims distinguish compilation, runtime tests and full-node/duty
  operation. Linux, Windows, BSD, macOS, Android, iOS and applicable WASM/embedded
  profiles are tracked; Aesynx remains a portability target until runnable.
  Platform limits must never silently become stricter Ethereum validity rules.
- Complete Ethereum support is a pinned, executable capability matrix, not a
  promise to implement every proposed EIP or every deployed contract forever.
  Historical and adopted forks, client protocols and standard workflows are
  mandatory. Experimental proposals/custom-chain extensions remain isolated;
  each newly admitted capability receives numbered implementation and evidence
  passes before it is advertised. No selected rule may be deferred without an
  owner version. Upstream review continues through RC and maintenance.

Every release must have:

- a clear definition of done;
- a local verification command;
- security review notes;
- known limitations;
- release notes;
- dependency-policy evidence;
- spec-source evidence for protocol behavior;
- completed pentest evidence for the exact implementation commit being
  reviewed;
- no hidden dependency on one developer machine.

Every release should prefer:

- one protocol boundary at a time;
- fixtures before broad implementation;
- pinned official Ethereum source revisions before consensus-sensitive code;
- first-party implementations for core Ethereum wire formats, state machines,
  validation rules, and execution behavior;
- third-party crates only as reviewed optional backends, references, or
  compatibility adapters unless a cryptographic primitive is explicitly
  accepted with a first-party boundary and replacement/audit plan;
- negative and adversarial tests with each parser;
- explicit fork and chain context over global "latest" behavior;
- optional local Ethereum node smoke fixtures before RPC functionality is
  treated as complete;
- no default networking, signing, Reth, P2P, or local key storage.

## Required Milestone Format

Every release, including historical tagged releases, must remain a standalone
implementation handoff with
these explicit sections:

- `Status`: whether the release is planned, in implementation, awaiting
  pentest, or ready to tag;
- `Patch rationale`: required between `Status` and `Goal` for every planned
  `v0.x.y` milestone where `y != 0`, explaining why the work is
  compatibility-preserving and does not require a new minor release;
- `Goal`: the single outcome the release exists to achieve;
- `Scope`: mandatory for unpublished milestones, stating implementation versus
  completion work, its predecessor and any narrower remaining implementation;
- `Deliverables`: the bounded implementation and documentation work included
  in that version;
- `Verification`: release-specific tests, vectors, differential checks,
  interoperability checks, or operational evidence;
- `Exit criteria`: the observable definition of done, ending with the exact
  release version and `implementation stop reached. Run pentest for this exact
  commit.`

Release-specific verification is additive to the repository-wide release
gates and pentest handoff below. It never replaces `scripts/checks.sh`,
dependency-policy checks, SBOM validation, documentation and release-note
review, exact-commit pentesting, clean retesting, GitHub CI and CodeQL review,
or local release-readiness validation.

Summary tables may be added for navigation, but they must not replace these
per-version sections. Split or add versions whenever a goal, deliverable set,
verification pass, or exit criterion is too broad for one reviewable release.

## Pentest Before Tags

Every version must pass a security review and pentest before it is tagged. This
applies to `v0.N.P` patch tags as well as milestone tags.

From v0.56.0 onward, implementation checks run before the pentest handoff;
final tag admission only validates metadata/report readiness after green
GitHub checks and explicit maintainer authorization. The active versioned
gate uses `--implementation` for the first phase and defaults to `--tag` for
the second. External-client/Podman tests remain separate evidence-producing
commands, never silently weakened or reported passed when unavailable. The
[runbook](RELEASE_RUNBOOK.md#integration-evidence) defines scope and limitation
recording. No new feature may claim conformance without its required evidence.

A version is not tag-ready until:

- `scripts/checks.sh` passes;
- `cargo deny check` passes;
- `cargo audit` passes;
- `scripts/generate-sbom.sh --check` confirms the committed SPDX inventory
  matches the current dependency graph;
- release notes exist at `release-notes/RELEASE_NOTES_X.Y.Z.md`;
- a pentest report exists at `security/pentest/vX.Y.Z.md`;
- the pentest report names the exact full 40-character `Reviewed-Commit:`;
- the pentest report has `Status: PASS`;
- the pentest report has non-blank `Tester:` and `Scope:` fields;
- the pentest report has a `Date: YYYY-MM-DD` field;
- `scripts/validate-release-metadata.sh` derives the current version from
  `release-crates.toml` and checks it matches the `eth` manifest version
  without requiring the still-pending current pentest report, so normal CI can
  pass on implementation and retest commits;
- `scripts/validate-release-readiness.sh vX.Y.Z` requires the matching
  `security/pentest/vX.Y.Z.md` report to have `Status: PASS` and is run by the
  local release gate before tagging and directly by the publisher after
  tagging;
- `sbom/eth.spdx.json` exists, is non-empty, and passes semantic drift
  comparison against a freshly generated document;
- the tag does not already exist locally;
- `scripts/validate-release-readiness.sh vX.Y.Z` passes before the tag is
  created;
- GitHub's release workflow is metadata-only and manually dispatched. Do not
  rely on a tag-push workflow for readiness: after a tag is pushed, the
  readiness script still fails closed when invoked directly because the tag
  already exists. `release_crates.py` supplies a guarded post-tag context only
  after verifying that the expected tag resolves exactly to `HEAD` and has a
  valid cryptographic signature.

`scripts/check_latest_tools.sh` is a mandatory networked release check. Every
version-specific release gate must run it before tagging so stale stable Rust,
cargo tools, or GitHub Action pins fail closed. Normal commit CI may omit the
live query to avoid making every development commit depend on upstream
availability; the release gate may not omit it.

`scripts/check_latest_crates.py` is the matching mandatory direct-dependency
check. It reads every workspace and fuzz manifest, compares exact direct
crates.io versions against the newest stable release compatible with Rust
`1.90`, and includes semver-major updates. Compatibility-only output from
`cargo outdated` is not sufficient for a release decision.

Ethereum upstream monitoring is also a maintenance requirement. When the EVM
or fork-aware protocol surface is active, the planned automation must check the
latest REVM registry line, official Ethereum hardfork/spec sources, and pinned
fixture revisions, then report whether a maintenance release is needed for new
fork rules, opcodes, gas costs, precompiles, transaction types, or test
fixtures. Live upstream checks are advisory inputs; concrete release claims
still depend on pinned revisions in `spec-lock.toml`.

When a version's implementation criteria are done, stop and say:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

No tag is created at that point.

### Pentest Handoff Flow

Use this loop for every version:

1. Implementation reaches the version stop point.
2. Portable implementation checks pass, including the versioned gate's
   `--implementation` phase, `scripts/checks.sh`, Deny and Audit.
3. The maintainer runs pentest and writes temporary findings to root
   `PENTEST.md`.
4. Findings are reviewed and fixed.
5. Documentation, tests, and release notes are updated for the fixes.
6. `PENTEST.md` is removed after findings are handled, before the remediation
   commit is finalized.
7. Local gates are run again.
8. GitHub CI and CodeQL default setup are checked after the fix commit.
9. Keep the permanent `security/pentest/vX.Y.Z.md` report/history current during
   fixes; mark it PASS only when review is clean. If the maintainer reports a
   clean pentest directly, document that result without requiring scratch findings.
10. Commit only the permanent report as the release report commit.
11. GitHub CI and CodeQL default setup are checked on the release report commit.
    Reported GitHub failures return to fixes, regression tests, report updates
    and new commits, followed by another GitHub wait.
12. After the maintainer confirms GitHub green and explicitly authorizes the
    tag, `scripts/validate-release-readiness.sh vX.Y.Z` passes through the
    versioned gate's default/`--tag` phase. Do not rerun implementation or
    environment-dependent client workloads at this point.
13. Tagging and pushing tags happen only when explicitly requested.
14. Internal milestones stop after their signed tag is pushed. Scheduled
    public checkpoints continue to publication only after a cumulative
    integration pentest covering the complete train.
15. Publishing through `release_crates.py` requires the signed release tag to
    resolve exactly to `HEAD`, reruns release-readiness/SBOM validation plus
    `cargo deny` and `cargo audit`, validates the package plan, and retains
    Cargo's package verification. It does not rerun environment-dependent
    integration workloads already required by the pre-tag versioned gate.

Root `PENTEST.md` is temporary scratch input. It must not be committed.
The permanent report is part of the release tag. Because committing the report
changes `HEAD`, the report records `Reviewed-Commit:` rather than claiming to
hash itself. The release-readiness gate requires the tag candidate commit to
have the reviewed commit as its first parent and to change only the permanent
report file.

Prerelease candidates use the same workflow and a semver-prerelease report
path such as `security/pentest/v1.0.0-rc.1.md`. Release tooling must parse
prerelease versions structurally rather than assuming three numeric
components. A final `v1.0.0` same-commit promotion may reuse the exact approved
RC pentest evidence only when the tag target, package archives, checksums,
SBOM, and provenance are unchanged; otherwise a new RC and complete review
cycle are required.

## Crate Versioning And Publish Order

Workspace crates use independent versions from `v0.4.0` development onward.
The facade crate remains `eth`, but support crates are not republished just
because another crate changed.

Track every release in `release-crates.toml` and
`docs/CRATE_VERSION_MATRIX.md`:

- `code`: the crate received meaningful implementation, API, or documentation
  changes and uses the release version, either a milestone such as `0.4.0` or
  a deliberately scoped patch release such as `0.9.1`;
- `dependency`: the crate only needs a manifest update because a related crate
  changed outside its current dependency range, so it receives a patch bump on
  its existing line, for example `0.3.0` to `0.3.1`;
- `metadata`: the crate must be republished with the milestone version to
  correct immutable crates.io package metadata such as the license expression;
- `unchanged`: the crate stays on the previous published version and is not
  published.

`scripts/release_crates.py --check` validates the table against Cargo metadata
and refuses accidental lockstep publication. The script still publishes in
dependency order, but only for crates marked `publish = true`.

## Completeness Review Register

Every planning or pentest pass must check this register for implied work that
has not been assigned to a release. If a row affects the 1.0 execution-layer
scope, it must have a versioned milestone before work continues past the
relevant dependency point.

| Gap | Resolution |
| --- | --- |
| Standard transaction signing digests were implied by sender recovery but not scheduled. | Added `v0.22.0 - Transaction Signing Hashes`. |
| End-to-end decoded transaction signature validation was implied by typestates but not scheduled. | Added `v0.23.0 - Full Transaction Signature Validation`. |
| Set-code typed transactions were listed as missing without a version. | Added `v0.24.0 - Set-Code Transaction Decode`. |
| EIP-7702 set-code signing, authorization signatures, empty-list rejection, and fork/account-state validation were deferred by the syntactic decoder. | Added `v0.24.1 - Set-Code Signing And Authorization Validation` and `v0.24.2 - Set-Code Transaction Validity Gate`. |
| EIP-7702 validity gates did not explicitly schedule consensus state application and delegated-code execution. | Expanded `v0.137.0 - State Transition And Journaling` to require ordered authorization processing, persistent delegation writes, authority nonce/refund accounting, one-hop delegated-code resolution, and official EIP-7702 state-transition evidence. |
| Public RLP derives had only an evaluation/prototype milestone. | Added `v0.25.0 - Public RLP Derives`. |
| Full EIP-712 `encodeType`/`encodeData`/`hashStruct` support was missing from the roadmap. | Added `v0.26.0 - EIP-712 Typed-Data Encoder`. |
| EIP-712 JSON-RPC typed-data parsing was deferred from the no-JSON typed encoder without a visible patch milestone. | Added `v0.26.1 - EIP-712 JSON Typed-Data Parser Boundary`. |
| A first-party optional software Keccak backend was deferred without a versioned admission point. | Added `v0.27.0 - Optional Keccak Backend Admission`. |
| Formal verification evidence was not scheduled. | Added `v0.289.0` through `v0.294.0`, including `v0.291.0` cryptographic arithmetic proofs, for Kani, Miri, sanitizers, protocol/concurrency model checking, side-channel review, and bounded invariant evidence as extra assurance, not replacements for fuzzing, conformance tests, pentest, or audit. |
| ABI encoding, Engine API, SSZ, and DevP2P/RLPx were marked deferred. | Added explicit ABI/contract releases `v0.207.0..=v0.218.0`, consensus/Engine releases `v0.233.0..=v0.251.0`, and networking releases `v0.252.0..=v0.272.0`. |
| ENS and common ERC/application standards were not scheduled. | Added `v0.214.0..=v0.218.0` for common standards, ENS, permit, interface helpers, and contract-tooling fuzz/DX gates. |
| Node-level sync, txpool, mining/validator boundaries, and observability were not scheduled. | Added storage/client releases `v0.219.0..=v0.232.0`, networking/sync releases `v0.252.0..=v0.272.0`, and operational-runtime release `v0.231.0`. |
| REVM dependency admission failed the existing dependency policy. | Added `v0.37.1 - REVM Dependency Recheck` before execution work may continue. |
| Native audited EVM execution was not explicitly versioned; REVM could look like the long-term core. | Added the first-party engine, correctness, resource-boundary, precompile, core-crypto, and shared-capability sequence at `v0.40.0..=v0.98.0`, then complete execution, state transition, conformance, tracing, and simulation at `v0.129.0..=v0.173.0`. |
| Default verification previously depended directly on `k256` and used direct `sha3` test wrappers, which conflicted with the long-term first-party-core goal. | Added `v0.37.2` and `v0.37.3` to audit core dependencies, move cryptographic implementation crates behind explicit boundaries/features, and document any accepted cryptographic backend plan. |
| `subtle`, `alloy-rlp`, dev `serde_json`, optional `serde`/`serde_json`, and optional `sanitization` need explicit long-term dependency classifications before execution grows. | Added `v0.37.4` and `v0.37.5` so constant-time helpers, reference oracles, JSON parser support, and sanitization bridges remain deliberate dependency choices. |
| `v0.45.0` deliberately admits cryptographic precompiles as fail-closed descriptors without concrete execution backends. | Added `v0.46.0` through `v0.52.0` for SHA-256, RIPEMD-160, ECRECOVER, ModExp, BN254, BLAKE2F, KZG/BLS backend planning, conformance vectors, fuzzing, dependency review, and pentest gates before state-test claims depend on them. |
| Native opcodes alone do not make full Ethereum execution support; genesis, full block validity, trie-root construction, state transition integration, blob/KZG validation, EOF, and full execution fixtures were not versioned before RPC work. | Added `v0.129.0..=v0.167.0` for full execution, KZG, EOF, current-fork maintenance, fixtures, differential evidence, and performance gates. |
| The native EVM state-access pass intentionally fails closed for pre-London forks until historical gas/opcode rules are implemented. | Added `v0.43.1 - Native EVM Historical Fork Matrix` and `v0.43.2 - Native EVM Pre-Berlin State Gas Schedules` before calls/create build on state access. |
| Rich protocol values were borrowed views only, leaving no owned SDK model. | Added `v0.110.0..=v0.118.0` for general integer/byte primitives, owned transaction/block/state models, and lossless ref/owned/validated conversions. |
| Protocol typestates did not carry transaction payloads or evidence. | Added `v0.121.0 - Payload-Bound Transaction Typestates`. |
| Protocol and native EVM crates exposed disconnected address, word, gas, state, and result domains. | Added `v0.123.0` and `v0.124.0` for shared execution domains and native-core integration before state transition. |
| Fork selection relied on fragmented enums and ordinal chronology. | Added `v0.122.0 - Fork Rules And Chain Specification 2.0` with identity, activation, capability, and parameter separation. |
| Provider transports and end-to-end transaction workflows were not concretely planned. | Added `v0.174.0..=v0.193.0` for typed RPC methods, HTTP/WS/IPC/EIP-1193 transports, provider layers, transaction builders/fillers, simulation, signing, broadcasting, watching, replacement, and live-node tests. |
| Wallet, key-management, contract-signature, multisig, and account-abstraction ecosystems were missing. | Added `v0.194.0..=v0.206.0` for local/remote/hardware signers, keystores, HD wallets, ERC-1271, Safe, ERC-4337, paymasters, session keys, and EIP-7702 delegated workflows. |
| Database, canonical-chain, fork-choice, crash consistency, pruning, history expiry, and runtime supervision were not planned concretely. | Added `v0.219.0..=v0.232.0` for persistent stores, atomic batches, migrations, snapshots, pruning/archive modes, canonical reorgs, head tracking, invalidation, supervision, and performance gates. |
| Consensus light-client work lacked bootstrap, weak subjectivity, aggregate signatures, committee rotation, scoring, persistence, and execution-proof binding. | Added `v0.242.0..=v0.249.0` for a complete light-client security model and official end-to-end vectors. |
| Peer management, request scheduling, bans, bounded multi-peer sync, and historical-data acquisition were absent. | Added `v0.264.0..=v0.272.0` for peer services, request schedulers, txpool, sync, Portal/history acquisition, and builder/validator boundaries. |
| EVM tracing, state overrides, call traces, state diffs, and debug/trace models were missing. | Added `v0.171.0..=v0.173.0` for inspector hooks, trace models, deterministic simulation, and RPC trace interoperability. |
| Witnesses, stateless execution, commitment-scheme agility, Verkle/binary trees, and state/history evolution were not versioned. | Added `v0.273.0..=v0.285.0` for proof-format abstraction, witnesses, stateless execution, future commitments, state-expiry policy, zk-proof boundaries, and fork-maintenance automation. |
| SDK compatibility and documentation drift were not release-blocking. | Added `v0.126.0`, `v0.295.0`, and `v0.296.0` for feature truthfulness, generated dependency snippets, semver/feature/serde compatibility gates, and task-oriented documentation. |
| Consensus types, Engine boundaries, and a light client did not amount to a full beacon node. | Added `v0.304.0..=v0.364.0` for consensus architecture, complete transition and fork choice, storage, networking, sync, Engine coordination, PeerDAS, historical deposits/genesis, beacon orchestration, block production, and server/validator APIs. |
| PeerDAS state, storage, networking, and sync consumers were scheduled before the cell/KZG/reconstruction core. | Moved the first-party PeerDAS core to `v0.315.0`; all DA consumers now follow it, and `v0.397.0` audits the implementation and acceleration boundaries. |
| Historical deposit-contract tracking, deposit trees, eth1 voting, and genesis construction were missing. | Added `v0.357.0` and `v0.358.0` before beacon-node orchestration. |
| Block-production ownership was split ambiguously between beacon and validator clients. | Added beacon-owned, embeddable unsigned production at `v0.363.0`; `v0.364.0` exposes it, and `v0.371.0` limits the validator client to independent checks, slashing authorization, signing, and publication. |
| Live validator duties preceded slashing protection and the signer. | Reordered `v0.365.0..=v0.374.0` so the slashing kernel, transactional database, EIP-3076, key foundation, and signer all precede duty scheduling and every signature-producing duty. |
| Validator key generation and deposit artifacts lacked EIP-2333/EIP-2334 and withdrawal-key separation. | Added `v0.368.0` for key derivation, strict key roles, offline withdrawal credentials, and deposit-data generation/verification. |
| Keymanager, remote signing, and HSM/hardware custody were conflated. | Split operator Keymanager control, outbound remote signing/slashing authority, and signer-to-HSM/KMS custody into `v0.375.0..=v0.378.0`; added threshold/DVT coordination at `v0.380.0`. |
| Builder relay integration and safe local-builder fallback were only a boundary decision. | Added `v0.381.0` and `v0.382.0` for Builder API workflows, relay multiplexing, bid/reveal validation, local fallback, withholding defenses, and protocol-native PBS evolution. |
| Optional network slashing detection, distributed signing, validator analytics, and connectivity diagnostics were absent. | Added `v0.380.0`, `v0.383.0`, `v0.384.0`, and `v0.385.0` with explicit trust and resource boundaries. |
| Production beacon-node and validator-client executables, packaging, data directories, signals, exit codes, upgrades, and rollback were not explicit. | Added separate binary and packaging milestones at `v0.386.0` and `v0.387.0`. |
| A Lighthouse/Prysm-class claim lacked deterministic simulation, mandatory Hive suites, broad client matrices, and quantitative long-testnet/performance gates. | Added `v0.389.0..=v0.394.0`, including the numeric acceptance contract at `v0.390.0`. |
| Later SSZ, BLS, PeerDAS, erasure-coding, and acceleration implementations were not covered by an implementation-level core audit. | Added `v0.397.0` and expanded the integration audits at `v0.398.0..=v0.401.0`. |
| The final unchanged-candidate claim ignored manifest, lockfile, SBOM, and checksum changes required by a `1.0.0` version promotion. | Added an RC-aware tooling foundation at `v0.404.0..=v0.405.0`, then assigned the final rehearsal, admission, and explicit `v1.0.0-rc.1` exact-candidate flow to `v0.447.0..=v0.449.0`; the stable tag must point to the unchanged approved RC commit. |
| Builder and relay ownership remained ambiguous after local block production moved into the beacon node. | `v0.363.0` now exposes only a fail-closed blinded-production hook before builder admission; `v0.381.0` and `v0.382.0` place relay communication in the beacon node while the validator client submits preferences and independently validates and signs blinded blocks. |
| Proposer planning incorrectly implied separate sidecar signatures. | `v0.371.0` now requires one beacon-block proposer signature and constructs sidecars carrying the corresponding signed block header, matching pinned Deneb/Fulu honest-validator rules. |
| RC prose did not yet define package-version/tag mismatch handling, prerelease paths, repeated candidates, gate naming, or exact archive publication. | `v0.404.0..=v0.405.0`, `v0.447.0..=v0.449.0`, and `v1.0.0-rc.1` now require distinct package and candidate identifiers, prerelease-aware tooling/tests, RC-specific gates, exact approved archive upload, and repeated `rc.N` handling. |
| The RC could be read as forcing every independently versioned support crate to `1.0.0`. | `v0.403.0` records the preliminary crate classification and `v0.446.0` makes the binding promotion decision; only `eth` and deliberately approved products are promoted, while support crates keep independent versions and exact reviewed dependency bindings where needed. |
| Companion status/spec documents retained obsolete consensus ranges. | The planning pass updates `docs/current-status.md` and `docs/SPEC_MATRIX.md` through `v0.449.0` and `v1.0.0-rc.N`. |
| Quantitative gates used undefined "client-attributable" failure labels. | `v0.390.0` now requires a machine-enforced attribution taxonomy, predeclared fault windows, conservative ambiguous-event handling, independent approval, and fallback-specific responsibility rules. |
| The final API freeze and production release candidate occurred before full consensus-client abstractions existed. | Reclassified `v0.297.0..=v0.303.0` as foundation stabilization and `v0.403.0..=v0.406.0` as an interim foundation/consensus baseline; the complete freeze, rehearsal, promotion, and candidate admission now occur at `v0.442.0..=v0.449.0` plus `v1.0.0-rc.1`. |
| Core transaction, EVM, and execution-network paths still depended on optional Keccak-256 and secp256k1 implementations without a first-party replacement milestone. | Moved first-party Keccak-256, secp256k1 arithmetic, ECDSA/recovery/ECDH, symmetric transport/keystore primitives, integration, and initial audit forward to `v0.74.0..=v0.87.0`; `v0.407.0..=v0.410.0` now revalidate the completed full-stack consumer set instead of introducing core crypto late. |
| Historical execution support did not explicitly implement Ethash seal verification, historical difficulty, ommers, rewards, and irregular pre-Merge transitions. | Added `v0.411.0..=v0.413.0` for Ethash, complete pre-Merge consensus rules, and a genesis-to-Merge historical execution gate. |
| The roadmap described execution libraries and runtime traits but did not yet produce a complete independently runnable execution client. | Added `v0.414.0..=v0.427.0` for a reviewed production database backend, staged sync and healing, local payload building, authenticated Engine server, inbound execution RPC/GraphQL surfaces, operational discovery, an execution-node binary, recovery tools, and production controls. |
| Execution assurance focused on fixtures and the beacon node consuming other execution clients, not on independent consensus clients driving the first-party execution client. | Added `v0.428.0..=v0.433.0` for execution Hive/RPC compatibility, multi-consensus-client Engine interoperability, public-network sync/follow evidence, performance, independent audit, and remediation. |
| Execution and consensus networking milestones could be read as delegating Ethereum semantics to generic networking crates. | Tightened `v0.252.0..=v0.265.0` and `v0.342.0..=v0.348.0`: generic socket, runtime, and reviewed cryptographic adapters may remain optional infrastructure, but Ethereum codecs, validation, fork compatibility, scoring, resource policy, and protocol state machines are first-party. |
| Separate execution, beacon, and validator binaries did not yet provide an integrated node product, reproducible devnet, or final mixed-client system evidence. | Added `v0.434.0..=v0.441.0` for integrated node orchestration, private-network tooling, mixed-client matrices, long-running integrated tests, performance/recovery, operator guides, full-stack audit, and remediation. |
| The 1.0 admission gate occurred before the newly identified execution-client and integrated-node product work. | Moved final quantitative admission, API/crate freeze, exact release rehearsal, version promotion, and candidate admission to `v0.442.0..=v0.449.0` plus `v1.0.0-rc.N`. |
| Truncated `PUSHn` bytecode was rejected instead of zero-padded, causing consensus divergence. | Added `v0.52.2 - Truncated PUSH Consensus Correction` before broader native execution. |
| Decode budgets reset across nested RLP iterators, transaction stages, proofs, and ownership conversion. | Added `v0.52.3 - Shared Decode Session And Work Ledger` with one non-copyable operation-wide ledger and complexity oracles. |
| MPT proof nodes were hashed before proof-size accounting and locally non-canonical trie forms remained admissible. | Added `v0.52.4 - MPT Proof Preflight And Strict Canonicality`. |
| Account and storage proofs were independently rooted and therefore not cryptographically composed. | Added `v0.52.5 - Composed Account And Storage Proofs` with a non-forgeable `VerifiedAccount` capability. |
| Live proof consumers lacked hash-keyed resolution, multiproof deduplication, immutable snapshot binding, and bounded orchestration. | Added `v0.52.6 - MPT Resolver Multiproof And Snapshot Orchestration`. |
| Opaque classified EIP-2718 envelopes could be confused with executable transactions, and EVM host powers were monolithic. | Added `v0.52.7 - Execution Admission And Host Capability Split`. |
| Fixed linear warm-access arrays become quadratic under adversarial node-scale workloads. | Added `v0.53.0 - Bounded Access Tracking And Execution Governor`, retaining the fixed-array implementation only as an explicit embedded profile. |
| Precompile plans lacked non-forgeable charged authorization, precise CALL outcomes, and gas-derived work/output contracts. | Added `v0.54.0 - Metered Precompile Outcome Contract`. |
| ModExp rejected protocol-valid operands above 64 bytes and a fixed global input ceiling could become a private consensus rule. | Added `v0.55.0 - Consensus-Complete ModExp`; advanced BLS milestones moved to `v0.56.0..=v0.70.0`. |
| Architecture, shared resource governance, and cryptographic backend contracts needed to stabilize before broad SDK and node work. | Added `v0.71.0..=v0.73.0` for dependency/capability invariants, hierarchical budgets, and cryptographic substrate contracts. |
| RPC methods, provider trust, multi-call anchors, transaction lifecycle, and signer boundaries needed stronger type-directed requirements. | Expanded `v0.176.0`, `v0.177.0`, `v0.183.0`, `v0.185.0`, `v0.189.0`, `v0.194.0`, and `v0.195.0`. |
| Durable storage behavior and fault recovery were scheduled too late to validate abstractions before synchronization. | Added `v0.220.0 - Production Storage Pilot` and `v0.226.0 - Persistent Fault And Recovery Gate`. |
| End-to-end Engine integration first appeared too late in the roadmap. | Added `v0.240.0 - Early Engine Vertical Devnet`, an expanding in-process/authenticated adapter-equivalence lane reused by later milestones. |
| Execution and consensus networking risked sharing peer semantics, while txpool and sync requirements lacked several adversarial policies. | Strengthened `v0.252.0..=v0.270.0` with separate protocol planes, hierarchical resource capabilities, policy/consensus separation, EIP-7702/blob/reorg behavior, backpressured stages, and snapshot-bound composed proofs. |
| Fuzzing lacked deep valid structures and work oracles; protocol/concurrency and side-channel assurance were incomplete. | Added `v0.288.0`, `v0.293.0`, and `v0.294.0` for structure-aware continuous fuzzing, complexity oracles, mutation/regression policy, TLA+/Quint/Apalache and Loom models, secret-path testing, and public gas-to-cycles evidence. |
| Shared decode accounting covered RLP/MPT but not SSZ, JSON, ABI, Snappy, SSZ-Snappy, Req/Resp, or GossipSub. | Added `v0.88.0 - Cross-Format Decode Work Accounting` and bound the concrete format milestones at `v0.176.0`, `v0.208.0..=v0.209.0`, `v0.235.0`, `v0.252.0`, `v0.263.0`, and `v0.344.0..=v0.347.0` to its parent ledger. |
| Core cryptographic implementations were scheduled after wallets and networking, and AES-CTR/HMAC/ECIES/KDF boundaries were unnamed. | Added `v0.74.0..=v0.87.0` before Phase 9; local signing and RLPx now consume already admitted first-party or explicitly audited primitives. |
| Current-fork system operations and execution requests were hidden inside broad transition milestones. | Added `v0.139.0 - System Operations And Execution Requests` for EIP-4788, EIP-2935, EIP-6110, EIP-7002, EIP-7251, EIP-7685, ordering, rollback, persistence, Engine encoding, and header binding. |
| Resource cancellation could conflate consumed work, reservations, and replenishing rate limits. | Expanded `v0.72.0` with distinct non-refundable work, releasable reservations, policy-replenished rates, cross-thread conservation, double-release prevention, and deterministic consensus work units. |
| Node-scale execution lacked snapshot-bound cache/prefetch and deterministic speculative parallelism milestones. | Added `v0.168.0 - Snapshot-Bound Execution Caches And Prefetch` and `v0.170.0 - Deterministic Speculative Parallel Execution`. |
| KZG/BLS batch verification did not state coefficient, transcript, entropy, cache, isolation, or latency soundness. | Expanded `v0.148.0`, `v0.311.0`, and `v0.315.0` with fail-closed context-complete batch contracts. |
| Differential execution did not enumerate all compared consensus outputs or require sufficiently diverse clients. | Expanded `v0.167.0` to compare status, halts, gas/refunds, logs/bloom, receipts and all roots, account/code changes, diffs, and traces against official fixtures plus Geth, Besu, and Nethermind. |
| Engine, scheduling, backpressure, and txpool models were planned only after implementations. | Added design-time executable models to `v0.72.0`, `v0.237.0..=v0.239.0`, `v0.252.0..=v0.265.0`, and `v0.268.0`, while retaining `v0.293.0` as the integrated formal gate. |
| Security-relevant caches lacked one global identity and validation-level invariant. | Expanded `v0.71.0`, `v0.168.0`, and `v0.346.0` with chain/genesis, fork, snapshot/root, object, and validation-domain identity plus atomic reorg/fork invalidation. |
| Snap was planned mainly as a consumer rather than a bounded snapshot-pinned serving protocol. | Expanded `v0.263.0` with range/proof construction, serving budgets, fairness, snapshot cancellation, and mixed-snapshot rejection. |
| ModExp length handling and shared time semantics needed explicit wide-integer and clock-domain contracts. | Expanded `v0.55.0` with 256-bit length preservation and virtual padding; added `v0.89.0 - Shared Clock And Time-Evidence Contract`. |
| Local resource, timeout, dependency, storage, or backend failures could be confused with consensus invalidity. | Added `v0.93.0 - Validation Outcomes, Object Invalidity And Peer Evidence` and required its non-forgeable evidence in block import, Engine, sync, txpool, peer scoring, gossip, and negative caches. |
| Expected policy refusal, unsupported capabilities, duplicate objects, and deferred processing could be misclassified as either local failure or protocol invalidity. | Expanded `v0.93.0` with explicit non-action outcomes: they cannot create object-invalidity evidence, while separately evidenced repeated quota abuse may create peer-policy evidence without poisoning the object. |
| Object invalidity and peer protocol/policy violations were conflated, contradicting later peer-scoring requirements. | Expanded `v0.93.0`, `v0.264.0..=v0.269.0`, and `v0.346.0..=v0.348.0` with disjoint evidence types, identities, consumers, and cache authority. |
| A failed cryptographic batch could incorrectly identify every member or every contributing peer as invalid. | Added the non-attributable `BatchContainsInvalid` outcome to `v0.93.0` and expanded `v0.148.0`, `v0.311.0`, and `v0.315.0` with bounded individual isolation before member-specific invalidity or punishment. |
| Invalidity evidence did not explicitly bind auxiliary objects such as blob sidecars, PeerDAS data, KZG domains, Engine bundles, and Snap ranges. | Expanded `v0.93.0` with auxiliary-object identities and substitution tests so one invalid component cannot poison a valid sibling or containing block. |
| Deterministic ECDSA was ordered before its RFC 6979 HMAC-SHA256 dependency and lacked complete nonce/fault requirements. | Expanded and renamed `v0.81.0 - First-Party HMAC ECDSA Recovery And ECDH`; `v0.86.0` now consumes its admitted HMAC implementation. |
| First-party secp256k1 arithmetic proofs were scheduled after local signer and network-identity consumers. | Added `v0.94.0 - Early secp256k1 Arithmetic Proof Gate` before those consumers; `v0.291.0` retains the broader cross-primitive consolidation and extension gate. |
| Generic signer and key abstractions could permit secp256k1 execution or transport material to cross into BLS consensus duties. | Added `v0.95.0 - Signing And Transport Capability Separation` and expanded `v0.194.0`, `v0.195.0`, `v0.257.0`, `v0.368.0`, and `v0.369.0` with sealed scheme-specific signing capabilities, a separate non-signing transport identity capability, opaque tagged custody types, withdrawal-key separation, and compile-fail cross-capability tests. |
| Deployment resource policy could accidentally narrow protocol-valid object limits and create local consensus divergence. | Added `v0.98.0 - Contextual Protocol, Wire And Operational Limit Domains` and expanded `v0.122.0`, `v0.231.0`, networking consumers, and consensus Req/Resp with immutable per-object contexts, advertised static envelopes, dynamic candidate-derived work, protocol-versioned wire limits, readiness withdrawal, and local-only exhaustion outcomes. |
| An immutable validation context could still be forged if callers could construct, deserialize, or substitute its fork and limit fields. | Expanded `v0.98.0`, `v0.121.0`, `v0.122.0`, `v0.134.0`, and `v0.225.0` with private context fields, sealed rules-engine constructors, verified parent/genesis authority, rules/limits digests, derived child contexts, and corrupted-storage tests. |
| Invalidity and peer evidence could itself amplify memory, persistence, serialization, logging, or diagnostic output. | Expanded `v0.93.0`, `v0.225.0`, `v0.228.0`, `v0.264.0`, `v0.346.0`, `v0.385.0`, and `v0.390.0` with evidence budgets, compact witnesses, bounded counters/windows, retention, redaction, and fail-local construction semantics. |
| Proof limits did not distinguish consensus-embedded proofs from Snap/MPT wire proofs and RPC/provider policy. | Expanded `v0.98.0`, `v0.176.0`, `v0.185.0`, `v0.263.0`, and `v0.273.0` with authority-tagged proof limits and cross-domain non-substitution tests. |
| Peer-observation windows lacked restart/session and rollback-safe time semantics. | Expanded `v0.89.0`, `v0.93.0`, `v0.264.0`, and `v0.348.0` with monotonic in-session windows, boot/session identity, carefully defined UTC persistence, and rollback/stale-source expiry tests. |
| Evidence-budget exhaustion after proving invalidity could erase the authoritative result. | Expanded `v0.93.0`, `v0.133.0`, `v0.134.0`, `v0.228.0`, `v0.238.0`, and `v0.346.0` with pre-validation fixed `EvidenceSlot` reservation, infallible allocation-free minimal evidence, and optional cache/diagnostic/persistence attachments that cannot change the immediate result. |
| Nested and batch validators could independently reserve, reset, duplicate, or lose evidence capacity. | Added `v0.100.0 - Hierarchical Evidence Capability Composition` and expanded `v0.133.0`, `v0.134.0`, `v0.135.0..=v0.139.0`, `v0.148.0`, `v0.225.0`, `v0.228.0`, `v0.289.0`, `v0.293.0`, `v0.311.0`, `v0.315.0`, and `v0.346.0` with linear parent/child reservations, bounded batch cardinality, exactly-once lifecycle rules, committed-record recovery, Kani conservation proofs, and Loom concurrency checks. |
| Evidence collection cardinality and optional sink access could be left implicit, allowing diagnostics to affect validity or consume slot authority. | Added `v0.102.0 - Evidence Collection Modes And Immutable Sink Access` and expanded `v0.98.0`, `v0.133.0`, `v0.134.0`, `v0.148.0`, `v0.225.0`, `v0.228.0`, `v0.238.0`, `v0.289.0`, `v0.311.0`, `v0.315.0`, and `v0.346.0` with explicit `FirstInvalid`, `CollectUpTo<N>`, and `BatchIsolateUpTo<N>` operational modes, validity invariance, immutable evidence borrowing, and one final slot-ownership transition. |
| Evidence safety machinery could impose valid-path allocation, contention, hashing, sink work, code-size growth, or public API complexity. | Added `v0.104.0 - Evidence Hot-Path And API Containment Gate` and expanded `v0.71.0`, `v0.99.0..=v0.102.0`, `v0.133.0`, `v0.134.0`, `v0.148.0`, `v0.167.0`, `v0.232.0`, `v0.287.0`, `v0.295.0`, `v0.311.0`, `v0.315.0`, `v0.346.0`, `v0.390.0`, and `v0.394.0` with allocation-free uncontended valid paths, parent arenas/index handles, amortized reservation, admitted cardinalities, internal machinery, stable non-generic outcomes, evidence-disabled internal baselines, and release-blocking overhead/size thresholds. |
| Evidence arenas lacked an explicit capacity, reuse, transfer, and structurally equivalent benchmark-baseline contract. | Added `v0.106.0 - Evidence Arena Capacity And Benchmark Integrity` and expanded `v0.72.0`, `v0.99.0..=v0.104.0`, `v0.134.0`, `v0.137.0`, `v0.170.0`, `v0.231.0`, `v0.286.0`, `v0.287.0`, `v0.289.0`, `v0.293.0`, `v0.295.0`, and `v0.346.0` with capability-backed simultaneous-work sizing, local backpressure/exhaustion, ABA-safe handles, audited stack placement, reference-safe transfer/cancellation, optional benchmark-justified pools, non-generic public mode dispatch, and optimizer-resistant equivalent baselines. |
| Invalid-path baselines, benchmark timing/instrumentation, runtime cardinality-class mapping, and generation-wrap behavior remained ambiguous. | Added `v0.109.0 - Evidence Benchmark Measurement And Dispatch Closure` and expanded `v0.101.0..=v0.106.0`, `v0.287.0`, `v0.289.0`, `v0.295.0`, `v0.390.0`, and `v0.394.0` with valid-only disabled baselines, invalid semantic projections/minimal-evidence baselines, protocol-versus-evidence counters, lifecycle-complete timing with untimed setup/result work, uninstrumented production thresholds, separate non-perturbing conformance instrumentation, exact requested-limit enforcement over upward internal capacity classes, physical-class resource charging, and fail-closed generation retirement. |
| Validation contexts could remain non-forgeable yet accidentally retain recursive ancestry or unstable process-local identities. | Expanded `v0.98.0`, `v0.122.0`, `v0.134.0`, `v0.137.0`, `v0.168.0`, and `v0.225.0` with bounded parent handles, borrowed child contexts, deterministic lease release, canonical versioned encoding, domain-separated cryptographic digests, and constant-size/stability tests. |
| First-party cryptographic arithmetic needed explicit machine-checked implementation evidence beyond the early secp gate. | Added `v0.291.0 - Kani Cryptographic Arithmetic Proofs` for limbs, reduction, conversion, inversion, square roots, point exceptions, scalar multiplication, and canonical serialization across the broader cryptographic core. |
| Provider JSON-RPC, HTTP, WebSocket, and IPC boundaries lacked several canonicality, redirect, rebinding, proxy, credential, and local-peer controls. | Expanded `v0.176.0` and `v0.178.0..=v0.182.0` with canonical quantity/bytes/ID rules, decoded-byte charging, redirect/origin/DNS/proxy/credential policy, Unix ownership/symlink checks, and Windows pipe ACL/identity checks. |
| Txpool entries were not explicitly revalidated across heads, forks, fees, restarts, account/delegation state, and blob-sidecar lifecycle. | Expanded `v0.268.0`; persisted/local/protected status never bypasses fresh consensus validation. |
| Negative and bad-block caches needed stricter evidence, identity, invalidation, retention, and anti-flood rules. | Expanded `v0.93.0`, `v0.228.0..=v0.229.0`, `v0.269.0`, and `v0.346.0`; only `ObjectInvalidityEvidence` may enter object-negative caches, while peer evidence remains separate. |
| Speculative parallel execution did not enumerate all implicit transaction and block dependencies. | Expanded `v0.170.0` with nonce/balance, coinbase, creation/code, SELFDESTRUCT, EIP-7702, transient/original/warm state, system request, precompile environment, receipt/log/gas/order dependencies, and sequential fee-delta commit. |

## Phase 0: Repository And Release Discipline

### v0.1.0 - Repository Foundation

Status: tagged as `v0.1.0`.

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.98.1` pinned.
- Rust `1.90.0` through `1.98.1` compatibility policy.
- Focused no_std workspace crates.
- CI, dependency policy, security policy, release notes.
- Implementation, release, scope, threat-model, modularity, toolchain,
  unsafe, spec, and supply-chain docs.

Verification:

- `scripts/checks.sh`
- `scripts/check_latest_tools.sh`
- `scripts/release_0_1_gate.sh`

Exit criteria:

- A new contributor can understand the scope, security posture, and release
  process from the repository docs.
- `v0.1.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.2.0 - Release Readiness Gate

Status: tagged as `v0.2.0`.

Goal: make the pentest-before-tag process and crates.io publish order
enforceable by local tooling.

Deliverables:

- `scripts/validate-release-readiness.sh`;
- `scripts/release_crates.py`;
- support crates renamed into the `eth-valkyoth-*` crates.io namespace while
  `eth` remains the facade crate;
- release-note metadata checks;
- permanent pentest-report metadata checks;
- SBOM presence checks;
- spec-source policy document;
- tag-exists guard.

Verification:

- `scripts/checks.sh`
- `scripts/release_0_2_gate.sh`
- `scripts/release_crates.py --check`
- `scripts/test-release-readiness.sh`
- `cargo deny check`
- `cargo audit`

Exit criteria:

- The project can refuse a tag-ready claim when pentest or release evidence is
  missing.
- Future protocol milestones have an explicit source-check workflow.
- `v0.2.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 1: Primitive And Error Foundation

### v0.3.0 - Domain Newtypes

Status: tagged as `v0.3.0`.

Goal: make Ethereum numeric and byte domains explicit.

Deliverables:

- chain, block, gas, nonce, timestamp, address, hash, wei, and transaction type
  primitives;
- bounded constructors where values have protocol limits;
- optional sanitization and derive support crate boundaries outside default
  features;
- tests for all constructors and conversions.

Verification:

- `scripts/checks.sh`
- `scripts/release_0_3_gate.sh`
- `cargo deny check`
- `cargo audit`

Exit criteria:

- Public APIs no longer use unqualified integers for core protocol concepts.
- `v0.3.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.4.0 - Stable Error Model

Status: tagged as `v0.4.0`.

Goal: establish non-panicking error categories for protocol operations.

Deliverables:

- codec, protocol, verification, feature, fork, and resource-exhaustion errors;
- no secret-bearing error payloads;
- tests for error stability and formatting;
- independent support-crate release planning through `release-crates.toml` and
  `docs/CRATE_VERSION_MATRIX.md`;
- release tooling that publishes only changed crates while preserving crates.io
  dependency order.

Verification:

- `scripts/checks.sh`
- `scripts/release_0_4_gate.sh`
- `scripts/release_crates.py --check`

Exit criteria:

- Malformed input and unsupported protocol data return errors, not panics.
- Unchanged support crates are not republished for the `0.4.0` release.
- `v0.4.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.5.0 - Decode Budget Model

Status: tagged as `v0.5.0`.

Goal: make resource limits mandatory for untrusted bytes.

Deliverables:

- byte, list, nesting, allocation, proof-node, and item-count limits;
- checked arithmetic helpers for lengths and offsets;
- adversarial tests for budget rejection.

Verification:

- `cargo test -p eth-valkyoth-codec`

Exit criteria:

- No decoder entry point can be designed without an explicit budget parameter.
- `v0.5.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 2: RLP Codec In Small Passes

### v0.6.0 - RLP Scalar Decoder

Status: tagged as `v0.6.0`.

Goal: decode RLP bytes and strings with exact consumption.

Deliverables:

- current dependency and GitHub tooling review before parser implementation;
- official execution-spec and EIP source revisions pinned in `spec-lock.toml`;
- scalar RLP item model;
- short and long string handling;
- trailing-data rejection;
- malformed length tests.

Verification:

- `scripts/check_latest_tools.sh`
- `cargo test -p eth-valkyoth-codec`

Exit criteria:

- Scalar RLP inputs are accepted or rejected deterministically.
- `v0.6.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.7.0 - RLP List Decoder

Status: tagged as `v0.7.0`.

Goal: decode nested RLP lists under resource limits.

Deliverables:

- list header parsing;
- nested traversal without recursive stack growth where practical;
- item-count and nesting-depth enforcement;
- adversarial nesting tests.

Verification:

- `cargo test -p eth-valkyoth-codec`

Exit criteria:

- Deep or oversized RLP lists fail closed.
- `v0.7.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.8.0 - Canonical RLP Integers

Status: tagged as `v0.8.0`.

Goal: enforce Ethereum canonical integer rules.

Deliverables:

- leading-zero rejection;
- zero representation policy;
- bounded integer conversion helpers;
- official and negative vector tests.

Verification:

- `cargo test -p eth-valkyoth-codec -p eth-valkyoth-primitives`

Exit criteria:

- Noncanonical integer encodings cannot reach protocol validation.
- `v0.8.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.9.0 - RLP Encoding Round Trips

Status: tagged as `v0.9.0`.

Goal: add canonical encoding for admitted RLP values.

Deliverables:

- encoding helpers;
- decode-then-encode canonicality tests;
- property tests or table-driven round trips.

Verification:

- `cargo test -p eth-valkyoth-codec`

Exit criteria:

- Canonical values round-trip without accepting noncanonical forms.
- `v0.9.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.9.1 - Canonical Integer Source Of Truth

Status: tagged as `v0.9.1`.

Goal: remove duplicated Ethereum RLP integer canonicality logic between
`eth-valkyoth-codec` and `eth-valkyoth-primitives`.

Deliverables:

- public codec helpers for canonical integer payload validation and conversion;
- primitive constructors delegate canonical payload parsing to codec helpers;
- primitive errors map codec failures without leaking codec internals into
  primitive domain APIs;
- cross-crate tests proving codec and primitives accept and reject identical
  integer payloads;
- comments that explicitly forbid reintroducing duplicate canonical integer
  parsing in primitives.

Verification:

- `cargo test -p eth-valkyoth-codec -p eth-valkyoth-primitives`
- `scripts/checks.sh`

Exit criteria:

- There is one implementation of canonical RLP integer payload rules.
- A canonicality-rule change cannot silently diverge between codec and
  primitive domain constructors.
- `v0.9.1 implementation stop reached. Run pentest for this exact
  commit.`

### v0.9.2 - Primitive RLP Bridge

Status: tagged as `v0.9.2`.

Goal: make primitive domain types directly usable with the bounded RLP codec
without callers writing repeated field glue.

Deliverables:

- buffer-based RLP encode helpers for `ChainId`, `BlockNumber`, `Gas`, `Nonce`,
  `UnixTimestamp`, `Wei`, `Address`, and `B256`;
- exact-consumption RLP decode helpers for the same primitive domains;
- fixed-width byte scalar policy for address and hash domains;
- no-allocation APIs with caller-provided output buffers;
- table-driven round-trip and malformed-input tests.

Verification:

- `cargo test -p eth-valkyoth-codec -p eth-valkyoth-primitives -p eth`
- `scripts/checks.sh`

Exit criteria:

- Users can encode and decode common Ethereum primitive fields without
  reimplementing codec/primitive bridging themselves.
- Primitive RLP helpers preserve the single canonicality source established in
  `v0.9.1`.
- `v0.9.2 implementation stop reached. Run pentest for this exact
  commit.`

### v0.9.3 - Keccak Boundary Decision

Status: tagged as `v0.9.3`.

Goal: decide and document the Keccak-256 boundary before transaction hashes,
sender recovery, or header hashing are implemented.

Deliverables:

- evaluated options: trait boundary, admitted single dependency, or both behind
  explicit feature gates;
- no_std, no-alloc, license, maintenance, and audit review for any dependency
  considered;
- `eth-valkyoth-hash` crate with the minimal hasher trait and test doubles;
- `docs/keccak-boundary.md` with the dependency decision and future admission
  checklist;
- if admitting a dependency, add dependency-policy evidence before use;
- release-plan updates for transaction hashing, sender recovery, header
  hashing, and proof verification milestones.

Verification:

- `scripts/checks.sh`
- `cargo deny check`
- `cargo audit`

Exit criteria:

- Later transaction and proof work has an explicit hashing boundary and cannot
  accidentally pull hashing into the default graph without review.
- `v0.9.3 implementation stop reached. Run pentest for this exact
  commit.`

### v0.10.0 - RLP Fuzz Harness

Status: tagged as `v0.10.0`.

Goal: continuously fuzz every RLP parser.

Deliverables:

- cargo-fuzz workspace before additional parser expansion;
- RLP fuzz target;
- committed hex seed corpus from unit fixtures and adversarial cases;
- local seed materializer for ignored `fuzz/corpus/` directories;
- crash reproduction docs.

Verification:

- fuzz target builds;
- committed seed corpus validates;
- `scripts/checks.sh`

Exit criteria:

- Every future RLP parser change has a fuzz target to update.
- No untrusted parser ships without a corresponding fuzz target.
- `v0.10.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 3: Transaction Envelopes

### v0.11.0 - Transaction Envelope Shell

Status: tagged as `v0.11.0`.

Goal: classify legacy and typed transaction envelopes safely.

Deliverables:

- EIP-2718 and execution-spec revisions pinned in `spec-lock.toml`;
- EIP-2718 envelope type model;
- unsupported transaction type errors;
- exact-consumption tests.

Verification:

- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-codec`

Exit criteria:

- Unknown transaction types are rejected or represented explicitly without
  panics.
- `v0.11.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.12.0 - Legacy Transaction Decode

Status: tagged as `v0.12.0`.

Goal: decode legacy Ethereum transactions without sender recovery.

Deliverables:

- field model;
- gas, value, nonce, input, and signature field bounds;
- malformed field tests.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Legacy transactions can be decoded into an unvalidated state only.
- `v0.12.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.13.0 - Access List Transaction Decode

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tag.

Goal: decode EIP-2930 access-list transactions.

Deliverables:

- access-list structure;
- address and storage-key limits;
- duplicate and oversize policy;
- negative tests.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Access lists are bounded before validation.
- `v0.13.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.14.0 - Dynamic Fee Transaction Decode

Status: tagged as v0.14.0.

Goal: decode EIP-1559 dynamic-fee transactions.

Deliverables:

- minimal `Transaction1559` field model with typed primitive domains;
- max-fee and priority-fee fields;
- `to`, `value`, calldata, and access-list field shape;
- fee ordering checks deferred to validation state;
- malformed transaction tests.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Dynamic-fee transactions parse without implying they are valid for a fork.
- `v0.14.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.15.0 - Blob Transaction Decode

Status: tagged as v0.15.0.

Goal: decode EIP-4844 blob transaction structure.

Deliverables:

- blob versioned-hash list;
- blob fee fields;
- list size limits;
- malformed and oversize tests.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Blob transaction data remains bounded and fork-unvalidated.
- `v0.15.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.16.0 - Transaction Encoding

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tag.

Goal: encode admitted transaction envelopes canonically.

Deliverables:

- canonical envelope encoding;
- EIP-1559 transaction encoding as the first useful transaction encode path;
- round-trip tests for each admitted type;
- unsupported type behavior documented.

Verification:

- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-codec`

Exit criteria:

- Transaction encoding cannot produce known noncanonical forms.
- `v0.16.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.16.1 - RLP Derive Evaluation

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tag.

Goal: decide and prototype derive support for RLP encoding and decoding only
after hand-written primitive and transaction APIs have stabilized.

Deliverables:

- derive macro API design for `RlpEncode` and `RlpDecode`;
- explicit field-order policy and skip/default-field rules;
- documentation requiring generated code to use the same bounded codec helpers
  as hand-written paths;
- negative tests for unsupported generics, enums, unions, and ambiguous fields;
- documentation explaining when to prefer hand-written implementations.

Verification:

- `cargo test -p eth-valkyoth-derive -p eth-valkyoth-codec -p eth-valkyoth-protocol`
- `scripts/checks.sh`

Exit criteria:

- Derive macros cannot bypass decode budgets, canonicality checks, or
  transaction/fork validation typestates.
- `v0.16.1 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 4: Fork And Validation States

### v0.17.0 - Chain And Fork Specs

Status: tagged.

Goal: make chain and fork activation rules explicit.

Deliverables:

- execution-spec and relevant hardfork/EIP revisions pinned in
  `spec-lock.toml`;
- `ChainSpec` and `ForkSpec`;
- block-number and timestamp activation;
- unsupported fork errors;
- tests for boundary transitions.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Consensus-sensitive operations require explicit chain and fork context.
- `v0.17.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.18.0 - Transaction Validation Typestates

Status: tagged.

Goal: separate decode, canonicality, fork validation, and sender recovery.

Deliverables:

- transaction state markers;
- invalid transition tests;
- no partial mutation on failed validation.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Callers cannot accidentally treat decoded bytes as fork-valid transactions.
- `v0.18.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.19.0 - Replay Domain Validation

Status: tagged.

Goal: validate transaction chain binding before signatures are trusted.

Deliverables:

- EIP-155 chain checks;
- typed transaction chain checks;
- wrong-chain test vectors.

Verification:

- `cargo test -p eth-valkyoth-verify -p eth-valkyoth-protocol`

Exit criteria:

- Wrong-chain transactions fail before sender recovery results are accepted.
- `v0.19.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.20.0 - Sender Recovery

Status: tagged.

Goal: recover senders through an admitted secp256k1 dependency.

Deliverables:

- sender-recovery hashing uses the `eth-valkyoth-hash` trait boundary;
- sender-recovery hasher state-clearing requirements are documented or enforced
  at the call site;
- dependency admission record;
- low-s and recovery-id policy;
- valid and invalid signature fixtures.

Verification:

- `cargo test -p eth-valkyoth-verify`
- `cargo deny check`
- `cargo audit`

Exit criteria:

- Sender recovery has deterministic failure modes and dependency evidence.
- `v0.20.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.21.0 - EIP-712 Domain Safety

Status: tagged and released.

Goal: prevent structured-data domain confusion.

Deliverables:

- complete-domain helpers;
- expected chain and verifying-contract checks;
- tests for missing or wrong domain fields;
- EIP-712 signing digest helper using the EIP-191 `0x1901` prefix and
  caller-provided Keccak-256 boundary;
- domain-gated sender recovery helper so raw digest recovery is not the primary
  safe structured-data signing interface.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Raw digest signing is not the primary safe signing interface.
- `v0.21.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 4A: Signing And Encoding Completeness

These milestones were added after the `v0.21.0` planning review to make
previously implied work explicit. The goal is to avoid reaching signer, RPC, or
1.0 hardening work while still depending on caller-built hashes or external
typed-data encoders for core Ethereum signing flows.

### v0.22.0 - Transaction Signing Hashes

Status: tagged and released.

Goal: construct Ethereum transaction signing hashes from admitted decoded
transaction domains.

Deliverables:

- legacy EIP-155 signing preimage construction;
- EIP-2930 signing preimage construction;
- EIP-1559 signing preimage construction;
- EIP-4844 signing preimage construction;
- transaction hash domain newtype instead of raw `B256`;
- caller-provided Keccak-256 boundary for all signing hashes;
- tests against official or independently generated transaction vectors.

Verification:

- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify`

Exit criteria:

- Sender recovery no longer requires downstream callers to hand-build standard
  transaction signing digests.
- `v0.22.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.23.0 - Full Transaction Signature Validation

Status: tagged and released.

Goal: validate decoded transaction signatures end to end.

Deliverables:

- typed helpers that combine replay-domain checks, signing-hash construction,
  low-s/y-parity policy, and sender recovery;
- legacy `v` handling through EIP-155 chain binding;
- typed transaction signature validation for EIP-2930, EIP-1559, and EIP-4844;
- at least one external raw mainnet transaction KAT for each typed transaction
  family, sourced independently through public Ethereum RPC and checked against
  the RPC `from` sender;
- validated-signature result carrying the recovered sender and signing hash;
- protocol typestate sender-recovered promotion remains deferred until public
  proof constructors can be bound to transaction identity;
- wrong-chain, wrong-sender, high-s, and malformed-scalar tests.

Verification:

- `cargo test -p eth-valkyoth-verify -p eth-valkyoth-protocol`

Exit criteria:

- A caller can validate decoded transaction signatures without using raw digest
  recovery as the primary API.
- `v0.23.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.24.0 - Set-Code Transaction Decode

Status: tagged and released.

Goal: decode and encode the next typed transaction family currently left
opaque by the transaction-envelope shell.

Deliverables:

- current official EIP/source check for set-code transactions before coding;
- typed transaction prefix admission;
- field model with explicit authorization-list domain types;
- bounded decode and no-allocation encode helpers;
- fuzz seed coverage for malformed and maximal authorization lists;
- scope note for validation deferred to `v0.24.1` and `v0.24.2`.

Verification:

- `cargo test -p eth-valkyoth-protocol`
- `cargo check --manifest-path fuzz/Cargo.toml`

Exit criteria:

- The README no longer has to list set-code transaction parsing as an omitted
  typed transaction family.

Implementation note:

- Official source check completed against final EIP-7702 on 2026-07-01:
  transaction type `0x04`, authorization magic `0x05`, transaction payload
  fields, required destination address, and authorization tuple shape
  `[chain_id, address, nonce, y_parity, r, s]`.
- `v0.24.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.24.1 - Set-Code Signing And Authorization Validation

Status: implemented; pentest passed; ready for release.

Goal: add the cryptographic EIP-7702 validation pieces that were intentionally
left out of the syntactic set-code decoder.

Deliverables:

- refresh the official EIP-7702 source check before implementation;
- set-code transaction signing preimage and signing-hash helpers for type
  `0x04`;
- set-code authorization tuple signing hash using the EIP-7702 authorization
  magic/domain;
- authorization signer recovery with low-s, scalar, and y-parity policy;
- decoded set-code transaction signature validation no longer returns
  `UnsupportedTransactionType`;
- explicit tests for transaction signature validation versus authorization
  tuple signature validation so the domains cannot be substituted;
- KATs or independently generated vectors for set-code transaction hashes and
  authorization signer recovery;
- fuzz coverage for malformed authorization signatures and hash-construction
  scratch-buffer limits.

Verification:

- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify`
- `cargo check --manifest-path fuzz/Cargo.toml`

Exit criteria:

- A syntactically decoded EIP-7702 transaction can have its transaction
  signature and each authorization tuple signature validated through explicit
  verify-layer APIs.
- The transaction-signature domain and authorization-signature domain are
  represented by distinct APIs or newtypes.

Implementation note:

- Official source check refreshed against final EIP-7702 on 2026-07-02:
  set-code transactions sign `keccak256(0x04 || TransactionPayload)`, and
  authorization tuples sign `keccak256(0x05 || rlp([chain_id, address, nonce]))`.
- `v0.24.1 implementation stop reached. Run pentest for this exact
  commit.`

### v0.24.2 - Set-Code Transaction Validity Gate

Status: tagged as `v0.24.2`.

Goal: add the non-cryptographic EIP-7702 validity checks that decide whether a
decoded set-code transaction can advance beyond the unvalidated state.

Deliverables:

- refresh the official EIP-7702 source check before implementation;
- validation API that rejects empty authorization lists before a set-code
  transaction is considered transaction-valid;
- authorization chain-ID policy for universal chain ID `0` versus the expected
  chain, reported as per-tuple skip accounting;
- authorization nonce and integer-bound policy from the official EIP, reported
  as per-tuple skip accounting;
- fork activation check for set-code transaction admission;
- caller-provided account-state/delegation view for the EIP-7702 account and
  delegation-indicator rules;
- fee, gas, and account-state integration points that do not require a bundled
  node or trusted RPC dependency;
- typestate or proof token that distinguishes merely decoded set-code
  transactions from set-code transactions that passed the validity gate;
- regression tests for empty authorization lists, wrong authorization chain
  skips, stale authorization nonce skips, inactive fork, synthesized nonce-0
  empty account state, and malformed delegation-state skips.

Verification:

- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify`
- fork/account-state validation fixtures documented in the release notes.

Exit criteria:

- Empty authorization lists remain accepted by the syntactic decoder but are
  rejected by the set-code validity gate.
- Downstream callers have a single documented API boundary for promoting an
  EIP-7702 transaction from decoded/unvalidated to valid-for-context.
- Downstream callers can inspect applied and skipped authorization tuple counts
  without accidentally treating every skipped tuple as transaction-fatal.

Implementation note:

- Official source check refreshed against final EIP-7702 on 2026-07-02:
  set-code transactions require non-empty authorization lists; authorization
  chain IDs must be universal chain ID `0` or the current chain; authorization
  nonces must be less than `2**64 - 1`; missing authority accounts are treated
  as nonce-0 empty accounts by caller-supplied state views; and authority code
  must be empty or an EIP-7702 delegation indicator before a tuple can be
  applied. Failed tuple checks skip that tuple rather than rejecting the whole
  transaction.
- `v0.24.2 implementation stop reached. Run pentest for this exact
  commit.`

### v0.25.0 - Public RLP Derives

Status: tagged and released.

Goal: turn the private RLP derive prototype into a reviewed public derive
surface.

Deliverables:

- public `RlpEncode` and `RlpDecode` derives for supported structs;
- generated decode paths require `DecodeLimits`;
- generated integer and fixed-width field code delegates to codec and primitive
  helpers;
- explicit diagnostics for unsupported generics, enums, unions, and unsafe
  transaction-state shortcuts;
- trybuild-style compile-fail coverage.

Verification:

- `cargo test -p eth-valkyoth-derive`
- docs examples compile.

Exit criteria:

- Users can derive RLP for simple reviewed structs without bypassing the
  bounded codec contract.
- `v0.25.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.26.0 - EIP-712 Typed-Data Encoder

Status: tagged and released.

Goal: implement the full EIP-712 typed-data hashing pipeline instead of relying
on caller-provided `domainSeparator` and `hashStruct(message)` values.

Deliverables:

- `encodeType` implementation with dependency collection and canonical type
  ordering;
- `encodeData` implementation for admitted atomic, dynamic, array, and struct
  fields;
- `hashStruct` helper;
- EIP-712 domain separator construction;
- explicit no-JSON decision for this release, with JSON-RPC typed-data parsing
  scheduled in `v0.26.1`;
- official EIP-712 test vectors and adversarial type-graph tests;
- clear recursion, allocation, and input-size limits.

Verification:

- `cargo test -p eth-valkyoth-verify -p eth-valkyoth-protocol`
- EIP-712 vector test command documented in release notes.

Exit criteria:

- EIP-712 signing APIs no longer need callers to supply externally constructed
  domain and message hashes for standard typed data.

Implementation note:

- `v0.26.0` accepts caller-provided borrowed descriptors and values. It does
  not parse JSON typed-data documents, so the default crate remains `no_std`
  and allocation-free. JSON parsing is a separate boundary in `v0.26.1`.
- `v0.26.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.26.1 - EIP-712 JSON Typed-Data Parser Boundary

Status: tagged as `v0.26.1`.

Goal: admit a reviewed way to parse JSON-RPC typed-data payloads into the
borrowed EIP-712 encoder boundary without weakening the default `no_std` graph.

Deliverables:

- optional `json` feature in `eth-valkyoth-verify` and `eip712-json` facade
  feature in `eth`;
- dependency and license review for current `serde` and `serde_json`;
- size limits for type maps, field counts, array lengths, strings, and dynamic
  bytes;
- validation that parsed type strings map exactly to the `v0.26.0` descriptor
  model;
- JSON fixtures for Ether Mail and adversarial duplicate/missing type fields,
  parser limits, malformed hex, fixed-array mismatch, signed integer
  boundaries, and domain validation;
- duplicate JSON object-key rejection before type maps are admitted, with a
  bounded object-width guard.

Verification:

- parser-specific tests;
- `cargo deny check`;
- release notes documenting whether JSON support is first-party or
  application-owned.

Exit criteria:

- JSON-RPC typed-data payload handling has an explicit, versioned boundary
  instead of being treated as an informal caller responsibility.
- `v0.26.1 implementation stop reached. Run pentest for this exact
  commit.`

### v0.27.0 - Optional Keccak Backend Admission

Status: tagged and published.

Goal: optionally provide a reviewed software Keccak-256 backend without adding
it to the default core graph.

Deliverables:

- latest-version, license, feature, no_std, and maintenance review for
  `tiny-keccak 2.0.2`;
- `eth-valkyoth-hash` feature `tiny-keccak` and facade feature `keccak-tiny`,
  both outside default `eth`;
- `KECCAK256_EMPTY`, `KECCAK256_ABC`, and chunk-boundary KATs;
- EIP-712 JSON parser fuzz target, committed JSON seeds, and raw JSON
  structural-depth regression added during pentest remediation;
- duplicate-dependency and MSRV review;
- state-clearing contract documented for sender-recovery paths.

Verification:

- `cargo test -p eth-valkyoth-hash --all-features`
- `cargo test -p eth-valkyoth-verify --features json`
- `cargo deny check`
- `cargo audit`

Exit criteria:

- Applications that want a first-party software backend can opt in without
  changing the dependency-free default boundary.
- `v0.27.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 5: Blocks, Receipts, And Proofs

### v0.28.0 - Header Decode And Hashing

Status: tagged and published.

Goal: parse and hash execution-layer block headers.

Deliverables:

- header field model;
- fork-specific optional field handling;
- header hashing uses the `eth-valkyoth-hash` trait boundary;
- block header hashes use a domain newtype instead of raw `B256`;
- hash consistency tests.

Verification:

- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify`

Exit criteria:

- Headers can be decoded without implying full block validity.
- `v0.28.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.29.0 - Receipt Decode

Status: tagged as `v0.29.0`.

Goal: parse legacy and typed receipts.

Deliverables:

- receipt status/root policy;
- log structure;
- bloom field handling;
- malformed receipt tests.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Receipt data is bounded before trie or block validation.
- `v0.29.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.30.0 - Withdrawal And Post-Merge Fields

Status: tagged as `v0.30.0`.

Goal: model post-merge execution fields explicitly.

Deliverables:

- withdrawal structures;
- withdrawals-root input model;
- timestamp/fork interaction tests.
- withdrawal-list fuzz target.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Post-merge fields are not bolted onto pre-merge validation paths.
- `v0.30.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.31.0 - MPT Node Decoder

Status: tagged and published.

Goal: decode trie nodes with strict limits.

Deliverables:

- trie node representation;
- proof-node count and byte limits;
- malformed node tests.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Trie proof input cannot allocate or recurse without limits.
- `v0.31.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.32.0 - Inclusion Proof Verification

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

Goal: verify transaction and receipt inclusion proofs.

Deliverables:

- transaction proof verification;
- receipt proof verification;
- proof walkers reuse checked MPT node decode state where possible, avoiding
  duplicate proof-node and inline-child decode work introduced by the
  allocation-free v0.31.0 syntactic boundary;
- proof root hashing uses the `eth-valkyoth-hash` trait boundary;
- transaction hashes, receipt roots, and proof roots use distinct domain
  newtypes instead of raw `B256`;
- invalid proof fixtures.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Inclusion proof APIs distinguish malformed, absent, and wrong-root proofs.
- `v0.32.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.33.0 - Account And Storage Proofs

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

Goal: verify account and storage proofs against trusted roots.

Deliverables:

- account proof verification;
- storage proof verification;
- account and storage proof root hashing uses the `eth-valkyoth-hash` trait
  boundary;
- account and storage proof roots use distinct domain newtypes instead of raw
  `B256`;
- missing-node and wrong-value tests;
- wrong-root, absent-key, and proof-depth negative tests;
- proof-verification fuzz coverage for transaction, receipt, account, and
  storage entry points through a real Keccak backend.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Verified RPC state has a cryptographic proof path separate from trusted RPC.
- `v0.33.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 6: Conformance And Test Infrastructure

### v0.34.0 - Spec Lock And Fixture Import

Status: tagged as `v0.34.0`.

Goal: pin official Ethereum specification and fixture revisions.

Deliverables:

- populated `spec-lock.toml`;
- fixture import or download process;
- fixture license notes;
- reproducible fixture path;
- `/home/eldryoth/Work/test/eth` documented as the local reference store.

Verification:

- `scripts/checks.sh`

Exit criteria:

- Every conformance claim names exact upstream revisions.
- `v0.34.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.35.0 - Execution Test Harness

Status: tagged as `v0.35.0`.

Goal: run applicable Ethereum execution tests through protocol validation.

Deliverables:

- fixture runner;
- pass/fail report;
- known unsupported fixture list.

Verification:

- conformance runner command documented and passing for claimed fixtures.

Exit criteria:

- Validation behavior is tested against external Ethereum material.
- `v0.35.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.36.0 - Differential Test Harness

Status: tagged as `v0.36.0`.

Goal: compare selected behavior against independent implementations.

Deliverables:

- differential test plan;
- adapter for at least one independent reference path;
- mismatch reporting.

Verification:

- differential test command documented.

Exit criteria:

- The project is not only testing wrappers against themselves.
- `v0.36.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 7: Optional Execution

### v0.37.0 - REVM Dependency Admission Review

Status: tagged as `v0.37.0`.

Goal: review REVM for optional admission behind `eth-valkyoth-evm` without
weakening dependency policy.

Deliverables:

- dependency review;
- explicit non-admission result when policy fails;
- code-visible review metadata.

Verification:

- `cargo check --workspace --all-features`
- `cargo deny check`

Exit criteria:

- REVM cannot enter the graph until cargo-deny, MSRV, and feature policy pass.
- `v0.37.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.37.1 - REVM Dependency Recheck

Status: tagged as `v0.37.1`.

Goal: recheck the REVM ecosystem before execution adapter work starts and add
automation so future REVM/fork drift is visible.

Deliverables:

- latest REVM version check;
- MSRV-compatible REVM line check;
- dependency-policy result update;
- `scripts/check_ethereum_upstream.sh` or equivalent networked advisory check
  for latest REVM, official Ethereum hardfork/spec revisions, and pinned
  execution fixture revisions;
- safe network-fetch policy: metadata-only requests, no `curl | sh`, no `eval`,
  no execution of fetched content, and pinned commit/tag SHAs before any source
  is trusted for implementation work;
- maintenance-release report format for newly detected fork rules, opcode/gas
  schedule changes, precompile changes, transaction type changes, or fixture
  updates;
- decision on REVM, a narrower REVM subcrate, or an alternate execution
  boundary.

Verification:

- upstream-check script documents its sources and exits non-zero only for local
  policy/script failures, not merely because upstream moved;
- `cargo deny check`
- `cargo check --workspace --all-features`

Exit criteria:

- Execution work is either unblocked by a clean admitted graph or remains
  explicitly blocked with a documented reason.
- `v0.37.1 implementation stop reached. Run pentest for this exact
  commit.`

### v0.37.2 - Core Dependency Independence Audit

Status: complete and tagged.

Goal: review every dependency that touches core Ethereum behavior and decide
whether it is first-party, optional backend, reference-only, or temporary debt.

Deliverables:

- `docs/core-independence-audit.md`;
- inventory of default, optional, dev-only, and test-only dependencies that
  influence hashing, signatures, RLP, trie/proof behavior, execution, consensus,
  networking, or RPC semantics;
- explicit assessment of current default `k256` use in sender recovery;
- explicit assessment of current `sha3` use in verification tests and whether
  it should be dev-only, feature-gated, or replaced by `eth-valkyoth-hash`;
- policy for cryptographic primitives where a fully first-party implementation
  would be higher risk than a reviewed backend;
- versioned follow-up rows for every core dependency that remains in the graph.

Verification:

- `cargo tree -e features` evidence captured in the audit;
- `cargo deny check`;
- `cargo audit`.

Exit criteria:

- No core Ethereum dependency is accidental or undocumented.
- Every third-party core implementation has a boundary, an optional/reference
  classification, or a first-party replacement milestone.
- `v0.37.2 implementation stop reached. Run pentest for this exact
  commit.`

### v0.37.3 - Signature And Crypto Backend Boundaries

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

Goal: remove direct default dependence on cryptographic implementation crates
from verification APIs where a first-party boundary is feasible.

Deliverables:

- recoverable secp256k1 verification trait or backend boundary;
- `k256` moved behind an explicit reviewed backend feature, compatibility
  adapter, or documented exception with no hidden default expansion;
- verification tests use the project hashing boundary instead of a direct
  default `sha3` dependency where practical;
- KATs and malformed-signature vectors that run against every admitted
  secp256k1 backend;
- documentation for HSM, platform, WASM, or audited software backends;
- migration notes for callers that used direct sender-recovery helpers.

Verification:

- `cargo test -p eth-valkyoth-verify --all-features`;
- default `eth` dependency graph check proving no unintended concrete
  signature or hashing implementation enters by default;
- `cargo deny check`;
- `cargo audit`.

Exit criteria:

- Core signature verification APIs are boundary-driven.
- Concrete cryptographic implementation crates are explicit choices, not
  invisible protocol-core dependencies.
- `v0.37.3 implementation stop reached. Run pentest for this exact
  commit.`

### v0.37.4 - Constant-Time And Reference Dependency Policy

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

Goal: close the remaining default/runtime utility and reference-oracle policy
items found by the v0.37.2 audit.

Deliverables:

- reviewed long-term decision for `subtle` in primitive constant-time equality;
- wrapper or replacement plan if direct `subtle` use remains too broad;
- documented quarantine rule for `alloy-rlp` as a dev/fuzz reference oracle;
- documented fixture-parser rule for dev-only `serde_json` use in codec tests;
- cargo-tree assertions proving reference dependencies do not enter runtime
  crates.

Verification:

- default and all-feature cargo-tree checks documented in release notes;
- `cargo test --workspace --all-features`;
- `cargo deny check`;
- `cargo audit`.

Exit criteria:

- Constant-time helper behavior is either first-party-wrapped or explicitly
  accepted as a reviewed exception.
- Reference oracle crates are documented as test/fuzz-only and cannot silently
  become runtime protocol dependencies.
- `v0.37.4 implementation stop reached. Run pentest for this exact
  commit.`

### v0.37.5 - Optional Parser And Sanitization Boundary Review

Status: implementation, pentest remediation, and clean retest complete; waiting
for GitHub checks before tagging.

Goal: make optional parser and secret-sanitization bridges explicit before
execution, signing, and JSON-facing surfaces expand.

Deliverables:

- review `serde` and `serde_json` feature boundaries for EIP-712 JSON parsing;
- review `eth-valkyoth-sanitization` and external `sanitization` feature
  propagation;
- documentation that optional parser/sanitization crates are not part of the
  default facade graph;
- release-gate checks for default graph absence of optional parser and
  sanitization dependencies;
- follow-up milestones for any optional bridge that needs a narrower
  first-party wrapper.

Verification:

- `cargo tree -p eth -e features --no-default-features`;
- `cargo tree -p eth -e features --all-features`;
- `cargo test --workspace --all-features`;
- `cargo deny check`;
- `cargo audit`.

Exit criteria:

- Optional JSON parsing and sanitization support remain deliberate opt-ins.
- Downstream callers can see exactly when those dependencies enter the graph.
- The permanent pentest report is committed at
  `security/pentest/v0.37.5.md`.
- `v0.37.5 implementation stop reached. Run pentest for this exact
  commit.`

### v0.38.0 - Explicit Execution Environment

Status: implementation, pentest remediation, and clean retest complete; waiting
for GitHub checks before tagging.

Goal: execute with explicit fork, block, transaction, and snapshot inputs.

Deliverables:

- `ExecutionEnvironment` and `BlockExecutionContext` with fork/block
  consistency checks;
- `ExecutionTransaction` binding raw bytes to decoded envelope evidence;
- `StateSnapshot` trait and `SnapshotAccount` view;
- `ExecutionRequest`, `ExecutionReport`, and future `ExecutionResult` model;
- documentation and release gate for the no-backend execution boundary.

Verification:

- `cargo test -p eth-valkyoth-evm`;
- `cargo check -p eth --features evm`;
- `cargo tree -p eth --no-default-features --features evm -e normal`;
- `scripts/release_0_38_gate.sh`.

Exit criteria:

- Simulation reports the exact state and fork configuration used.
- No concrete EVM backend is admitted by this release.
- The permanent pentest report is committed at
  `security/pentest/v0.38.0.md`.
- `v0.38.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.39.0 - Bounded Gas Estimation

Status: implementation, pentest remediation, and clean retest complete; waiting
for GitHub checks before tagging.

Goal: make gas estimation bounded and auditable.

Deliverables:

- `GasEstimationPolicy` with maximum execution attempts, gas cap, and
  deterministic termination guard;
- hard release ceilings for maximum attempts, gas cap, backend steps, and
  worker timeout values;
- timeout, worker-isolation, and backend-step termination policy variants;
- `GasEstimationRequest` binding policy to the explicit execution request;
- `GasEstimationReport` binding outcomes to execution reports;
- deterministic `GasEstimationError` codes and messages.

Verification:

- adversarial gas-estimation tests;
- `cargo test -p eth-valkyoth-evm`;
- `cargo check -p eth --features evm`;
- `scripts/release_0_39_gate.sh`.

Exit criteria:

- Gas estimation cannot become an unbounded execution loop, including through
  practically infinite caller-provided limit values.
- No concrete EVM backend is admitted by this release.
- The permanent pentest report is committed at
  `security/pentest/v0.39.0.md`.
- `v0.39.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 8: Native Audited EVM Engine

This phase builds the first-party execution engine in small audited passes.
REVM, if ever admitted, is temporary/reference-only and must not become the
trusted core for 1.0 production execution claims.

### v0.40.0 - Native EVM Core Types

Status: implementation, pentest remediation, and clean retest complete;
awaiting final GitHub checks.

Goal: introduce the first-party execution crate and bounded core domains.

Deliverables:

- `eth-valkyoth-evm-core` crate;
- stack, memory, word, opcode, program counter, and execution-error domains;
- fork-aware opcode table skeleton;
- no_std-first design with no allocator requirement for fixed limits;
- explicit unsupported-opcode and unsupported-fork errors.

Verification:

- `cargo test -p eth-valkyoth-evm-core`
- stack and memory boundary tests.

Exit criteria:

- The native engine has a small audited type foundation independent of REVM.
- `v0.40.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.41.0 - Native EVM Arithmetic And Control Flow

Status: release candidate; pentest clean.

Goal: implement deterministic opcode execution for arithmetic, comparison, and
control-flow basics.

Deliverables:

- STOP, arithmetic, bitwise, comparison, PUSH, DUP, SWAP, POP, PC, JUMP, and
  JUMPI support;
- checked stack underflow/overflow behavior;
- hard bytecode length ceiling plus one-time no-alloc jumpdest validation;
- RETURN and REVERT shell behavior without state commits.

Verification:

- deterministic local vectors for the claimed opcode set;
- explicit unsupported-fixture documentation until full official state-test
  admission lands;
- differential tests against at least one independent engine when available.

Exit criteria:

- Claimed basic bytecode executes deterministically and fails closed on invalid
  control flow.
- `v0.41.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.42.0 - Native EVM Gas Accounting

Status: pentest passed; waiting for final GitHub checks before tagging.

Goal: make gas costs and memory expansion fork-aware and auditable.

Deliverables:

- fork-scoped gas schedule model;
- memory expansion cost calculation;
- out-of-gas error domain;
- gas accounting tests for boundary and overflow cases.

Verification:

- official gas fixtures for claimed forks;
- fuzz target for gas and memory expansion arithmetic.

Exit criteria:

- Every executed opcode in the claimed set consumes gas before side effects.
- `v0.42.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.43.0 - Native EVM State Access

Status: released.

Goal: add explicit account, code, balance, and storage access through bounded
state traits.

Deliverables:

- account and storage snapshot traits;
- SLOAD, SSTORE shell, BALANCE, EXTCODESIZE, EXTCODEHASH, EXTCODECOPY, and
  SELF BALANCE support where fork-applicable;
- warm/cold access accounting;
- state-read error classification.

Verification:

- official state-access fixtures for claimed forks;
- adversarial storage and code-size limit tests.

Exit criteria:

- State access is explicit, bounded, and fork-aware.
- `v0.43.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.43.1 - Native EVM Historical Fork Matrix

Status: tagged as `v0.43.1`.

Goal: make every historical execution fork explicit before more stateful
opcodes depend on fork selection.

Deliverables:

- first-party `EvmFork` coverage for Frontier, Homestead, Tangerine Whistle,
  Spurious Dragon, Byzantium, Constantinople, Petersburg, Istanbul, Berlin,
  London, Shanghai, Cancun, Prague/Pectra, and scheduled future forks;
- opcode-introduction gates for state opcodes and other already-modeled
  opcodes, including explicit unsupported-opcode errors before the introducing
  fork;
- public support matrix documenting which forks are admitted, fail-closed, or
  fixture-claimed for the current native engine subset;
- alignment notes between protocol `Hardfork` identities and native
  `EvmFork` identifiers.

Verification:

- fork-order and fork-identity tests;
- opcode-table tests at every introducing hardfork boundary;
- documentation check that no historical fork is silently collapsed into a
  later gas model.

Exit criteria:

- A caller can select every historical Ethereum execution fork by name, and
  unsupported behavior fails with explicit fork/opcode errors.
- `v0.43.1 implementation stop reached. Run pentest for this exact
  commit.`

### v0.43.2 - Native EVM Pre-Berlin State Gas Schedules

Status: tagged as `v0.43.2`.

Goal: replace the temporary pre-London state-access fail-closed behavior with
real historical state gas schedules where the current opcode subset is claimed.

Deliverables:

- Frontier, Homestead, Tangerine Whistle, Spurious Dragon, Byzantium,
  Constantinople/Petersburg, Istanbul, and Berlin gas-schedule entries for the
  currently executable state-opcode subset;
- fork-specific BALANCE, SLOAD, SSTORE-shell, EXTCODESIZE, EXTCODECOPY,
  EXTCODEHASH, and SELFBALANCE admission and pricing;
- explicit transition from pre-Berlin flat state gas to Berlin/London+
  warm/cold access accounting;
- tests proving that historical forks no longer use London/Berlin pricing by
  accident.

Verification:

- official or independently derived historical gas vectors for each claimed
  fork boundary;
- state-access fixture subset for claimed historical forks;
- regression test that a missing historical schedule fails closed instead of
  falling through to the latest schedule.

Exit criteria:

- Pre-London state execution is enabled only for forks with implemented,
  reviewed, and tested historical gas/opcode rules.
- `v0.43.2 implementation stop reached. Run pentest for this exact
  commit.`

### v0.44.0 - Native EVM Calls And Create

Status: tagged as `v0.44.0`.

Goal: implement call-frame semantics without hidden host behavior.

Deliverables:

- CALL, CALLCODE, DELEGATECALL, STATICCALL, CREATE, and CREATE2 planning with
  fail-closed interpreter handling;
- call depth, value-transfer, and static-frame policy;
- returndata handling;
- static-call write protection for CALL value and create attempts;
- commit/revert journal checkpoint model.

Verification:

- opcode-introduction boundary tests for call/create opcodes;
- stack/memory validation tests proving the interpreter fails closed without
  popping call/create operands;
- static-frame, return-data, journal, and depth-limit tests.

Exit criteria:

- Calls and creation cannot execute host behavior or commit state outside the
  explicit journal policy.
- `v0.44.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.45.0 - Native EVM Precompiles

Status: tagged as `v0.45.0`.

Goal: admit precompiles as fork-aware, bounded execution units.

Deliverables:

- precompile registry by fork;
- identity execution without third-party dependencies;
- sha256, ripemd160, ecrecover, modexp, BN254, blake2f, KZG point-evaluation,
  and BLS12-381 precompile admission decisions;
- bounded gas and input/output limit policies;
- fail-closed backend boundary for cryptographic precompiles until audited
  backends are admitted.

Verification:

- official precompile address, fork, length, and gas-policy tests;
- `cargo test -p eth-valkyoth-evm-core`;
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`;
- `cargo deny check`;
- fuzz target for precompile input parsing where applicable in the backend
  admission release.

Exit criteria:

- Precompiles are explicit audited modules and do not pull unreviewed crypto
  dependencies into the default graph.
- `v0.45.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.46.0 - Native EVM Hash Precompiles

Status: tagged as `v0.46.0`.

Goal: execute the Frontier SHA-256 and RIPEMD-160 precompiles behind explicit
first-party or reviewed-backend decisions.

Deliverables:

- SHA-256 precompile execution at address `0x02`;
- RIPEMD-160 precompile execution at address `0x03`;
- input padding/output-width behavior matching client semantics;
- no default dependency expansion without a written backend admission review;
- conformance KATs for empty input, short input, one-word input, and multi-word
  input.

Verification:

- official or independently reproduced precompile vectors;
- `cargo test -p eth-valkyoth-evm-core`;
- `cargo deny check`;
- dispatcher regression test proving precompile lookup happens before ordinary
  contract-call handling when CALL execution is wired;
- fuzz target for hash-precompile input length and output-buffer behavior.

Exit criteria:

- Hash precompile execution is deterministic, bounded, and covered by vectors
  before broader state tests can claim Frontier precompile support.
- `v0.46.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.47.0 - Native EVM ECRECOVER Precompile

Status: tagged as `v0.47.0`.

Goal: execute the `ecrecover` precompile without weakening the existing
secp256k1 backend boundary.

Deliverables:

- `ecrecover` input parser for the 128-byte canonical call frame;
- y-parity/v normalization, invalid-signature, and zero-output policy;
- full `0 < s < secp256k1n` scalar acceptance for the precompile, because
  EIP-2 applies low-s validation to transactions but leaves ECRECOVER
  unchanged;
- caller-provided or reviewed optional secp256k1 backend integration;
- address derivation through the Keccak trait boundary;
- backend sanitization requirements for temporary scalar/signature material.

Verification:

- Ethereum ecrecover vectors, including invalid and high-s signatures;
- differential vectors against an admitted reference engine;
- `cargo test -p eth-valkyoth-evm-core -p eth-valkyoth-verify`;
- dependency and sanitization review for any backend crate.

Exit criteria:

- ECRECOVER cannot silently bypass replay/signature policy or introduce a
  default signing/recovery dependency.
- `v0.47.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.48.0 - Native EVM ModExp Precompile

Status: pentest passed; awaiting tag.

Goal: execute the Byzantium modular exponentiation precompile with bounded
memory and gas semantics.

Deliverables:

- EIP-198/EIP-2565 input parser for base, exponent, and modulus lengths;
- exact gas formula by fork;
- bounded first-party no-alloc big-integer execution with an explicit release
  operand cap;
- zero-modulus, empty-base, empty-exponent, and oversized-length handling;
- output buffer and allocation limits.

Verification:

- official modexp vectors across Byzantium and Berlin pricing;
- adversarial length/gas overflow tests;
- fuzz target for the modexp header and length parser;
- `cargo deny check`.

Exit criteria:

- ModExp cannot allocate or run unboundedly from hostile calldata.
- `v0.48.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.49.0 - Native EVM BN254 Add And Mul Precompiles

Status: release candidate; pentest and retest complete.

Goal: execute BN254 point addition and scalar multiplication before pairing is
admitted.

Deliverables:

- BN254 field-element parsing and canonical range checks;
- point-at-infinity and invalid-point behavior;
- add and multiplication execution at addresses `0x06` and `0x07`;
- Byzantium and Istanbul gas schedule coverage;
- dependency/first-party curve implementation review.

Verification:

- official BN254 add/mul vectors;
- invalid field, invalid point, and infinity tests;
- `cargo test -p eth-valkyoth-evm-core`;
- `cargo deny check`.

Exit criteria:

- BN254 add/mul execution is isolated and vector-backed before pairing support
  adds more complex batch behavior.
- `v0.49.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.0 - Native EVM BN254 Pairing Frame Boundary

Status: release candidate; pentest and retest complete.

Goal: admit the BN254 pairing precompile frame safely before non-empty pairing
algebra is implemented.

Deliverables:

- pairing input parser for 192-byte tuple batches;
- empty-input success behavior;
- Byzantium and Istanbul gas formulas;
- G1 and G2 field/range/curve validation for pairing frames;
- non-empty pairing execution fails closed after validation;
- batch-size limits relative to the release precompile input cap.

Verification:

- official G2 generator curve-membership vector;
- malformed tuple, invalid point, and oversized batch tests;
- fuzz target for pairing input segmentation.

Exit criteria:

- Pairing frame validation cannot create an unbounded CPU path independent of
  gas and release limits.
- Non-empty pairing algebra remains explicitly fail-closed until subgroup and
  pairing arithmetic releases land.
- `v0.50.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.1 - Native EVM BN254 Pairing Subgroup Validation

Status: release candidate; pentest and retest complete.

Goal: add reviewed G2 subgroup validation before any non-empty pairing result
can be trusted.

Deliverables:

- first-party G2 scalar multiplication or equivalent subgroup check;
- precompute the BN254 twist curve coefficient used by G2 validation so the
  frame parser does not recompute `3 / (9 + i)` for every tuple;
- explicit subgroup error mapping at the precompile boundary;
- deterministic valid-twist, invalid-subgroup regression vector;
- fuzz coverage for validated non-empty tuple frames.

Verification:

- official G2 generator subgroup acceptance vector;
- deterministic valid-twist, invalid-subgroup rejection vector;
- `cargo test -p eth-valkyoth-evm-core`;
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`.

Exit criteria:

- Every non-empty pairing tuple is rejected unless both G1 and G2 inputs are in
  the admitted groups.
- `v0.50.1 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.2 - Native EVM BN254 Fp6/Fp12 Tower Foundation

Status: release candidate; pentest and retest complete.

Goal: add the first-party Fp6/Fp12 tower arithmetic required by the BN254
Miller loop without claiming non-empty pairing execution yet.

Deliverables:

- Fp6/Fp12 arithmetic split into files below the 500-line cap;
- Fp6/Fp12 zero, one, add, subtract, multiply, square, and tower non-residue
  relations;
- bounded internal tower exerciser tied to the already validated
  pairing tuple count while non-empty execution still fails closed;
- official cross-client invalid-subgroup vector if one is available in the
  admitted Ethereum fixture sources;
- no default BN254, bigint, allocator, crypto, or pairing backend dependency.

Verification:

- algebraic tower relation tests for `v^3 = 9 + i` and `w^2 = v`;
- identity, zero, squaring, and distributivity tests;
- `cargo test -p eth-valkyoth-evm-core bn254_tower`;
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`.

Exit criteria:

- The Fp6/Fp12 tower foundation is deterministic, bounded by tuple count when
  reached from pairing execution, and ready for line-function review.
- The temporary tower exerciser is documented as reachability scaffolding only,
  not a validation boundary.
- Future dispatcher integration must treat `PrecompileBackendUnavailable` as a
  reverting precompile call and must charge the precompile gas before invoking
  pairing parsing or execution.
- `v0.50.2 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.3 - Native EVM BN254 Pairing Tuple Stream

Status: release-ready; pentest passed after remediation and retest.

Goal: add the validated, allocation-free `(G1, G2)` tuple stream that the
Miller-loop releases will consume.

Deliverables:

- internal `Bn254PairingTuple` domain over already validated G1 and G2 points;
- allocation-free tuple streaming helper that stops at the first invalid tuple;
- fail-closed pairing path feeds the Fp12 tower accumulator from typed tuple
  data rather than count-only scaffolding;
- note: `v0.50.5` consumes the same typed tuple stream through the internal
  Miller-loop accumulator;
- dispatcher-facing BN254 pairing plan execution method charges the supplied
  gas meter on every call before validation work is reachable;
- tests for tuple order, stop-on-invalid behavior, and tower accumulation over
  validated tuples, plus repeated plan execution charging on every call.

Verification:

- `cargo test -p eth-valkyoth-evm-core bn254_pairing`;
- `cargo test -p eth-valkyoth-evm-core bn254_tower`;
- `cargo clippy -p eth-valkyoth-evm-core --all-targets --all-features -- -D warnings`.

Exit criteria:

- Validated tuple streaming is deterministic, allocation-free, bounded by
  input length, and does not claim Miller-loop or pairing correctness.
- Dispatcher-style plan execution cannot reach BN254 pairing validation work
  without charging the supplied gas meter for that call.
- `v0.50.3 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.4 - Native EVM BN254 Line-Function Foundation

Status: release candidate; pentest remediation and retest complete.

Goal: implement reviewed first-party line-function arithmetic over the admitted
Fp12 tower and validated tuple stream.

Deliverables:

- line-function coefficient representation;
- point doubling and addition line helpers over validated G2 inputs;
- G1 evaluation wiring without final Miller accumulation claims;
- dispatcher-facing gas-gated plan execution for ModExp and BN254 add/mul,
  matching the pairing hardening boundary;
- explicit documentation that low-level free functions are unmetered helpers for
  standalone tests and fuzzing, not interpreter-dispatch entry points;
- focused algebraic and differential tests for line-helper shape.

Verification:

- line-function relation tests over admitted generator fixtures;
- gas-charge regression tests for BN254 add, BN254 mul, and ModExp plan calls;
- dependency review for any dev-only reference engine.

Exit criteria:

- Line-function arithmetic is deterministic, bounded, and vector-backed before
  Miller-loop accumulation consumes it.
- Dispatcher-facing ModExp, BN254 add/mul, and BN254 pairing execution cannot
  reach validation or arithmetic work without charging the supplied gas meter
  for that call.
- `v0.50.4 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.5 - Native EVM BN254 Miller Loop

Status: release candidate; pentest remediation and retest complete.

Goal: implement the first-party Miller loop over validated BN254 pairing tuples.

Deliverables:

- Miller-loop implementation;
- batch accumulation limits tied to gas and input length;
- deterministic generator, infinity, tuple-stream, and batch accumulation
  vectors;
- fuzz coverage for valid pairing frames reaching the accumulator.

Verification:

- official positive and negative pairing vectors remain reserved for the final
  exponentiation release where complete pairing outputs can be checked;
- fuzz target for batch accumulation shape;
- dependency review confirms no dev-only reference engine is admitted in this
  slice.

Exit criteria:

- Miller-loop accumulation is deterministic, bounded, and vector-backed.
- Non-empty EIP-197 pairing execution remains fail-closed until final
  exponentiation is admitted.
- `v0.50.5 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.6 - Native EVM BN254 Sparse Miller Economics

Status: release candidate; pentest clean.

Goal: close the pairing gas-vs-CPU gap before any non-empty pairing result can
be admitted.

Deliverables:

- sparse line-function multiplication path for the Miller accumulator;
- benchmark harness for `miller_loop_tuple` and batch accumulation;
- documented wall-time budget relative to EIP-1108 pair gas;
- regression test or bench note proving the dense generic `Fp12::mul` path is
  not used for line-function multiplication.

Verification:

- benchmark evidence on the supported Rust stable toolchain;
- fuzz target still reaches the sparse accumulation path;
- pentest gate before tagging.

Exit criteria:

- Non-empty EIP-197 pairing remains fail-closed, and final exponentiation cannot
  be admitted until sparse multiplication and gas/CPU evidence are reviewed.
- `v0.50.6 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.7 - Native EVM BN254 Pairing Final Exponentiation Foundation

Status: released.

Goal: add bounded first-party final exponentiation without claiming non-empty
EIP-197 pairing success before the full optimal-ate accumulator is complete.

Deliverables:

- first-party final exponentiation over the existing Fp12 tower;
- fixed-size exponent schedule for `(p^12 - 1) / q`;
- fail-closed pairing path exercises final exponentiation after validated
  non-empty Miller accumulation without writing an output result;
- tests proving empty input still returns one and non-empty input remains
  `PrecompileBackendUnavailable`;
- roadmap split for the missing optimal-ate post-loop Frobenius/addition terms.

Verification:

- final-exponentiation edge KAT for `Fp12::ONE`;
- inverse-batch regression proving the exponentiation path maps an admitted
  inverse Miller accumulator to one;
- fuzz target still reaches validated non-empty frames and observes
  fail-closed execution;
- pentest gate before tagging.

Exit criteria:

- Final exponentiation is bounded by a fixed exponent and cannot create an
  unbounded CPU path independent of gas and release limits.
- Non-empty EIP-197 pairing execution remains fail-closed until the optimal-ate
  post-loop line terms and final result admission are reviewed.
- `v0.50.7 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.8 - Native EVM BN254 Frobenius Post-Loop Point Foundation

Status: released.

Goal: admit the G2 Frobenius point mapping required by Ethereum's BN254
optimal-ate post-loop terms without wiring an incorrect line-carrier into
public execution.

Deliverables:

- G2 Frobenius map helpers required for the post-loop `Q1` and `-Q2` terms;
- KATs for Frobenius coefficients and point mapping, computed independently
  from the BN254 field modulus and twist factor;
- fail-closed non-empty execution exercises the admitted post-loop point helper
  without multiplying the post-loop lines into the accumulator yet;
- documentation of the discovered line-carrier gap: applying the post-loop
  points through the current affine line carrier maps the EIP-197 generator
  tuple to one after final exponentiation, so a projective/reference-aligned
  line carrier is required before result admission.

Verification:

- official EIP-197 point-encoding semantics;
- Geth/cloudflare BN256 optimal-ate post-loop shape reviewed for Q1/-Q2 point
  construction;
- focused BN254 G2, Miller, and pairing tests;
- pentest gate before tagging.

Exit criteria:

- The Q1/-Q2 point foundation is vector-backed and bounded, but public
  non-empty success remains disabled until the line-carrier and
  result-admission releases.
- `v0.50.8 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.9 - Native EVM BN254 Projective Post-Loop Line Carrier

Status: ready for release after pentest and retest; awaiting GitHub green and tag.

Goal: replace the current affine line-carrier shortcut with a
projective/reference-aligned line carrier that can safely multiply the Q1 and
-Q2 post-loop lines into the accumulator.

Deliverables:

- projective G2 line-function carrier matching the reviewed optimal-ate
  algorithm shape;
- post-loop line additions after the ate loop;
- regression proving the EIP-197 generator tuple is not accidentally mapped to
  one by the completed accumulator and final exponentiation;
- inverse-batch regression still maps to one;
- scalar reconstruction regression for the hard-coded BN254 `6u+2` NAF table;
- bilinearity regression over a G1 generator double:
  `e([2]P, Q) == e(P, Q)^2`;
- non-empty execution still fails closed after computing the complete
  accumulator and final exponentiation.

Verification:

- differential vectors against a reviewed reference engine;
- fuzz target for non-empty complete accumulator execution;
- release-mode gas/CPU evidence for the complete accumulator;
- pentest gate before tagging.

Exit criteria:

- The complete optimal-ate accumulator is vector-backed and bounded, but public
  non-empty success remains disabled until the result-admission release.
- `v0.50.9 implementation stop reached. Run pentest for this exact
  commit.`

### v0.50.10 - Native EVM BN254 Pairing Result Admission

Status: tagged as `v0.50.10`.

Goal: admit non-empty EIP-197 pairing success and failure words only after the
complete accumulator is independently verified.

Deliverables:

- full `execute_bn254_pairing` non-empty result path;
- official EIP-197 positive and negative vectors;
- go-ethereum precompile positive and negative vectors;
- benchmark notes consuming the `v0.50.6` sparse-Miller budget and the
  `v0.50.7` through `v0.50.9` algebra costs;
- fuzz target asserting valid non-empty frames return only canonical `0` or
  `1` output words.

Verification:

- official BN254 pairing vectors;
- differential vectors against an admitted reference engine;
- complete precompile gas-vs-CPU release evidence;
- pentest gate before tagging.

Exit criteria:

- Pairing execution cannot create an unbounded CPU path independent of gas and
  release limits, and every admitted non-empty result is vector-backed.
- `v0.50.10 implementation stop reached. Run pentest for this exact
  commit.`

### v0.51.0 - Native EVM BLAKE2F Precompile

Status: release candidate; pentest remediation and retest complete.

Goal: execute the Istanbul BLAKE2 compression precompile with exact input
shape and round-count behavior, and remediate the pre-existing BN254
final-exponentiation performance issue found during this release's pentest.

Deliverables:

- exact 213-byte input parser;
- final-block flag validation;
- rejection of final-block indicator bytes outside `{0, 1}`;
- round-count gas semantics;
- first-party or reviewed backend implementation decision;
- output buffer behavior matching EIP-152.
- optimized BN254 final exponentiation using Fp6/Fp12 inversion, Frobenius
  operations, the easy-part reduction, and the BN-parameter hard-part chain.

Verification:

- EIP-152 KATs and invalid-input vectors;
- fuzz target for input-shape parsing;
- optimized BN254 final-exponentiation comparison against the previous
  full-exponent reference on real Miller-loop accumulator output;
- release-mode BN254 final-exponentiation timing evidence;
- `cargo test -p eth-valkyoth-evm-core`;
- dependency review if a backend crate is admitted.

Exit criteria:

- BLAKE2F execution is fully shaped by the EIP-152 input contract and cannot
  accept alternate encodings.
- The BN254 final-exponentiation pentest finding is closed without adding
  default crypto, bigint, allocator, or backend dependencies.
- `v0.51.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.0 - Native EVM Advanced Precompile Backends

Status: release candidate; pentest remediation and retest complete.

Goal: version the remaining advanced cryptographic precompile work before full
state-test claims depend on it.

Deliverables:

- KZG point-evaluation execution plan aligned with the blob/KZG release;
- first-party BLS12-381 EIP-2537 implementation plan;
- exact KZG/BLS input-shape, output-length, and gas-planning metadata while
  arithmetic remains fail closed;
- shared backend conformance checklist for hash, secp256k1, big-int, BN254,
  BLAKE2, KZG, and BLS backends;
- sanitization and zeroization requirements for backend scratch state where
  secret-bearing or key-adjacent material may be processed;
- a `0.22.0` minor release for `eth-valkyoth-verify` because EIP-712 schema
  bounds, signing-value trait removals, redacted formatting, and the new public
  error variant change its public compatibility surface;
- release-blocking vector list for each precompile still not executable.

Verification:

- documented backend conformance commands;
- `cargo deny check`;
- release-plan check proving every fail-closed precompile has a later version
  or explicit exclusion.

Exit criteria:

- No cryptographic precompile remains merely "deferred"; each one is either
  implemented, assigned to a concrete later release, or explicitly excluded
  from a claimed fork.
- Every changed support-crate public API has a semver-compatible independent
  crate version and matching workspace, fuzz, publish-plan, lockfile, and
  crate-matrix metadata.
- `v0.52.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.1 - BLS12-381 Canonical Field And Point Encodings

Status: release candidate; pentest remediation and clean retest complete.

Goal: add bounded first-party Fp, Fr, Fp2, G1, and G2 wire domains for EIP-2537.

Deliverables:

- dependency-free fixed-width Fp and scalar representations with no allocator
  requirement;
- canonical 64-byte big-endian Fp decoding that checks all top padding bytes
  and rejects values greater than or equal to the field modulus;
- Fp2 decoding in the exact EIP-2537 `c0 || c1` coefficient order;
- 32-byte big-endian MSM scalar decoding that accepts the full 256-bit input
  domain instead of incorrectly requiring values below the subgroup order;
- exact 128-byte G1 and 256-byte G2 point parsing, including the all-zero
  infinity encodings and rejection of alternate infinity encodings;
- frame parsers for every `0x0b..=0x11` input that reuse the release's existing
  length and item-count policies;
- malformed-field, padding, infinity, and frame-boundary fuzz coverage.
- pentest remediation that zero-initializes caller-provided EVM memory, makes
  execution one-shot until destructive reset, and restores warm/cold access
  tracking on every failed or reverted stateful run;
- pentest remediation that validates EIP-712 identifiers, rejects duplicate
  borrowed type/field/value names and atomic-looking custom struct names,
  bounds borrowed fields, values, and arrays before expensive traversal,
  caps each complete operation at 4,096 recursive value visits, validates
  schemas once, caches type hashes across recursive borrowed and JSON hashing,
  validates fully unwrapped member types even for empty arrays, caps cumulative
  dynamic hashing work with default and caller-selected limits, and clears
  partial encode-data output.

Verification:

- official EIP-2537 field, point, infinity, and scalar encoding rules;
- boundary tests for `0`, `p - 1`, `p`, non-zero top padding, `q`, and scalars
  greater than `q`;
- round-trip tests for every admitted wire domain;
- `cargo test -p eth-valkyoth-evm-core`;
- `cargo test -p eth-valkyoth-verify --all-features`;
- strict core and fuzz clippy gates;
- pentest and retest before tagging.

Exit criteria:

- Every BLS input byte is either decoded into one canonical bounded domain or
  rejected before arithmetic, without allocation or alternate encodings.
- Arithmetic precompiles remain fail closed until their assigned releases.
- EVM execution cannot expose recycled caller memory or silently inherit
  stack, memory, program-counter, or discounted warm-access state.
- EIP-712 signing rejects ambiguous or delimiter-injected schemas before
  hashing and leaves no partial member encoding after failure.
- `v0.52.1 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 8A: Consensus-Correctness And Resource Boundaries

### v0.52.2 - Truncated PUSH Consensus Correction

Status: release candidate; pentest remediation and clean retest complete,
awaiting final GitHub checks.

Goal: make `PUSH1..=PUSH32` match Ethereum's zero-padding rule at end of
bytecode before any broader native-execution claim is admitted.

Deliverables:

- Zero-pad every missing immediate byte during execution instead of rejecting
  truncated `PUSHn` bytecode;
- apply the identical rule in jump-destination analysis and every bytecode
  scanner so analysis and execution cannot disagree;
- remove `PushImmediateOutOfBounds` from consensus execution paths while
  retaining precise host/input errors outside EVM semantics.

Verification:

- Yellow Paper and execution-spec vectors for every truncation length of
  `PUSH1..=PUSH32`;
- differential execution and jump-destination checks against independent
  clients;
- exhaustive boundary tests and fuzz assertions that analysis and execution
  consume the same padded instruction stream.

Exit criteria:

- No valid bytecode diverges because an immediate ends at code EOF.
- `v0.52.2 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.3 - Shared Decode Session And Work Ledger

Status: implementation complete; pentest findings remediated; clean retest
complete; final release gate and GitHub checks pending.

Goal: make one non-copyable budget capability account for all work caused by
one untrusted decode request.

Deliverables:

- A non-copyable `DecodeSession` threaded through envelope classification,
  RLP fields, nested iteration, transaction substructures, proof parsing, and
  owned conversion;
- cumulative counters for encoded bytes scanned, RLP headers visited, items,
  nesting, allocation capacity, hashes, hash bytes, and total work;
- single-pass structural visitation plus pre-charged exact nested-list recounts
  for compatibility metadata;
- charged borrowed-model traversal for access lists, storage keys, blob hashes,
  authorization tuples, and inline MPT nodes;
- named reviewed policies with checked cross-limit relationships and actual
  allocation-capacity debits before allocation.

Verification:

- Focused pre-implementation review of `DecodeSession` ownership, accounting
  conservation, reset authority, and nested-consumer composition;
- Complexity-oracle tests for near-limit nested RLP and composite typed
  transactions;
- fuzzing that asserts cumulative counters never reset or exceed policy;
- allocation tests proving requested capacity, not input length, is charged;
- benchmark evidence that admitted parsing work is linear in the documented
  session budget.

Exit criteria:

- `max_total_*` limits describe the complete untrusted operation rather than
  a collection of independently reset local passes.
- `v0.52.3 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.4 - MPT Proof Preflight And Strict Canonicality

Status: release candidate; pentest remediation and clean retest complete;
awaiting green GitHub CI and CodeQL before tagging.

Goal: reject over-budget or locally non-canonical trie proofs before hashing
attacker-controlled node bytes.

Deliverables:

- Proof preflight for node count, individual encoded length, cumulative bytes,
  and local syntax, followed by a conservative dry traversal that atomically
  checks every remaining parser, hash, nibble, value, and aggregate-work
  ceiling;
- charge the dry traversal itself to the caller's operation-wide session while
  keeping the future verification plan opaque and noncommitting;
- charge and reject before each Keccak invocation;
- reject zero-nibble extension paths and every locally detectable redundant
  extension or degenerate branch form required by canonical trie construction;
- integrate all proof work with the shared `DecodeSession` ledger.

Verification:

- Tests proving no proof-node hasher call occurs after any failed preflight
  charge, independently covering encoded bytes, headers, items, hashes,
  nibbles, values, and aggregate work;
- canonical/non-canonical trie fixtures and boundary vectors;
- structure-aware proof fuzzing that constructs valid roots before mutation;
- complexity assertions for nodes, bytes, nibbles, and hashes.

Exit criteria:

- Proof input cannot trigger unmetered hashing or pass a locally detectable
  non-canonical node form.
- `v0.52.4 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.5 - Composed Account And Storage Proofs

Status: tagged as `v0.52.5`.

Goal: make storage-proof authority derive cryptographically from the verified
account value instead of a caller-supplied unrelated storage root.

Deliverables:

- Canonical account decoding for `[nonce, balance, storageRoot, codeHash]`;
- a `VerifiedAccount` capability produced only by account inclusion or
  canonical absence verification;
- storage-proof APIs that consume the embedded root or a non-forgeable
  capability derived from `VerifiedAccount`;
- canonical account absence, storage absence, and zero-value semantics;
- a complete bounded `eth_getProof` verification workflow.
- migration of the optional secret-handling bridge to `sanitization 2.0` with
  canonical wiping, drop-safety contracts, and runtime protection reporting;
- an MSRV-aware direct dependency freshness gate that detects major releases.

Verification:

- Official and cross-client account/storage proof fixtures;
- negative tests that substitute account values, storage roots, paths, and
  absence claims;
- property and fuzz tests for composed proof chains and zero semantics.
- all direct crates.io dependencies and CI/release tools checked against their
  latest compatible upstream releases.

Implementation evidence:

- the pinned Execution APIs Hive account-plus-storage fixture is verified end
  to end under one shared `DecodeSession`;
- focused negative tests cover root substitution, malformed account fields,
  account absence, storage absence, explicit zero values, and pre-hash account
  rejection;
- the structure-aware MPT fuzz target constructs composed account/storage
  roots, requires successful verification, and rejects an unrelated storage
  proof.
- the optional bridge tests exercise canonical wiping and require generated
  sanitizers to satisfy `DropSafeSanitize`.

Exit criteria:

- A caller cannot combine a valid account proof with storage proven against a
  different trie root.
- Optional secret handling uses the reviewed `sanitization 2.0` contracts, and
  release tooling fails closed on stale direct dependencies.
- `v0.52.5 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.6 - MPT Resolver Multiproof And Snapshot Orchestration

Status: tagged as `v0.52.6`.

Goal: retain the allocation-free proof kernel while providing a bounded
orchestration layer suitable for live synchronization.

Deliverables:

- A `NodeResolver` keyed by node hash rather than proof-array position;
- shared-node deduplication, multiproof traversal, and proof-wide output
  accounting;
- immutable snapshot anchors binding every node in a batch to one state root;
- optional `alloc`/`std` arenas, caches, and parallel scheduling outside the
  consensus kernel with explicit cancellation and capacity limits.

Verification:

- Multiproof fixtures with shared, reordered, missing, duplicate, and
  unrelated nodes;
- snapshot-mixing and cache-poisoning negative tests;
- bounded parallelism, cancellation, and deterministic-result tests;
- canonical short-child hash-reference and empty-branch absence parity tests;
- direct coverage for missing roots, hash mismatches, invalid limits, node
  limits, query limits, and retained-capacity rejection before hashing;
- external pentest and clean retest on the exact implementation lineage.

Exit criteria:

- Live proof consumers can resolve and deduplicate nodes without weakening
  root binding, deterministic verification, or `no_std` kernel portability.
- Resolver and owned-arena admission reject non-canonical proof semantics and
  over-budget raw or retained input before attacker-controlled hashing or
  infallible reallocation.
- The release pentest is recorded, every finding is remediated, and the clean
  retest passes before the final report-only commit and tag.
- `v0.52.6 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.7 - Execution Admission And Host Capability Split

Status: implementation complete; pentest findings remediated; clean retest
passed. Awaiting green GitHub CI and CodeQL before tagging.

Goal: prevent an opaque EIP-2718 classification result from becoming
executable and separate execution host responsibilities into auditable
capabilities.

Deliverables:

- Explicit `ClassifiedEnvelope -> CanonicallyDecodedTransaction ->
  ForkValidatedTransaction -> ExecutionReadyTransaction` promotion;
- rejection of unknown or empty typed envelopes at execution admission;
- request-bound private `StateJournal`, `AccessTracker`, `CryptoProvider`, and
  `TransactionArena` capabilities plus post-transition `Inspector` events;
- immutable original storage, journal-authoritative current storage,
  transaction-wide warmth with scope rollback, closure-scoped iterative call
  frames, and resettable transaction arenas;
- fail-closed host poisoning after partial lifecycle transitions or unwinding
  from an unfinished child;
- pessimistic poisoning across destructive transaction reset and every direct
  root journal, access, crypto, or arena mutation;
- semantic YAML validation of every GitHub Action `uses` key, including flow
  mappings and reusable workflows, canonical checkout identity, and
  digest-only Docker actions, job containers, and services, including
  dot-prefixed `.yml` and `.yaml` workflow files.

Verification:

- Compile-fail tests preventing shell envelopes from reaching execution;
- fork/type admission matrices and unknown-type negative tests;
- nested-call rollback tests proving state and scope-local warmth revert while
  pre-entry warmth survives;
- nested LIFO finalization and journal-authoritative post-write storage tests;
- frame-rejection cleanup tests preserving both the arena rejection and any
  journal rollback failure;
- partial-finalization poisoning and post-transition inspector dispatch tests;
- panic-unwind tests at checkpoint, frame entry, rejection rollback,
  commit/revert, frame exit, and child execution proving unfinished child
  scopes poison the host and block every subsequent mutable capability;
- second-reset panic matrices for journal, access tracker, and arena plus root
  mutation panic matrices for storage, warmth, hashing, recovery, and memory;
- conserved legacy classification and canonical-reparse accounting;
- semantic Action-pin fixtures covering block/flow syntax, quoted keys and
  values, both workflow extensions, reusable workflows, local and Docker
  actions, expressions, malformed values, spoofed checkout comments,
  noncanonical checkout casing, mutable action/job/service images, and normal
  or dot-prefixed workflow filenames;
- deep-call and memory-expansion tests proving EVM exceptions replace host
  recursion or bounds errors.

Exit criteria:

- Only fork-validated typed transactions can enter a request-bound execution
  host whose state/environment provenance and private capabilities cannot be
  independently substituted.
- No ordinary return, error, or panic unwind can leave an unfinished child
  lifecycle reusable, and no YAML spelling or comment can bypass immutable,
  current Action pins; every regular workflow file is checked regardless of a
  leading dot in its filename.
- Partial transaction resets and failed or unwound root mutations cannot
  remain execution-usable, and every executable container reference is
  content-addressed.
- The release pentest is recorded, all 15 findings are remediated, and the
  clean retest passes before the final report-only commit and tag.
- `v0.52.7 implementation stop reached. Run pentest for this exact
  commit.`

### v0.53.0 - Bounded Access Tracking And Execution Governor

Status: implementation complete; all eight pentest findings remediated; clean
retest passed. Awaiting green GitHub CI and CodeQL before tagging.

Goal: eliminate quadratic warm-access membership from node-scale execution
while preserving a bounded `no_alloc` implementation for embedded users.

Deliverables:

- An injectable `AccessTracker` with deterministic pre-reserved compressed
  radix indexes, bounded undo storage, and documented fixed-key-width lookup,
  insertion, and mutation-proportional rollback costs;
- nested LIFO warmth checkpoints aligned with journal child scopes;
- retain fixed-array tracking only as an explicit embedded profile;
- transaction-scoped reset and capacity enforcement for access sets, journals,
  frames, memory, reusable arenas, and caches;
- type-separated cumulative and high-water accounting plus bounded
  hierarchical execution work tokens compatible with the later node resource
  governor;
- Prague-aware total warm-address capacity and optimizer-resistant erasure of
  allocator-backed address/slot keys on rollback, reset, and drop.

Verification:

- Adversarial distinct-address and storage-slot benchmarks;
- differential warmth semantics across embedded and node trackers;
- repeated zero-change and one-insertion child reverts over populated outer
  scopes, proving rollback does not rebuild retained index state;
- capacity, nested rollback, partial-finalization poisoning, reset,
  cancellation, retained-memory sanitization, and accounting-mode tests.

Exit criteria:

- Adversarial access patterns cannot turn valid gas-bounded execution into an
  undocumented quadratic host workload.
- Child rollback work is proportional only to unique insertions after that
  checkpoint, independent of retained outer-scope access-set size.
- Reverted child scopes cannot retain discounted access state, accounting mode
  cannot be selected incorrectly, and every configured authority has a
  reviewed hard ceiling.
- The release pentest is recorded, all eight findings are remediated, and the
  clean retest passes before the final report-only commit and tag.
- `v0.53.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.54.0 - Metered Precompile Outcome Contract

Status: implementation complete; pentest findings remediated; clean retest
passed; awaiting green GitHub CI and CodeQL before tagging.

Goal: make precompile charging, validation, execution, failure, and output
semantics impossible to bypass or confuse at CALL integration.

Deliverables:

- Crate-private non-forgeable `PaidPrecompile<K>` authorization produced only
  after a valid gas quote and work/output admission;
- `GasQuote` semantics bound to exact input or its digest to prevent
  time-of-check/time-of-use substitution;
- `PrecompileOutcome` covering success, call failure, gas consumed, and output;
- fail-closed paid-capability abandonment and atomic authorize-and-execute
  entry points;
- cheap framing separated from metered BN254/BLS curve and subgroup work;
- backend contracts for maximum work and output, with gas-derived input bounds
  instead of a consensus-invalid global byte ceiling.

Verification:

- Forgery/TOCTOU compile-fail and mutation tests;
- external raw-authorization, armed-type naming, and `mem::forget` bypass
  compile-fail tests;
- CALL rollback and all-supplied-gas failure matrices;
- release-blocking adversarial work-per-gas ceilings for every variable-work
  native precompile;
- output-unchanged tests for failures before execution authorization.

Exit criteria:

- No expensive precompile work runs without charged authorization, and CALL
  semantics receive one precise must-use outcome without repeating work;
- dropping or unwinding through paid authority consumes the complete dedicated
  child meter, and benchmark regressions fail the gate.
- `v0.54.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.55.0 - Consensus-Complete ModExp

Status: tagged and published after pentest, clean retest, GitHub CI, and CodeQL
passed.

Goal: replace the 64-byte operand subset with consensus-compatible EIP-198 and
EIP-2565 execution bounded by protocol gas and available memory.

Deliverables:

- Arbitrary declared base, exponent, and modulus lengths admitted whenever the
  protocol gas and memory rules permit them;
- keep all declared 256-bit lengths wide through gas calculation and convert
  to `usize` only after charged bounds prove the conversion representable;
- virtual right-padding and streamed reads avoid allocating attacker-declared
  absent payload bytes;
- bounded big-integer workspace or streaming arithmetic whose limits derive
  from the transaction/block gas envelope;
- exact adjusted-exponent-length, gas, zero modulus, zero-length, truncation,
  padding, and output rules;
- no fixed implementation ceiling that rejects a protocol-valid call.

Verification:

- Official and execution-spec ModExp vectors above and below 64 bytes;
- differential tests against Geth, Besu, Nethermind, and an independent
  arithmetic oracle;
- adversarial length/gas fuzzing and cycles-per-gas benchmarks;
- memory, stack, cancellation, and output-limit evidence.

Exit criteria:

- Every ModExp call allowed by the claimed fork is either executed correctly
  or fails only for an Ethereum-defined reason, never a private 64-byte cap.
- `v0.55.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 8B: Advanced BLS12-381 Precompiles

### v0.56.0 - BLS12-381 Base Field

Status: tagged as v0.56.0 after clean retest and GitHub approval.
The maintainer-approved workflow separates portable implementation checks from
final report/tag admission. The host-unavailable external-client run remains
documented missing evidence, not an additional tag blocker for this field-only
milestone. Internal tag only; publication at v0.60.0.

Goal: establish canonical first-party Fp arithmetic independently of curve execution.

Scope: implementation pass. Depends on v0.55.0. The exact APIs, resource bounds
and commands are in [BLS12-381 base field](bls12-381-base-field.md). The retained
workstream contract at v0.58.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement Fp conversion, add/subtract, multiply/square, reduction, inversion and square root with fixed-width scratch and explicit public-input timing policy.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official field constants, independent big-integer oracle, boundary residues, inversion/sqrt postconditions and fixed-work benchmarks.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Canonical field results match an independent oracle; no G1 precompile is enabled yet.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.56.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.57.0 - BLS12-381 G1 Group Operations

Status: tagged as v0.57.0 after clean pentest and GitHub approval.
Internal signed tag; publication at v0.60.0.

Goal: build complete G1 arithmetic on the admitted field.

Scope: implementation pass. Depends on v0.56.0. Exact APIs, resources,
fixture provenance and commands are in [G1 arithmetic](bls12-g1-arithmetic.md). The retained
workstream contract at v0.58.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement affine/projective conversion, infinity, negation, addition/doubling and on-curve parsing; keep subgroup membership separate.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent point vectors, equal/inverse/infinity cases, malformed coordinates and group-law properties.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- All G1 exceptional cases pass before charged precompile integration.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.57.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.58.0 - BLS12-381 G1 Arithmetic And Addition Completion

Status: tagged as v0.58.0 after clean pentest and GitHub checks.
Internal signed tag; publication at v0.60.0.

Goal: implement dependency-free G1 field arithmetic and the `0x0b` addition
precompile with official positive, infinity, invalid-field, and invalid-curve
vectors.

Scope: completion and integration pass. Depends on v0.57.0.
The implementation passes v0.56.0 through v0.57.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
charged G1 addition dispatch, exact gas and atomic canonical output.
See [the integration contract](bls12-g1-addition.md) for exact admission versus
terminal CALL failure behavior, evidence commands and public-input bounds.

Deliverables:

- fixed-width Fp add, subtract, multiply, square, inversion, and square-root
  operations with checked domain conversion;
- complete G1 affine/projective conversion, negation, doubling, and addition
  formulas covering infinity, equal points, and inverse points;
- curve-membership validation separated from subgroup validation;
- charged `0x0b` execution over exactly two G1 points with canonical 128-byte
  output and no subgroup rejection, as required by EIP-2537;
- output-unchanged and no curve-work evidence for wrong-kind, wrong-length
  and out-of-gas failures; malformed points perform only bounded validation
  after payment, never group addition or normalization.

Verification:

- official EIP-2537 G1 addition vectors and generator constants;
- identity, inverse, doubling, commutativity, and associativity property tests;
- differential tests against an independently reviewed BLS12-381 reference
  used only as a development oracle;
- fuzz coverage for G1 decoding and charged addition execution;
- release-mode fixed-gas CPU evidence;
- pentest and retest before tagging.

Exit criteria:

- Address `0x0b` produces only canonical EIP-2537 G1 results after exact gas
  charging, and does not reject valid on-curve points solely for subgroup
  membership.
- `v0.58.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.59.0 - BLS12-381 Quadratic Extension Field

Status: implementation candidate; pentest clean, GitHub/tag approval pending.
Internal signed tag after approval; publication at v0.60.0.

Goal: freeze Fp2 coefficient and non-residue conventions before G2 formulas.

Scope: implementation pass. Depends on v0.58.0. The retained
workstream contract at v0.61.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement Fp2 arithmetic, conjugation, inversion and square root without enabling G2 execution.
- Freeze `v^2=-1`, canonical `c0 || c1` encoding, smaller-wire-root selection,
  zero/nonsquare behavior and public-input-only resource bounds in the
  [Fp2 contract](bls12-fp2-arithmetic.md).
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent extension-field vectors and zero/nonresidue/inversion properties.
- `cargo test -p eth-valkyoth-evm-core --test bls12_fp2_differential`,
  `cargo +nightly fuzz run bls12381_fp2 -- -max_total_time=30 -max_len=193`,
  fixed-work `bls12_fp2_benchmark` and `scripts/release_0_59_0_gate.sh --implementation`.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Fp2 conventions are tested and documented before G2 integration.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.59.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.60.0 - BLS12-381 G2 Group Operations

Status: planned; public crates.io checkpoint after cumulative review.

Goal: complete G2 point formulas before charged addition.

Scope: implementation pass. Depends on v0.59.0. The retained
workstream contract at v0.61.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement G2 affine/projective conversions, complete addition/doubling and curve validation without subgroup-only rejection.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent G2 vectors, infinity/equal/inverse cases and malformed coefficient ordering.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- G2 arithmetic is vector-backed before the precompile dispatcher consumes it.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.60.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.61.0 - BLS12-381 Fp2, G2, And Addition Completion

Status: planned; internal signed tag, publication at v0.65.0.

Goal: implement dependency-free Fp2/G2 arithmetic and the `0x0d` addition
precompile, then establish the extension-tower foundation required by pairing.

Scope: completion and integration pass. Depends on v0.60.0.
The implementation passes v0.59.0 through v0.60.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
charged G2 addition dispatch, exact gas and canonical output.

Deliverables:

- Fp2 add, subtract, multiply, square, conjugate, inverse, and square-root
  operations using the EIP-2537 non-residue and coefficient order;
- complete G2 affine/projective conversion, negation, doubling, and addition;
- G2 curve-membership validation that remains separate from subgroup checks;
- charged `0x0d` execution over exactly two G2 points with canonical 256-byte
  output and no subgroup rejection;
- non-forgeable gas/work authorization before remotely reachable Fp2/G2
  arithmetic, including inversion or square-root work, preserving the v0.59.0
  public-input-only contract;
- explicit extension-tower conventions reused by later pairing releases.

Verification:

- official EIP-2537 G2 addition vectors and generator constants;
- cross-client G2 vectors against pinned Geth, Besu and Nethermind versions;
  instrumented regressions proving failed admission performs no inversion or
  square-root work and paid failures preserve the output/gas contract;
- Fp2 field identities plus G2 identity, inverse, doubling, commutativity, and
  associativity properties;
- independent differential vectors for field and point arithmetic;
- malformed-coordinate, infinity, wrong-length, out-of-gas, and output-buffer
  tests;
- fuzz and pentest gates before tagging.

Exit criteria:

- Address `0x0d` is executable with canonical EIP-2537 behavior, while valid
  on-curve non-subgroup points remain accepted by addition as required.
- The Fp2/G2 conventions needed by pairing are documented and vector-backed.
- `v0.61.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.62.0 - BLS12-381 Subgroup Validation

Status: planned; internal signed tag, publication at v0.65.0.

Goal: admit bounded first-party G1/G2 subgroup checks for MSM and pairing
inputs without incorrectly adding subgroup rejection to the addition APIs.

Scope: bounded milestone. Depends on v0.61.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- straightforward subgroup-order multiplication checks retained as a test
  oracle;
- reviewed bounded optimized G1 and G2 subgroup checks suitable for public
  precompile input;
- distinct validated-point domain tokens that cannot be constructed by mere
  curve-membership parsing;
- mandatory subgroup validation in MSM and pairing preparation paths;
- tests proving `0x0b` and `0x0d` addition still do not apply subgroup checks;
- fail-before-arithmetic behavior after the plan's gas charge for invalid
  subgroup inputs.

Verification:

- known on-curve points both inside and outside the prime-order subgroups;
- optimized checks compared with subgroup-order multiplication over generated
  and fixture points;
- differential checks against an independent implementation;
- fuzz coverage for curve-valid non-subgroup inputs;
- fixed-iteration and CPU-bound evidence;
- pentest and retest before tagging.

Exit criteria:

- No MSM or pairing path can consume an unvalidated subgroup point, and no
  addition path rejects a point for a subgroup rule that EIP-2537 omits.
- `v0.62.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.63.0 - BLS12-381 Multiscalar Multiplication

Status: planned; internal signed tag, publication at v0.65.0.

Goal: implement `0x0c` and `0x0e` with bounded Pippenger-style execution,
official discount gas, item limits, vectors, differential tests, and CPU/gas
evidence.

Scope: bounded milestone. Depends on v0.62.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- complete 256-bit scalar multiplication, including scalars not reduced in the
  wire format and the required arithmetic reduction behavior;
- bounded no-allocator G1 and G2 MSM execution with reviewed window and bucket
  limits;
- charged `0x0c` and `0x0e` execution for non-empty complete item lists;
- mandatory point curve/subgroup validation before MSM arithmetic;
- reuse of the independently checked 128-entry discount tables and capped
  `k > 128` pricing;
- deterministic scratch-space ceilings and explicit maximum item counts.

Verification:

- official EIP-2537 MSM vectors and required group properties;
- naive repeated-multiplication oracle for small item counts;
- independent differential tests for G1 and G2 results;
- gas tests for every discount entry plus `k = 129` and maximum input limits;
- fuzzing for item segmentation, scalars, subgroup rejection, and output
  invariants;
- release-mode CPU/gas evidence across representative and worst-case counts;
- pentest and retest before tagging.

Exit criteria:

- Addresses `0x0c` and `0x0e` return canonical subgroup points for every
  admitted non-empty frame, with execution cost and scratch use bounded by the
  same input count used for gas planning.
- `v0.63.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.64.0 - BLS12-381 Map-To-Curve

Status: planned; internal signed tag, publication at v0.65.0.

Goal: implement the EIP-2537 Fp-to-G1 and Fp2-to-G2 mappings at `0x10` and
`0x11` from the pinned mapping specification and official vectors.

Scope: bounded milestone. Depends on v0.63.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- pinned simplified-SWU and isogeny-map parameter source with checksum and
  provenance;
- first-party Fp-to-G1 and Fp2-to-G2 mapping algorithms with fixed iteration
  bounds and explicit sign/square-root conventions;
- canonical input-field validation before mapping;
- charged `0x10` and `0x11` execution with canonical point output;
- post-map curve and subgroup assertions used as internal fault detection;
- no byte-string hash-to-curve claim: these APIs map already-decoded field
  elements exactly as scoped by EIP-2537.

Verification:

- official EIP-2537 mapping vectors and pinned mapping-spec vectors;
- independent differential vectors for both maps;
- property tests proving every output is on-curve and in the correct subgroup;
- malformed-field, out-of-gas, output-buffer, and fuzz coverage;
- release-mode CPU/gas evidence;
- pentest and retest before tagging.

Exit criteria:

- Addresses `0x10` and `0x11` are executable and vector-backed, with no
  ambiguity between field-to-curve mapping and a higher-level hash-to-curve
  protocol.
- `v0.64.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.65.0 - BLS12-381 Pairing Extension Tower

Status: planned; public crates.io checkpoint after cumulative review.

Goal: establish Fp6/Fp12 arithmetic independently of Miller iteration.

Scope: implementation pass. Depends on v0.64.0. The retained
workstream contract at v0.68.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement extension arithmetic, Frobenius maps and canonical coefficient ordering using the admitted Fp2 substrate.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent tower vectors, sparse/dense multiplication equivalence and inversion properties.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- The pairing tower has an external oracle and bounded scratch use.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.65.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.66.0 - BLS12-381 Miller Loop

Status: planned; internal signed tag, publication at v0.70.0.

Goal: implement the pairing line and Miller computation without exposing verification.

Scope: implementation pass. Depends on v0.65.0. The retained
workstream contract at v0.68.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement line evaluation, loop schedule and accumulator handling; keep production pairing fail closed.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent Miller vectors, infinity/line exceptions and loop-work bounds.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Miller outputs match the reference; final exponentiation remains a distinct step.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.66.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.67.0 - BLS12-381 Final Exponentiation

Status: planned; internal signed tag, publication at v0.70.0.

Goal: complete the reduced pairing arithmetic before charging integration.

Scope: implementation pass. Depends on v0.66.0. The retained
workstream contract at v0.68.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement easy/hard exponentiation stages and identity testing on admitted Miller results.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent exponentiation vectors, bilinearity and inverse-pair batches plus CPU/stack measurements.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Reduced pairings match independent results before the precompile is enabled.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.67.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.68.0 - BLS12-381 Pairing Foundation Completion

Status: planned; internal signed tag, publication at v0.70.0.

Goal: add the first-party Fp6/Fp12 tower, line functions, Miller loop, and
bounded final-exponentiation foundation while pairing remains fail closed.

Scope: completion and integration pass. Depends on v0.67.0.
The implementation passes v0.65.0 through v0.67.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
composition of tower, Miller and final-exponentiation results; production pairing stays separately gated.

Deliverables:

- Fp6/Fp12 tower arithmetic using one documented coefficient and twist
  convention;
- independently derived Frobenius coefficients with checked fixture copies;
- sparse line multiplication and the fixed negative BLS parameter Miller loop;
- bounded easy/hard final-exponentiation chain;
- tuple streaming that does not allocate or reparse validated G1/G2 points;
- fail-closed `0x0f` execution after exercising the complete internal
  accumulator, without admitting a public boolean result.

Verification:

- extension-field identity, inversion, Frobenius-cycle, and exponentiation
  tests;
- line-function, Miller-loop, and final-exponentiation vectors from an
  independent reference;
- bilinearity and inverse-pair properties over validated subgroup points;
- fuzz coverage for bounded tuple accumulation;
- release-mode pairing CPU/gas measurements;
- pentest and retest before tagging.

Exit criteria:

- Pairing arithmetic is complete, deterministic, fixed-bound, and
  independently vector-backed, but non-empty `0x0f` still returns backend
  unavailable until result admission is separately reviewed.
- `v0.68.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.69.0 - BLS12-381 Pairing Execution

Status: planned; internal signed tag, publication at v0.70.0.

Goal: admit non-empty `0x0f` pairing execution with canonical zero/one output,
subgroup enforcement, official vectors, differential checks, and gas/CPU
evidence.

Scope: bounded milestone. Depends on v0.68.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- charged non-empty pairing execution over complete 384-byte tuples;
- mandatory G1/G2 curve and subgroup validation for every tuple;
- canonical 32-byte false/true output words and no alternate success values;
- explicit precompile-error outcome contract so the future CALL dispatcher can
  burn all supplied call gas as required by EIP-2537;
- output-unchanged behavior on charge failure and malformed input;
- bounded batch accumulation reusing the `v0.68.0` arithmetic exactly once per
  tuple.

Verification:

- official positive, negative, infinity, malformed, and non-subgroup vectors;
- independent differential checks for single and multi-pair equations;
- canonical-output and repeated-charge tests;
- fuzzing that permits only a 32-byte zero/one result on success;
- worst-case tuple-count CPU/gas and stack-use evidence;
- pentest and retest before tagging.

Exit criteria:

- Address `0x0f` returns a consensus-compatible canonical result for every
  admitted frame, and exposes enough error classification for dispatcher-level
  all-gas burning without repeating cryptographic work.
- `v0.69.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.70.0 - Prague Advanced-Precompile Admission

Status: planned; public crates.io checkpoint after cumulative review.

Goal: run the complete official EIP-2537 fixture set, fuzz and pentest every
advanced precompile path, and admit only the Prague claims backed by evidence.

Scope: bounded milestone. Depends on v0.69.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- complete `0x0b..=0x11` registry-to-execution dispatch coverage for Prague;
- generated conformance matrix covering frame shape, field/point validation,
  subgroup policy, gas, output, and error behavior for all seven precompiles;
- explicit integration policy for EIP-2537 all-supplied-gas burning on errors;
- official fixture lock and drift checker;
- cross-client differential report and performance evidence;
- fuzz corpus promotion for every security-relevant failure class;
- final dependency-independence, memory, stack, panic, and unsafe-code audit;
- independent pentest report covering the entire BLS sequence.

Verification:

- complete pinned EIP-2537 vectors and property suite;
- all advanced-precompile fuzz targets and seed materialization checks;
- differential comparison against at least two independent client/reference
  implementations where practical;
- `scripts/checks.sh`, `cargo deny check`, `cargo audit`, and current SBOM;
- release-mode CPU/gas report for every operation;
- pentest, remediation, and clean retest before tagging.

Exit criteria:

- The support matrix claims only library-level Prague precompile behavior that
  is backed by official vectors, differential evidence, bounded resource use,
  and a clean pentest.
- Any remaining CALL-dispatch or state-transition integration is assigned to a
  named later release rather than implied by this admission.
- `v0.70.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.71.0 - Architecture Invariant Gate

Status: planned; internal signed tag, publication at v0.75.0.

Goal: freeze the security-critical dependency direction and privilege
boundaries before the SDK, storage, networking, and client surface expands.

Scope: bounded milestone. Depends on v0.70.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Machine-checked crate-layer and feature-power-set rules;
- a first-party and transitive unsafe-code inventory with explicit adapter
  exceptions and owners;
- consensus-kernel versus orchestration/API boundary tests;
- public API and semver budgets for every independently published crate;
- evidence slots, reservation trees/arenas, collection cardinalities, sink
  adapters, and internal context lifetimes remain private to validation
  services wherever possible; ordinary SDK users consume a stable
  `ValidationOutcome<T, E>`-style result rather than becoming generic over
  evidence machinery;
- feature combinations cannot change consensus validity, validation return
  types, evidence authority, or collection defaults, and optional sinks cannot
  pull `std`, allocation, async, or runtime requirements into the `no_std`
  kernel;
- an architecture-wide cache identity rule requiring chain/genesis, fork
  rules, snapshot/root, object key, and validation level in every
  security-relevant cache key;
- separate cache domains for untrusted, quorum-derived, and cryptographically
  verified values, with atomic invalidation or rebinding on canonical-head and
  fork-rule changes;
- an architecture rule that local resource, timeout, cancellation, dependency,
  backend, storage, and internal failures can never be promoted into
  `ObjectInvalidityEvidence` or object-negative-cache authority;
- an architecture decision stating that the dependency-free `no_std`
  consensus kernel may use optional audited platform adapters, while those
  adapters cannot decide Ethereum validity.

Verification:

- CI architecture tests over all feature combinations and dependency graphs;
- unsafe inventory and exception review;
- compile-fail tests for forbidden dependency and capability directions;
- public API surface snapshots proving internal evidence capabilities,
  lifetimes, sink types, and arbitrary cardinality generics do not leak.

Exit criteria:

- Later convenience, runtime, database, network, or hardware integrations
  cannot acquire consensus authority through dependency or feature drift.
- `v0.71.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.72.0 - Hierarchical Resource Governor Contract

Status: planned; internal signed tag, publication at v0.75.0.

Goal: define one composable resource-governance model before node-facing
layers invent incompatible local limits.

Scope: bounded milestone. Depends on v0.71.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Hierarchical capabilities for node, connection, peer, protocol, request,
  decode session, execution, proof, and background-task scopes;
- distinct monotonic consumed-work units for bytes scanned, hashes, signature
  checks, database reads, and deterministic validation work, which are never
  refundable;
- distinct reservations for queue slots, memory capacity, and worker permits,
  refundable only on release, plus rate resources for bandwidth, request
  frequency, and CPU scheduling quotas replenished only by explicit policy;
- evidence arenas reserve memory and simultaneous-validation worker capacity
  through these capabilities before construction; attacker-controlled object,
  transaction, proof, cell, or message counts cannot directly size an arena;
- child-token conservation, cross-thread transfer, partial consumption,
  cancellation, timeout, double-release prevention, and fail-closed exhaustion
  semantics;
- prohibit wall-clock CPU time as a consensus-admission input; deterministic
  work units govern validity while wall time governs scheduling and peer policy;
- resource exhaustion, cancellation, timeout, and unavailable schedulers map
  to retryable local outcomes defined at `v0.93.0`, never protocol invalidity;
- `no_std` accounting traits plus optional runtime schedulers that cannot mint
  or bypass consensus work authorization.

Verification:

- Conservation and non-forgeability property tests for every resource class;
- nested cancellation/refund, retry amplification, cross-thread transfer,
  partial-consumption, concurrent exhaustion, and double-release simulations;
- evidence-arena memory/worker reservation, serialization/backpressure, local-
  exhaustion, cancellation, and transfer-with-live-borrow simulations;
- Kani-ready bounded state-machine harnesses and complexity-oracle adapters.

Exit criteria:

- Every later untrusted boundary can consume a shared hierarchical budget
  instead of resetting an unrelated local counter.
- `v0.72.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.73.0 - Cryptographic Substrate Contract Freeze

Status: planned; internal signed tag, publication at v0.75.0.

Goal: stabilize the cryptographic provider contracts before wallets,
networking, storage proofs, KZG, BLS, or validator duties depend on backend
details.

Scope: bounded milestone. Depends on v0.72.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Separate public-input and secret-bearing backend contracts for Keccak-256,
  secp256k1, KZG, BLS, AES-CTR, HMAC-SHA256, RLPx ECIES/KDF behavior,
  keystore KDF/cipher behavior, entropy, and authenticated transport primitives;
- explicit conformance, fixed-work/constant-time, zeroization, trusted-setup,
  maximum-work/output, error, and acceleration requirements per domain;
- backend admission and replacement rules that preserve first-party consensus
  semantics while allowing reviewed optional acceleration;
- a secret-taint register for private keys, nonces, scalars, seeds, KDF state,
  and conversion scratch space.

Verification:

- Backend conformance mocks and known-answer suites;
- compile-fail capability-separation tests;
- dependency, side-channel, sanitization, and failure-mode review.

Exit criteria:

- Higher layers depend on stable auditable cryptographic capabilities, not a
  concrete third-party implementation or an impossible absolute-erasure
  promise.
- `v0.73.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.74.0 - First-Party Keccak-256 Core

Status: planned; internal signed tag, publication at v0.75.0.

Goal: admit the first-party Ethereum Keccak-256 implementation before any
wallet, network, proof, or execution consumer can make a production claim.

Scope: bounded milestone. Depends on v0.73.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Dependency-free Keccak-f[1600] and Keccak-256 absorb/finalize, streaming,
  fixed-output, `no_std`, no-allocation implementation;
- compatibility with the frozen `v0.73.0` provider contract;
- explicit separation from SHA3-256 and reviewed state-clearing behavior for
  secret-bearing callers;
- external implementations retained only as optional differential/reference
  adapters.

Verification:

- Official and independent KATs including the empty-input Keccak/SHA3
  discriminator;
- permutation-round differential tests, fuzzing, endian/target matrix, stack,
  panic, timing, and sanitization review;
- dual-backend differentials for every existing transaction, proof, and EVM
  hashing consumer.

Exit criteria:

- Every existing production hashing path can select a first-party Keccak-256
  backend before higher-level consumers are implemented.
- `v0.74.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.75.0 - secp256k1 Field And Scalar Arithmetic

Status: planned; public crates.io checkpoint after cumulative review.

Goal: separate secret-capable field/scalar arithmetic from point formulas.

Scope: implementation pass. Depends on v0.74.0. The retained
workstream contract at v0.77.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement canonical field/scalar encodings and fixed-work limb operations with distinct modulus types.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent arithmetic vectors, carry/reduction boundaries, secret-taint and timing review.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Field/scalar substitution is rejected and arithmetic has independent evidence.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.75.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.76.0 - secp256k1 Point And Scalar Multiplication

Status: planned; internal signed tag, publication at v0.80.0.

Goal: admit secret-capable point operations before ECDSA.

Scope: implementation pass. Depends on v0.75.0. The retained
workstream contract at v0.77.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement complete point operations, fixed-work secret multiplication and separate public-input acceleration.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Two independent point oracles, exceptional cases, assembly/timing and fault tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Secret-dependent branches/indexes and unreviewed exceptional cases block admission.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.76.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.77.0 - First-Party secp256k1 Arithmetic Completion

Status: planned; internal signed tag, publication at v0.80.0.

Goal: establish first-party field, scalar, and point arithmetic before local
signers, node identity, discovery, or RLPx are implemented.

Scope: completion and integration pass. Depends on v0.76.0.
The implementation passes v0.75.0 through v0.76.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
field/scalar/point API integration and final arithmetic admission.

Deliverables:

- Canonical secp256k1 field/scalar parsing and bounded arithmetic;
- point addition, doubling, multiplication, compression, decompression, and
  infinity policy;
- fixed-work reviewed secret-scalar paths separated from public-input paths;
- dependency-free `no_std` operation with bounded caller-owned workspaces.

Verification:

- SEC and independent vectors plus group-law and malformed-point properties;
- differential checks against at least two independent implementations;
- subgroup, fault, side-channel, stack, panic, and fuzz review.

Exit criteria:

- Ethereum secp256k1 domains have a first-party arithmetic foundation before
  any production signer or network identity depends on them.
- `v0.77.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.78.0 - HMAC-SHA256 And Deterministic Nonces

Status: planned; internal signed tag, publication at v0.80.0.

Goal: admit the nonce derivation kernel separately from signature production.

Scope: implementation pass. Depends on v0.77.0. The retained
workstream contract at v0.81.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement HMAC-SHA256, RFC 6979 conversion/retry and isolated wiped nonce state; do not export raw nonce material.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- HMAC/RFC vectors, message-to-scalar limits, retries and interleaved-key state isolation.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A nonce request cannot reuse another key/request state or escape after failure.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.78.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.79.0 - ECDSA Signing And Verification

Status: planned; internal signed tag, publication at v0.80.0.

Goal: produce and verify canonical signatures using admitted arithmetic and nonce generation.

Scope: implementation pass. Depends on v0.78.0. The retained
workstream contract at v0.81.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement r/s creation, low-s normalization, strict verification, signature domains and pre-release fault checks.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Two reference implementations, malformed signatures, zero scalars, fault injection and timing tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No signature escapes a detected nonce/arithmetic fault.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.79.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.80.0 - secp256k1 Recovery And ECDH

Status: planned; public crates.io checkpoint after cumulative review.

Goal: separate public-key recovery and transport agreement from signing authority.

Scope: implementation pass. Depends on v0.79.0. The retained
workstream contract at v0.81.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement recovery IDs, public-key recovery and role-bound ECDH, with validated peer points and wiped shared secrets.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- ECRECOVER and transport vectors, invalid/infinity peer keys, domain-substitution and output-failure tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Recovery and ECDH match independent implementations without granting transport keys signing authority.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.80.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.81.0 - First-Party HMAC ECDSA Recovery And ECDH Completion

Status: planned; internal signed tag, publication at v0.85.0.

Goal: complete the first-party HMAC-SHA256 and secp256k1 operations required by
deterministic signing, transactions, ECRECOVER, node identity, discovery, and
RLPx in dependency order.

Scope: completion and integration pass. Depends on v0.80.0.
The implementation passes v0.78.0 through v0.80.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
existing signer/recovery provider adapters and cross-domain integration.

Deliverables:

- First-party HMAC-SHA256 compatible with the frozen provider contract;
- RFC 6979-compatible deterministic ECDSA nonce generation with exact
  message-to-scalar handling;
- bounded retry for zero or out-of-range nonce, `r`, or `s`, and optional
  additional entropy only through a reviewed domain-separated input;
- strict verification, low-s normalization, recoverable signatures,
  public-key recovery, and ECDH;
- distinct execution transaction, message, EIP-712, authorization, and
  networking-identity domains; consensus validator duties are explicitly
  excluded because they require BLS12-381;
- no nonce/HMAC state reuse across keys or requests;
- sanitation of HMAC state, nonce candidates, intermediate scalars, and keys;
- fault detection prevents any signature from escaping after an arithmetic,
  nonce-generation, or consistency fault;
- compatibility adapters for existing signer and recovery provider traits.

Verification:

- Ethereum transaction, ECRECOVER, ENR, discovery, and RLPx vectors;
- two-implementation differential checks and dual-backend consumer tests;
- RFC 6979 vectors, message-to-scalar boundaries, retry-path, additional-
  entropy domain, nonce-state isolation, invalid-recovery, timing, fault,
  escaped-signature, and fuzz tests.

Exit criteria:

- All Ethereum-required secp256k1 operations have a first-party path before
  wallet and networking implementation begins.
- `v0.81.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.82.0 - AES Block And CTR Substrate

Status: planned; internal signed tag, publication at v0.85.0.

Goal: provide reviewed AES and counter-mode primitives for Ethereum consumers.

Scope: implementation pass. Depends on v0.81.0. The retained
workstream contract at v0.86.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement or admit an isolated audited AES provider, exact counter/IV rules, key ownership and non-exporting state.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- AES/CTR KATs, counter exhaustion, aliasing, wiping and backend-equivalence tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- AES/CTR is admitted independently of protocol handshakes.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.82.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.83.0 - AES-GCM And Discovery Key Schedule

Status: planned; internal signed tag, publication at v0.85.0.

Goal: supply the authenticated encryption required by discovery.

Scope: implementation pass. Depends on v0.82.0. The retained
workstream contract at v0.86.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement or admit GHASH/AES-GCM and HKDF-SHA256 with authenticated associated data and distinct discovery keys.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- AEAD/HKDF and discv5 vectors, forged tags, nonce misuse, truncated messages and unchanged-output failures.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Unauthenticated plaintext cannot leave the API and discovery has no unnamed AEAD dependency.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.83.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.84.0 - SHA-512 HMAC And Password KDFs

Status: planned; internal signed tag, publication at v0.85.0.

Goal: make wallet and keystore derivation dependencies explicit before their consumers.

Scope: implementation pass. Depends on v0.83.0. The retained
workstream contract at v0.86.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement or admit SHA-512/HMAC-SHA512, PBKDF2 and scrypt in isolated reviewed boundaries; meter password work and wipe scratch.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent KDF vectors, overflow/parameter-DoS, Unicode-input boundary contracts and allocation/cancellation failure tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- BIP-39/BIP-32 and keystore consumers have admitted primitives, not implicit later dependencies.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.84.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.85.0 - RLPx ECIES Transcript Cryptography

Status: planned; public crates.io checkpoint after cumulative review.

Goal: bind admitted symmetric and ECDH primitives to the RLPx transcript.

Scope: implementation pass. Depends on v0.84.0. The retained
workstream contract at v0.86.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement ECIES/KDF/MAC and session key derivation with protocol-specific key/nonce domains; no live sockets yet.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official handshake vectors, transcript substitution, MAC failure and replay cases.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Transcript outputs interoperate without leaking keys or admitting unauthenticated sessions.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.85.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.86.0 - Symmetric Transport And Keystore Cryptography Completion

Status: planned; internal signed tag, publication at v0.90.0.

Goal: provide or explicitly admit every non-TLS cryptographic primitive needed
by RLPx and Web3 keystores before those consumers are built.

Scope: completion and integration pass. Depends on v0.85.0.
The implementation passes v0.82.0 through v0.85.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
keystore parameter/password contracts and complete primitive-to-consumer admission.

Deliverables:

- First-party or separately audited AES-CTR capability and reuse of the
  admitted HMAC-SHA256 implementation from `v0.81.0`;
- exact RLPx ECIES, KDF, MAC, key schedule, and transcript behavior;
- Web3 keystore cipher, MAC, PBKDF/scrypt parameter, and password-handling
  contracts;
- explicit audited-adapter boundaries for TLS, QUIC, Noise, OS entropy, and
  hardware acceleration, none of which can decide Ethereum validity;
- bounded work, key separation, nonce/IV policy, error, and zeroization rules.

Verification:

- Published RLPx, Web3 keystore, AES, HMAC, ECIES, and KDF vectors;
- malformed, downgrade, parameter-DoS, nonce/IV reuse, oracle, and
  interoperability tests;
- dependency, side-channel, secret-taint, and backend-admission review.

Exit criteria:

- RLPx and keystore milestones have no unnamed cryptographic dependency or
  ambiguous trust boundary.
- `v0.86.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.87.0 - Early Core Cryptography Integration And Audit

Status: planned; internal signed tag, publication at v0.90.0.

Goal: make the first-party Keccak/secp path the default production-capable core
before owned SDK, wallets, providers, and networking expand.

Scope: bounded milestone. Depends on v0.86.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Integrate hashing, transaction signing/recovery, EVM `KECCAK256`,
  ECRECOVER, CREATE address derivation, proof hashing, ENR, discovery, and RLPx
  provider seams available at this point;
- retain external implementations only behind explicit reference or optional
  acceleration features;
- audit domain separation, constant-time boundaries, zeroization, feature
  graphs, errors, and backend replacement;
- publish a first-party core-cryptography conformance report.

Verification:

- Workspace dual-backend differential matrix;
- dependency graph proving no external Keccak/secp implementation is required
  by production core paths;
- independent cryptography audit, remediation, and clean retest.

Exit criteria:

- Later signers and networking use an already admitted first-party core rather
  than being migrated from a temporary backend near release.
- `v0.87.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.88.0 - Cross-Format Decode Work Accounting

Status: planned; internal signed tag, publication at v0.90.0.

Goal: extend the shared decode/session ledger from RLP and MPT into every later
untrusted serialization and compression boundary.

Scope: bounded milestone. Depends on v0.87.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Reusable child-ledger contracts for SSZ, JSON, ABI, Snappy, SSZ-Snappy,
  Req/Resp chunks, and GossipSub messages;
- deterministic accounting for compressed/decompressed bytes, ratio,
  structural nodes, depth, offsets, strings, list/bitlist elements, chunks,
  hashes, Merkleization, allocation capacity, and output;
- reject before decompression, allocation, hashing, or ownership conversion
  when the relevant reservation/work budget cannot be acquired;
- JSON duplicate-key rejection and bounded structural depth/node count;
- format-specific releases must prove they consume this parent ledger rather
  than creating fresh local budgets.

Verification:

- Cross-format conservation and nested-child property tests;
- compression-bomb, duplicate-key, offset, list, bitlist, chunk, and output
  complexity-oracle fixtures;
- adapters that deliberately reset accounting are rejected by conformance tests.

Exit criteria:

- Every planned wire format has one defined route into operation-wide resource
  accounting before its parser is implemented.
- `v0.88.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.89.0 - Shared Clock And Time-Evidence Contract

Status: planned; internal signed tag, publication at v0.90.0.

Goal: define one explicit time capability for networking, Engine, consensus,
validator, and operational scheduling before those layers diverge.

Scope: bounded milestone. Depends on v0.88.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Separate monotonic time for deadlines/cancellation from wall/UTC time for
  slots, epochs, timestamps, and operator display;
- monotonic time governs peer rate windows, retries, and in-process sanctions;
  every persisted observation carries an explicit boot/session identity;
- UTC is permitted for operator display and explicitly defined persisted
  expiry only, never as an in-process elapsed-time source;
- clock rollback, stale external time, restart, or session mismatch cannot
  extend a ban, revive expired evidence, or replay an old observation window;
- detected step/slew, stale-time, unavailable-time, uncertainty, and source
  evidence states;
- test/deterministic clock implementations and bounded external-source adapters;
- external time evidence may inform policy but cannot directly override
  validator slashing or consensus safety decisions.

Verification:

- Backward/forward step, slew, pause, stale source, disagreement, and rollover
  simulations;
- restart/session-change, persisted-expiry, ban-extension, and expired-evidence
  replay simulations;
- compile-time separation of monotonic and UTC timestamp domains;
- deterministic network and validator scenario integration.

Exit criteria:

- No deadline, slot, peer, or signer safety path silently mixes monotonic and
  wall time or treats an external source as authority.
- `v0.89.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.90.0 - Validation Outcome And Minimal Evidence Kernel

Status: planned; public crates.io checkpoint after cumulative review.

Goal: make authoritative invalidity distinct from local inability to validate.

Scope: implementation pass. Depends on v0.89.0. The retained
workstream contract at v0.93.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement disjoint outcomes, fixed-size evidence, atomic pre-validation slot reservation, immutable object/context binding and infallible fill.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Compile-fail forging tests, slot exhaustion before work and allocation/fault injection on every outcome.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Local failures cannot manufacture invalidity and detected invalidity survives diagnostic failure.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.90.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.91.0 - Peer Attribution And Batch Invalidity

Status: planned; internal signed tag, publication at v0.95.0.

Goal: prevent object evidence and failed batches from becoming collective peer blame.

Scope: implementation pass. Depends on v0.90.0. The retained
workstream contract at v0.93.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement authenticated-delivery composition, wire/policy evidence and bounded individual isolation with no group penalty.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Mixed-source batches, stale delivery identities, repeated quotas and unknown-member tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Only identified evidence-bearing members can be penalized.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.91.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.92.0 - Invalidity Cache And Diagnostic Boundaries

Status: planned; internal signed tag, publication at v0.95.0.

Goal: preserve evidence authority through optional sinks.

Scope: implementation pass. Depends on v0.91.0. The retained
workstream contract at v0.93.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement bounded negative-cache identity/expiry, auxiliary-object scoping, redacted attachments and failure-independent immutable views.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Sibling poisoning, context changes, cache floods and sink serialization/persistence failures.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Caches and diagnostics neither erase immediate invalidity nor accuse unrelated objects.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.92.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.93.0 - Validation Outcomes, Object Invalidity And Peer Evidence Completion

Status: planned; internal signed tag, publication at v0.95.0.

Goal: ensure object invalidity, peer protocol violations, peer policy abuse,
and non-action outcomes have disjoint evidence and authority.

Scope: completion and integration pass. Depends on v0.92.0.
The implementation passes v0.90.0 through v0.92.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
outcome, attribution, cache and sink composition without a second evidence representation.

Deliverables:

- Shared `Valid`, `ProtocolInvalid { evidence: ObjectInvalidityEvidence }`,
  `MissingDependency`/
  `Syncing`, `LocalResourceExhausted`, `Cancelled`/`Stale`,
  `BackendUnavailable`, `InternalFault`, `PolicyRejected { policy_id }`,
  `UnsupportedCapability`, `Duplicate`/`AlreadyKnown`, and
  `DeferredUntil { dependency }` domains plus non-attributable
  `BatchContainsInvalid`;
- non-forgeable `ObjectInvalidityEvidence` bound to exact object bytes/hash,
  chain/genesis, fork rules, parent/root, validation version, stage, and reason;
- distinct `PeerProtocolViolationEvidence` bound to connection/peer identity,
  negotiated capabilities and protocol version, offending message,
  observation, and time window;
- distinct `PeerPolicyViolationEvidence` bound to connection/peer identity,
  policy version, announced quota/rule, observations, and time window;
- one non-copyable operation-wide `EvidenceBudget` covering entry bytes,
  observation count, diagnostic/path length, stored witness bytes,
  serialization/output bytes, and persistent retention reservations;
- a fixed-capacity `EvidenceSlot` is reserved atomically before any operation
  begins authoritative validation that may return `ProtocolInvalid`; inability
  to reserve returns `LocalResourceExhausted` before validation starts;
- once reserved, minimal `ObjectInvalidityEvidence` construction is infallible,
  allocation-free, and fixed-size, containing only object digest,
  context/rules digest, validation version, stage, stable reason code, and a
  bounded field/index location;
- compact evidence stores object and context/rules digests, stable reason code,
  bounded field/path location, and the minimum verification witness; full
  malformed objects require a separately reserved object store and are never
  retained implicitly as evidence;
- peer-policy evidence uses bounded counters and windows rather than event
  vectors, with time/session identity supplied by `v0.89.0`;
- diagnostic excerpts, detailed witnesses, traces, serialization,
  persistence, and cache insertion are optional attachments after minimal
  evidence exists; their exhaustion/failure is reported separately and cannot
  erase or alter the immediate `ProtocolInvalid` result;
- non-invalid validation paths release the reserved slot deterministically;
- public evidence diagnostics redact peer addresses, credentials, transaction
  privacy data, and secret-adjacent fields while preserving stable reason codes;
- only `ObjectInvalidityEvidence` may enter bad-block, bad-transaction,
  invalid-proof, or sidecar caches, produce Engine `INVALID`/
  `latestValidHash`, produce object-invalid GossipSub `REJECT`, or permanently
  reject a blockchain object;
- peer penalties, disconnects, and bans require `ObjectInvalidityEvidence`
  composed with a peer-bound authenticated delivery observation,
  `PeerProtocolViolationEvidence`, or `PeerPolicyViolationEvidence`; peer
  evidence can never prove that the transported blockchain object is invalid
  or enter an object-negative cache;
- negative entries bind exact object bytes/hash, chain/genesis, fork rules,
  parent/root, validation version, validation stage, and failure evidence;
- invalidate negative entries when any validation assumption changes, bound
  retention and capacity, and prevent unique-invalid-object cache flooding;
- missing trie nodes/state, resource exhaustion, timeout, cancellation, stale
  snapshots, unavailable cryptography, disk/backend errors, and internal faults
  remain local/retryable and preserve the original object's unknown status;
- local fee/tip/configuration refusal is `PolicyRejected`, locally unavailable
  methods/capabilities are `UnsupportedCapability`, seen objects are
  `Duplicate`/`AlreadyKnown`, and future nonces/missing sidecars are
  `DeferredUntil`; none creates object-invalidity evidence, while repeated
  quota abuse requires separate peer-policy evidence;
- a peer sending malformed or unnegotiated wire messages creates scoped
  `PeerProtocolViolationEvidence`, not `UnsupportedCapability` or object
  invalidity;
- a failed mixed-source cryptographic batch returns `BatchContainsInvalid`,
  proving only that the batch cannot be accepted, not which member or peer is
  responsible; bounded individual isolation must establish member-specific
  `ObjectInvalidityEvidence` before penalty or caching;
- budget, entropy, cancellation, or backend failure during batch isolation
  returns its corresponding local outcome and never penalizes all represented
  peers;
- auxiliary evidence identities cover blob sidecars by block/index/commitment/
  blob/proof, PeerDAS cells/columns by block root/coordinate/commitment/proof,
  KZG by setup digest/domain, Engine bundles by payload/blobs/requests/parent
  beacon root, and Snap ranges by snapshot/start/end/continuation/proof nodes;
- invalid auxiliary evidence is scoped to that exact object and cannot poison
  its containing block or sibling auxiliary objects;
- conversion and composition rules consumed by proof verification, block
  import, Engine, sync, txpool, gossip, and serving layers.

Verification:

- Focused pre-implementation review of `EvidenceSlot`,
  `ObjectInvalidityEvidence`, optional attachments, and side-effect failure
  ordering;
- Compile-fail tests preventing local outcomes and peer evidence from
  constructing object-invalidity evidence or entering object-negative caches;
- fault injection at every validation stage for memory, budget, timeout,
  cancellation, missing data, stale snapshot, crypto backend, disk, and
  internal failures;
- property tests proving only object evidence reaches permanent object
  rejection, object-negative caches, Engine `INVALID`, or object-invalid
  GossipSub `REJECT`, and only peer-bound evidence reaches peer sanctions;
- policy/unsupported/duplicate/deferred mapping tests for txpool, RPC, gossip,
  capability negotiation, and sidecar/future-nonce workflows;
- single-duplicate, duplicate-flood, low-tip, malformed-negotiated-message,
  unnegotiated-message, authenticated-delivery attribution, and repeated-rate-
  abuse evidence tests;
- mixed-source batch failure and bounded member-isolation tests proving no
  group penalty without member-specific evidence;
- auxiliary-object substitution and sibling/containing-block non-poisoning
  tests for blobs, PeerDAS, KZG, Engine bundles, and Snap ranges;
- negative-cache assumption-change, collision/substitution, expiry, eviction,
  and unique-invalid-object flood tests;
- maximum evidence entry, observation, path, witness, serialization, output,
  and retention boundary tests plus multi-megabyte malformed-object cases that
  retain only digest and bounded witness;
- pre-validation slot-reservation exhaustion tests proving no authoritative
  validation starts without minimal-evidence capacity;
- diagnostic, witness, allocation, serialization, persistence, and negative-
  cache insertion fault injection after invalidity is detected; the immediate
  minimal `ProtocolInvalid` result remains available while optional side
  effects fail separately;
- fixed-layout/no-allocation tests for every minimal reason/stage/location
  combination and deterministic slot release on valid/local outcomes;
- diagnostic snapshot tests proving required redaction and bounded output;
- model checking for outcome composition, retry, and evidence conservation.

Exit criteria:

- A constrained or faulty local node cannot label a valid object invalid or
  poison an object cache, peer sanctions cannot be created without peer-bound
  evidence, and evidence itself cannot become an unbounded storage, memory,
  serialization, logging, or privacy channel; once authoritative validation
  starts, enough capacity already exists to preserve any minimal invalid result.
- `v0.93.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.94.0 - Early secp256k1 Arithmetic Proof Gate

Status: planned; internal signed tag, publication at v0.95.0.

Goal: provide machine-checked evidence for the secret-bearing secp256k1 core
before the local execution signer or network identity consumers are built.

Scope: bounded milestone. Depends on v0.93.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Kani proofs for secp256k1 limb carries/borrows, wide multiplication,
  reduction, canonical field/scalar conversion, and serialization rejection;
- bounded scalar-multiplication equivalence against a small reviewed
  mathematical model;
- exceptional point-addition/doubling cases required by signing, recovery, and
  ECDH;
- explicit assumptions and unproved full-width domains paired with vector,
  property, differential, fuzz, side-channel, and audit evidence;
- reusable proof patterns fed into the broader `v0.291.0` crypto proof gate.

Verification:

- Pinned Kani report and independently reviewed mathematical model;
- seeded carry, reduction, exceptional-point, scalar-multiplication, and
  non-canonical serialization defects detected by the harnesses;
- exact first-party code paths from `v0.75.0..=v0.81.0` are proven rather
  than separate toy implementations.

Exit criteria:

- The first secret-bearing execution signer at `v0.195.0` and RLPx identity at
  `v0.257.0` do not rely only on vectors, fuzzing, and audit for core secp
  arithmetic invariants.
- `v0.94.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.95.0 - Signing And Transport Capability Separation

Status: planned; public crates.io checkpoint after cumulative review.

Goal: prevent secp256k1 execution signers, BLS12-381 consensus signers, and
secp256k1 transport key-agreement identities from being interchanged.

Scope: bounded milestone. Depends on v0.94.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Distinct sealed `ExecutionSigner` (secp256k1/ECDSA) and `ConsensusSigner`
  (BLS12-381) capabilities plus a non-signing `TransportIdentity`
  (secp256k1/ECDH where required) key-agreement capability;
- shared secp256k1 arithmetic remains an internal implementation detail; no
  public general-purpose signing supertrait or transport `sign` operation
  connects `TransportIdentity` to `ExecutionSigner`;
- scheme-tagged opaque key IDs, public keys, signatures, keystores, signing
  requests, roots/digests, and custody handles with no runtime algorithm strings;
- withdrawal credentials and withdrawal keys remain separate from validator
  signing keys and cannot implement `ConsensusSigner`;
- execution signing covers transactions, EIP-712, personal messages, and
  EIP-7702 authorizations; consensus signing covers only fork/domain-bound
  validator duties;
- provider, wallet, validator, hardware, remote, HSM/KMS, and threshold
  signer adapters must select one explicit signing scheme capability, while
  networking adapters consume only `TransportIdentity`.

Verification:

- Focused pre-implementation review of signer capability traits, key/custody
  ownership, scheme separation, and transport non-signing authority;
- Compile-fail tests proving secp signers cannot consume consensus duties and
  BLS signers cannot consume execution/message requests;
- compile-fail tests proving a transport identity/key cannot satisfy either
  signing capability or reach transaction/message signing APIs;
- compile-fail substitution tests across BLS/secp keys, signatures, IDs,
  keystores, roots, and requests;
- withdrawal-credential/key rejection tests for validator-signing slots;
- no string-selected algorithm dispatch or general-purpose signing capability
  in public signer or transport APIs.

Exit criteria:

- Algorithm and key-role confusion is structurally impossible before any
  signer implementation or validator duty is admitted.
- `v0.95.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.96.0 - Consensus Wire And Operational Limit Types

Status: planned; internal signed tag, publication at v0.100.0.

Goal: separate protocol validity from transport and local resource policy.

Scope: implementation pass. Depends on v0.95.0. The retained
workstream contract at v0.98.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement non-interchangeable limit domains and explicit retryable local-capacity outcomes.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Compile-fail domain substitutions and valid-object tests under deliberately small local capacity.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A local limit cannot become a consensus rejection rule.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.96.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.97.0 - Sealed Validation Context Issuance

Status: planned; internal signed tag, publication at v0.100.0.

Goal: bind validation to immutable parent and fork evidence.

Scope: implementation pass. Depends on v0.96.0. The retained
workstream contract at v0.98.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement trusted rules/context issuance, canonical digests, bounded parent leases and sealed child contexts.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Cross-fork/parent substitution, stale leases, concurrent branches and deserialized-context forgery tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Only the rules engine issues authority for an exact candidate and parent.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.97.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.98.0 - Contextual Protocol, Wire And Operational Limit Domains Completion

Status: planned; internal signed tag, publication at v0.100.0.

Goal: prevent global state, wire policy, or local resource capacity from
silently becoming a stricter Ethereum consensus rule.

Scope: completion and integration pass. Depends on v0.97.0.
The implementation passes v0.96.0 through v0.97.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
rules/limit/context integration with startup and local-capacity policy.

Deliverables:

- separate `ConsensusRules<Fork>`, `ContextLimits<Parent, Candidate>`,
  `WireLimits<Protocol, Version>`, `OperationalLimits`, and `WorkBudget`
  domains with no authority-conferring integer conversions among them;
- non-forgeable immutable per-operation `ValidationContext` with private fields
  and sealed rules-engine constructors accepting only trusted `ChainSpec`, a
  `VerifiedParent` or `GenesisContext`, the candidate header/envelope, and the
  validation implementation version;
- every context embeds parent identity/evidence plus a rules/limits digest used
  by caches and evidence; transaction, call, and precompile contexts derive as
  sealed children of the block context rather than caller-supplied structures;
- `VerifiedParent` and `GenesisContext` are bounded capabilities/handles that
  contain only required header, state, snapshot, and identity references; they
  never own a parent `ValidationContext` or recursive ancestry chain;
- block contexts retain bounded parent identity/evidence, not complete parent
  contexts; transaction/call/precompile children borrow or reference the
  minimum immutable rules/environment and never clone the block or parent
  evidence graph;
- context representation size is constant with chain height and call depth;
  dropping the final block/transaction scope deterministically releases its
  snapshot lease, arena reference, and other scoped capabilities;
- rules, limits, and context digests use canonical versioned encoding plus a
  domain-separated cryptographic hash; process-local `Hash`, pointer identity,
  randomized map state, and compiler-dependent struct layout are forbidden for
  persistent/cache/evidence identity;
- RPC input, peers, adapters, and stored records cannot deserialize, manually
  assemble, or substitute authoritative contexts; stored hints must be
  rederived and checked through the rules engine;
- historical blocks, side branches, out-of-order blocks, and concurrent fork
  contexts never consult one mutable global profile;
- static consensus capacities cover every explicit static maximum in the
  advertised supported network profile, including SSZ objects,
  consensus-embedded fixed SSZ branches, protocol-defined KZG proofs, fixed
  field/count dimensions, and request structures;
- Snap/MPT range proofs use negotiated `WireLimits` plus verification
  `WorkBudget`; JSON-RPC proofs use provider/RPC `OperationalLimits` plus
  verification work; neither category can create consensus invalidity solely
  from transport size or local policy;
- context-derived consensus work, including block-gas-limit changes, EVM
  memory, and ModExp/precompile work, is derived from the candidate's immutable
  parent/gas/fork context rather than startup constants;
- transactions and blocks constrained dynamically by gas receive no invented
  implementation-specific static byte maximum;
- protocol/version-negotiated wire limits classify message or connection
  violations through `PeerProtocolViolationEvidence`, never blockchain-object
  invalidity by themselves;
- `OperationalLimits` for local concurrency, caches, queues, peers, RPC,
  serving, scheduling, reservations, and additional evidence-collection
  cardinality without authority over validity;
- startup rejects static validating capacity below the advertised network
  profile or enters an explicit non-validating/light mode that cannot claim
  block validity;
- gas-derived execution/precompile bounds cannot be replaced by arbitrary
  local byte caps;
- objects violating consensus/context rules may produce
  `ProtocolInvalid { evidence: ObjectInvalidityEvidence }`; contextually valid
  objects that exceed available physical resources produce
  `LocalResourceExhausted` and withdraw validation readiness until capacity is
  restored;
- serving limits may be lower only as willingness-to-serve policy;
- production profiles publish a supported static and dynamic resource envelope;
- atomicity means one immutable validation context cannot mix fork rules or
  limits, not that the node has only one active ruleset.

Verification:

- Focused pre-implementation review of `ValidationContext`, bounded parent
  handles, child borrowing, digest encoding, and lease ownership/drop order;
- Static-maximum SSZ, consensus-proof, field/count, and request fixtures for
  every advertised network profile plus parent/candidate/gas-derived
  transaction and block work vectors;
- compile-fail direct-construction, deserialization, context mutation, fork/
  limit substitution, child-context escalation, and limit-domain conversion
  tests;
- genesis, unknown-parent, parent-evidence mismatch, corrupted stored-context,
  and rules/limits-digest substitution tests;
- million-parent synthetic-chain tests proving constant context size, child-
  context instrumentation proving no parent-evidence clone, and deep-call tests
  proving constant representation size;
- digest fixtures stable across restart, supported targets, map insertion
  order, and compiler settings; validation-version or canonical-encoding
  changes invalidate old hints safely;
- corrupted-handle, expired-snapshot-lease, deterministic-drop, and stored-
  context non-revival tests;
- concurrent canonical, side-branch, historical, out-of-order, and fork-digest
  validation using distinct immutable contexts;
- startup mode/configuration/resource-envelope matrices and compile-time type
  separation;
- fault tests proving within-protocol local exhaustion never produces
  invalidity and always withdraws/re-establishes readiness safely;
- wire-version violation, fork-context substitution, and
  serving-versus-validation independence tests;
- cross-domain proof tests showing RPC serving-policy rejection preserves
  cryptographic validity, oversized wire envelopes do not poison contained
  objects, local work exhaustion can later retry successfully, and consensus
  limits cannot be built from wire or operational limits;
- evidence-collection-limit substitution tests proving operational cardinality
  never enters consensus rules/context digests or changes validity.

Exit criteria:

- No implementation-specific cap classifies a contextually valid object as
  invalid; advertised static maxima are supported, dynamic work uses immutable
  rules-engine-issued candidate context, physical exhaustion removes readiness
  without manufacturing invalidity, and no caller can forge consensus
  authority by constructing a context or crossing limit domains; context size
  and identity remain bounded, non-recursive, canonical, and stable.
- `v0.98.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.99.0 - Evidence Reservation Lifecycle

Status: planned; internal signed tag, publication at v0.100.0.

Goal: implement conserved parent-child evidence ownership before worker and durable composition.

Scope: implementation pass. Depends on v0.98.0. The retained
workstream contract at v0.100.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement reserve/derive/fill/return/release with exact cardinality, cancellation and rollback semantics.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Bounded state-machine proofs, duplicate-return and abandoned-child fault tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No child can mint capacity or release another scope's filled evidence.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.99.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.100.0 - Hierarchical Evidence Capability Composition Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: make evidence capacity a conserved linear capability across standalone,
nested, concurrent, batch, and persistent validation workflows.

Scope: completion and integration pass. Depends on v0.99.0.
The implementation passes v0.99.0 through v0.99.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
worker transfer, batch and durable-record composition of the admitted lifecycle.

Deliverables:

- private, non-`Copy`, non-`Clone` `ReservedEvidenceSlot` and
  `FilledEvidenceSlot` typestates with exactly-once fill and one final
  return/commit/release ownership transition; optional attachments only borrow
  immutable evidence as refined by `v0.102.0`;
- RAII releases unused reservations during ordinary return, cancellation,
  unwinding, deferred processing, and local failure without permitting double
  release, use after return, or reuse of a filled slot;
- nested validators borrow a parent-authorized slot or consume an explicitly
  derived child reservation; they cannot mint capacity, reset accounting, or
  reserve an unrelated operation-wide slot;
- standalone transaction/proof/precompile/system-operation entry points reserve
  their own typed slot, while the same validators embedded in block validation
  require a parent-provided reservation;
- child invalidity evidence composes into parent evidence without changing the
  child's object identity, validation context, stage, or stable reason;
- producing both child-specific and parent-specific invalidity records requires
  an explicit atomic two-entry reservation before either authoritative check;
- batch verification reserves one slot for `BatchContainsInvalid`; bounded
  member isolation reserves a configured maximum of per-member child slots
  before those individual checks, independently of attacker-controlled batch
  size;
- member isolation stops with a local resource outcome when another child slot
  cannot be reserved, and only successfully filled member slots may reach
  negative caches, peer attribution, or sanctions;
- filled authoritative evidence is immutable: optional cache, persistence,
  attachment, diagnostic, or serialization failure cannot reclaim, mutate, or
  downgrade it;
- persistent stores commit only filled evidence records; abandoned reservations
  have no authoritative recovery representation and are discarded after a
  crash;
- one shared transition contract and adapters for transaction, block, proof,
  precompile, system-operation, batch, gossip, Engine, and import consumers;
- valid-object preparation is allocation-free and performs no diagnostic-only
  hashing/serialization, global mutex acquisition, globally contended atomic
  operation per child, parent/context clone, or optional sink invocation;
- "atomic reservation" means logical all-or-nothing ownership, not mandatory
  shared hardware atomics: standalone validation prefers a caller-owned fixed
  slot, block validation uses one parent-reserved bounded arena, and child slots
  are linear index handles with reservation amortized across nested validation;
- arena capacity derives from maximum simultaneous authorized validation work
  and the evidence mode, never attacker-controlled object or transaction count,
  and consumes `v0.72.0` memory/worker reservations before use;
- index reuse is protected by scoped borrowing or generation-tagged handles;
  cancellation/transfer cannot release a slot while a validator still borrows
  it, and exhaustion serializes, backpressures, or returns a retryable local
  outcome rather than object invalidity;
- large arenas are caller-supplied or externally allocated; stack placement is
  allowed only below a platform-audited ceiling;
- per-worker pools are optional and implemented only if `v0.104.0` benchmarks
  justify them; any admitted pool requires explicit ownership, reset,
  generation, and cross-worker transfer rules and cannot hide stale-slot reuse.

Verification:

- Compile-fail tests for copy, clone, direct construction, child capacity minting,
  fill after return, second fill, second release, and invalid typestate
  transitions;
- block to transaction to proof/precompile/system-operation nested reservation
  tests and standalone-versus-embedded validation/evidence equivalence tests;
- allocation counters, lock/atomic instrumentation, clone counters, and sink
  spies proving the valid path satisfies every hot-path prohibition;
- bounded `O(1)` reservation/release bookkeeping tests plus parent-arena/index-
  handle tests and, only when a pool is admitted, pool reset/generation tests;
- attacker-count-independent capacity, memory/worker capability exhaustion,
  stack-ceiling, stale-generation/ABA, live-borrow cancellation, and cross-
  worker transfer tests;
- child-to-parent evidence composition and explicit one-entry/two-entry
  reservation boundary tests preserving child object identity;
- multi-invalid batch isolation under capacity pressure, with a batch-size-
  independent maximum evidence count and no attribution for unfilled members;
- cancellation, panic/unwind, deferred, local-failure, valid, and ordinary-return
  tests proving every unused reservation is released exactly once;
- fault injection at every reserved, derived, filled, final-return/commit, and
  released transition plus every non-authoritative attachment, serialization,
  persistence, and cache side effect;
- deterministic recovery-model fixtures proving only committed filled records
  become authoritative evidence, with process-kill backend coverage assigned
  to `v0.225.0` and `v0.228.0`;
- bounded reference state-machine and deterministic concurrency tests plus
  stable harness interfaces consumed by the Kani proofs at `v0.289.0` and Loom
  exploration at `v0.293.0`.

Exit criteria:

- Evidence capacity cannot be created, duplicated, reset, leaked, or attributed
  without a filled slot; nested and batch validation preserve bounded cardinality
  and object identity, and recovery recognizes only committed filled evidence.
- `v0.100.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.101.0 - Evidence Collection Modes

Status: planned; internal signed tag, publication at v0.105.0.

Goal: separate consensus first-failure validation from bounded diagnosis and batch attribution.

Scope: implementation pass. Depends on v0.100.0. The retained
workstream contract at v0.102.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement FirstInvalid and validated bounded collection/isolation modes with one stable result contract.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Mode equivalence, first-record stability and zero/oversized configuration rejection.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Changing collection capacity cannot change object validity.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.101.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.102.0 - Evidence Collection Modes And Immutable Sink Access Completion

Status: planned; internal signed tag, publication at v0.105.0.

Goal: make evidence cardinality and optional sink behavior explicit without
allowing operational diagnostics to influence consensus validity or consume
linear slot authority.

Scope: completion and integration pass. Depends on v0.101.0.
The implementation passes v0.101.0 through v0.101.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
immutable sink views and stable public outcomes over the admitted modes.

Deliverables:

- sealed `EvidenceCollectionMode` domains for `FirstInvalid`,
  internal `CollectUpTo<N>` and internal `BatchIsolateUpTo<N>` implementations
  with validated nonzero cardinality plus one public non-generic bounded mode/
  configuration value; orchestration cannot instantiate arbitrary `N`;
- `FirstInvalid` is the default consensus-validation mode and reserves only
  the `v0.100.0` child and parent records required to reject the object;
- `CollectUpTo<N>` is an operational diagnostic mode: collection stops at `N`,
  exhaustion or sink failure is reported separately, and the first established
  validity result and authoritative evidence remain unchanged;
- `BatchIsolateUpTo<N>` starts only after `BatchContainsInvalid`, bounds
  member-specific attribution work independently of batch size, and leaves
  members beyond `N` explicitly unattributed;
- every collection cardinality is an `OperationalLimit` plus evidence-budget
  reservation, never a consensus rule, context/rules digest input, fork rule,
  object-invalid reason, or peer fault by itself;
- mode changes may affect only additional diagnostics, cache candidates, and
  peer attribution; they cannot turn invalid into valid, valid into invalid,
  or local failure into protocol invalidity;
- filling a `ReservedEvidenceSlot` remains exactly once and produces immutable
  evidence that cache, persistence, logging, tracing, and diagnostic sinks may
  borrow without taking or cloning slot authority;
- optional sink adapters accept immutable evidence views and return independent
  side-effect outcomes; they cannot fill, release, return, persist-authorize,
  or otherwise transition the slot;
- one final ownership transition returns/commits the filled authoritative
  record and releases reservation authority exactly once after all optional
  borrows end;
- a second authoritative object record always requires a distinct pre-reserved
  slot, even when its bytes, reason, parent, or diagnostic sinks overlap;
- public API documentation states which entry points use each mode and which
  outputs are authoritative, diagnostic-only, unattributed, or retryable;
- reservation trees, slot/index handles, collection implementations, sink
  types, and internal evidence lifetimes remain private; diagnostic modes keep
  the same public validation outcome shape as `FirstInvalid`, and internal
  dispatch maps validated runtime configuration onto a small reviewed set of
  admitted implementations;
- public requested limits of zero or above the documented maximum are rejected
  before validation; other limits map upward to the smallest admitted internal
  capacity class, but collection/isolation still stops at the exact requested
  limit and never silently rounds the operational policy upward;
- optional sinks stay outside the `no_std` kernel and cannot require heap
  allocation, `std`, async, or a runtime from consensus validation.

Verification:

- Compile-fail tests preventing optional sinks from owning, cloning, filling,
  releasing, returning, or retaining slot authority beyond the evidence borrow;
- API and code-size snapshots across every admitted cardinality and feature
  graph, including proof that const generics remain internal and arbitrary
  public instantiation is impossible;
- `no_std` default builds proving diagnostic sinks introduce no allocation,
  `std`, async, or runtime dependency;
- cross-mode property tests proving identical validity and identical first
  authoritative evidence for the same object/context regardless of operational
  collection limit;
- `FirstInvalid` child/parent reservation minima and no-extra-work tests;
- `CollectUpTo<N>` zero/one/maximum/beyond-limit tests proving deterministic
  stop behavior and unchanged validity under diagnostic allocation, logging,
  serialization, and sink failure;
- internal-class boundary tests for every public requested limit, including
  zero/above-maximum rejection, upward capacity-class dispatch, exact requested
  stop, and no rounding-down behavior;
- `BatchIsolateUpTo<N>` mixed-invalid and capacity-pressure tests proving
  members beyond `N` remain unattributed and cannot enter negative caches or
  peer sanctions;
- borrow-order permutations across cache, persistence, logging, tracing, and
  diagnostics followed by exactly one final ownership transition;
- property and compile-fail tests proving a second authoritative record cannot
  reuse the first record's slot or immutable evidence view;
- Kani-ready mode/slot reference model consumed by `v0.289.0`.

Exit criteria:

- Consensus validity is invariant across evidence-collection modes, evidence
  count is operationally bounded, optional sinks can observe but never consume
  authority, and each authoritative object record corresponds to one separately
  reserved and exactly-once-finalized slot.
- `v0.102.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.103.0 - Evidence API Containment

Status: planned; internal signed tag, publication at v0.105.0.

Goal: keep internal reservation and arena mechanics out of consumer APIs.

Scope: implementation pass. Depends on v0.102.0. The retained
workstream contract at v0.104.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement opaque outcome/evidence views, bounded private mode dispatch and a production-safe public feature surface.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- External compile fixtures, feature/API snapshots and monomorphization checks.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Consumers cannot disable authority or name internal arena/slot generics.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.103.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.104.0 - Evidence Hot-Path And API Containment Gate Completion

Status: planned; internal signed tag, publication at v0.105.0.

Goal: prove that evidence safety remains cheap for valid objects and does not
force internal reservation, cardinality, sink, or lifetime complexity into the
public SDK.

Scope: completion and integration pass. Depends on v0.103.0.
The implementation passes v0.103.0 through v0.103.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
valid-path performance baselines and SDK API containment acceptance.

Deliverables:

- production implementations of the `v0.100.0` caller-owned standalone slot,
  block-owned bounded arena, linear child index handle, and amortized nested
  reservation; per-worker pools are implemented only if measured contention/
  throughput evidence justifies their added lifecycle complexity;
- valid-object fast paths with zero heap allocation attributable to evidence
  machinery, zero diagnostic-only
  serialization/hashing, zero global mutex acquisition, zero globally
  contended atomic operation per nested validator, zero validation-context or
  parent-reservation clones, and zero optional sink calls; pre-existing
  consumer allocations are measured separately and are not mislabeled as
  evidence overhead;
- bounded `O(1)` slot reservation/release bookkeeping with operation counts
  independent of chain depth, call depth, and already-consumed arena entries;
- a private evidence service boundary: public validation APIs return one stable
  `ValidationOutcome<T, E>`-style shape and are not generic over slots,
  reservation trees, cardinality implementations, sinks, internal lifetimes,
  allocators, async runtimes, or worker pools;
- a small reviewed set of internal compile-time collection implementations plus
  one public non-generic validated bounded mode/configuration value;
  unrestricted public `CollectUpTo<N>` monomorphization is impossible;
- identical public return types and consensus behavior for `FirstInvalid`,
  diagnostic collection, default/minimal/all-feature graphs, and optional sink
  adapters;
- optional logging/cache/persistence/trace/diagnostic adapters remain outside
  the dependency-free `no_std` kernel and cannot require allocation, `std`,
  async, or a runtime from core validation;
- an internal evidence-disabled benchmark baseline compiled only by the
  benchmark harness, never as a production feature or validity path; this fully
  disabled baseline is valid-path only and replaces only evidence reserve/fill/
  finalize operations;
- invalid-path comparisons use both a stable semantic projection containing
  classification, reason, stage, object/context digests, and protocol-work
  counters and a minimal-evidence baseline that still constructs the fixed
  authoritative record but disables arena hierarchy and optional attachments;
- paired runs use identical inputs, contexts, scheduling, validator
  implementation, protocol-work accounting, and output consumption; valid
  outcomes and invalid semantic projections must match, protocol-work counters
  must remain equal, and evidence-operation counters are expected to differ;
- input construction, result/stable-digest hashing, benchmark-barrier setup, and
  equality verification occur outside the timed region;
- authoritative production thresholds use uninstrumented builds; allocation/
  lock/atomic/clone counters and sink spies run as separate conformance
  measurements using caller-owned/preallocated instrumentation that does not
  introduce the allocation or contention under test;
- all runs compute and consume result digests after the timed validator region
  through benchmark barriers so the optimizer cannot remove unrelated
  validation work;
- absolute production thresholds are authoritative; relative disabled-baseline
  deltas are diagnostic and cannot excuse an absolute regression;
- committed absolute and relative regression thresholds for slot operations,
  representative valid/nested validation, synthetic staged/batch contention,
  first-invalid, and diagnostic workloads on a reproducible hardware/toolchain
  profile, with real transaction/block/batch/gossip consumers required to adopt
  the baseline in their named later releases;
- committed stack size, validation-context size, reservation/arena size,
  generated code size, peak/retained memory, allocation count, lock/atomic
  operation count, and cycles/time measurements;
- threshold changes require benchmark evidence plus performance and security
  review rather than silent relaxation.

Verification:

- representative valid and nested validator benchmarks with normal evidence
  machinery and the internal evidence-disabled baseline, including allocation/
  lock/atomic/clone/sink conformance measurements and equality of outcomes/
  protocol-work counters;
- invalid-path semantic-projection and minimal-evidence-baseline benchmarks for
  first-invalid and diagnostic modes, with intentionally different evidence-
  operation counters;
- synthetic parent-arena benchmarks across child counts proving one amortized
  reservation and constant child bookkeeping;
- deterministic staged-worker benchmarks across worker counts and scheduling
  seeds, with no globally contended per-child operation;
- synthetic high-contention batch benchmarks for `BatchContainsInvalid` and
  bounded member isolation;
- `FirstInvalid`, every admitted compile-time collection cardinality, and
  minimum/maximum bounded runtime diagnostic-mode benchmarks;
- stack/context/slot/index/arena/code-size and peak/retained-memory reports for
  default, minimal, diagnostic, and all-feature graphs;
- API snapshots and compile-fail tests for leaked internal lifetimes, slots,
  arenas, sinks, worker pools, arbitrary `N`, and mode-dependent return types;
- benchmark-harness audits proving identical code paths around the replaced
  evidence operations, consumed output digests/barriers, and seeded optimizer-
  elision detection;
- timed-region audits and separate uninstrumented-performance/instrumented-
  conformance reports, including instrumentation self-tests proving counters
  and spies do not allocate or add global contention;
- `no_std` target builds with allocator/std/async/runtime dependency checks;
- seeded allocation, serialization, global-lock, contended-atomic, clone, sink-
  invocation, linear-scan reservation, and monomorphization regressions that
  breach the gate, plus stale-pool-reset regressions only when pools are
  admitted.

Exit criteria:

- Valid-object evidence overhead stays within committed release thresholds with
  no prohibited allocation, diagnostic work, contention, cloning, or sink
  invocation; public APIs expose stable validation outcomes rather than
  evidence machinery, and all feature/mode combinations preserve consensus
  behavior and the `no_std` kernel boundary.
- `v0.104.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.105.0 - Evidence Arena Ownership And Generations

Status: planned; public crates.io checkpoint after cumulative review.

Goal: make arena reuse safe under cancellation and worker transfer.

Scope: implementation pass. Depends on v0.104.0. The retained
workstream contract at v0.106.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement capability-sized arenas, scoped leases, generation-safe handles and fail-closed identity retirement.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Stale handles, partial reservations, concurrent transfer, reset and exhaustion state-machine tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Live evidence cannot alias reused arena storage.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.105.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.106.0 - Evidence Arena Capacity And Benchmark Integrity Completion

Status: planned; internal signed tag, publication at v0.110.0.

Goal: close arena sizing/reuse races and prove that evidence-overhead
comparisons measure only evidence machinery rather than optimizer or workload
differences.

Scope: completion and integration pass. Depends on v0.105.0.
The implementation passes v0.105.0 through v0.105.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
arena sizing and paired-run integrity acceptance using admitted handles.

Deliverables:

- an explicit `EvidenceArenaCapacity` derived from the maximum simultaneous
  validation work authorized by sealed worker/memory capabilities and the
  selected evidence mode, never raw attacker-controlled object, transaction,
  proof, cell, column, or message counts;
- arena construction consumes `v0.72.0` memory and worker reservations before
  exposing handles; partial reservation rolls back atomically and cannot begin
  authoritative validation;
- exhaustion has only operational outcomes: serialize work, apply bounded
  backpressure, reduce optional diagnostic/isolation work, or return
  `LocalResourceExhausted`; it cannot produce object invalidity, peer fault, or
  a consensus-rule change;
- scoped child borrows where possible and generation-tagged indexes where
  reuse/transfer requires handles, preventing stale-handle and ABA aliasing;
- generation exhaustion/wrap retires the affected slot or arena and returns a
  retryable local outcome; it never wraps into a previously valid handle
  identity or resets generation state in place;
- cancellation, unwind, timeout, and cross-worker transfer protocols retain a
  live-borrow/lease count so no slot can be released, reset, reused, or returned
  while a validator still references it;
- caller-supplied or externally allocated storage for arenas above a committed
  platform stack ceiling; stack placement is admitted only by audited size and
  target-specific stack evidence;
- per-worker pools remain absent by default and are implemented only after
  `v0.104.0` measurements show a material threshold benefit that outweighs
  synchronization, generation, reset, memory-retention, and transfer costs;
- any admitted pool has bounded retained memory, explicit owner/worker identity,
  generation/reset on reuse, deterministic drain/drop, and no implicit cross-
  worker migration;
- one public non-generic evidence mode/configuration value with validation and
  bounded runtime cardinality; internal dispatch alone selects from the small
  reviewed const-generic implementations;
- an optimizer-resistant paired benchmark contract: identical input, context,
  validator, schedule, work accounting, and output consumption, replacing only
  evidence reservation/fill/finalization operations on valid paths;
- invalid paths compare the `v0.109.0` semantic projection and minimal-evidence
  baseline rather than requiring a fully disabled run to synthesize an
  impossible byte-identical `ObjectInvalidityEvidence`;
- paired runs assert equal valid outcomes or invalid semantic projections and
  equal protocol-work counters, treat evidence-operation counters as expected
  differences, consume untimed stable result digests through benchmark
  barriers, and report evidence-attributable allocation separately from total
  consumer allocation;
- absolute production thresholds remain release authority; relative baseline
  deltas are supporting diagnostics only.

Verification:

- capacity property tests across evidence modes, worker counts, memory permits,
  and adversarially large object-count fields proving attacker counts do not
  size arenas;
- partial memory/worker reservation rollback, serialization/backpressure,
  reduced-diagnostic, and retryable-local-exhaustion tests with no invalidity or
  peer attribution;
- stale index, generation retirement/wrap, ABA, double reuse, use-after-release,
  live-borrow cancellation, timeout/unwind, and cross-worker transfer races;
- Loom models for allocate/borrow/fill/cancel/transfer/release/reset and Kani-
  ready capacity/conservation state models consumed at `v0.289.0` and
  `v0.293.0`;
- platform stack-ceiling checks, large-arena external-storage tests, retained-
  memory bounds, and deterministic pool drain/drop tests when pools are admitted;
- A/B harness source/IR or equivalent structural checks proving only evidence
  operations differ, with identical scheduling and validator dispatch;
- equal-outcome/equal-work-counter assertions, consumed result digests,
  benchmark barriers, invalid semantic projections/minimal-evidence baselines,
  protocol/evidence counter separation, and seeded dead-code-elimination
  regressions;
- allocation attribution tests distinguishing evidence allocations from
  unrelated consumer allocations for transaction, block, gossip, import, and
  batch harnesses;
- public API snapshots proving non-generic mode configuration and identical
  `ValidationOutcome<T, E>` across all admitted modes.

Exit criteria:

- Arena capacity is capability-backed and attacker-count-independent, stale or
  live handles cannot observe reused storage, exhaustion stays local, optional
  pools exist only with evidence, and benchmark deltas isolate evidence work
  without weakening authoritative absolute production thresholds.
- `v0.106.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.107.0 - Evidence Benchmark Semantic Oracles

Status: planned; internal signed tag, publication at v0.110.0.

Goal: prove benchmark variants perform the same protocol work.

Scope: implementation pass. Depends on v0.106.0. The retained
workstream contract at v0.109.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement valid/invalid semantic projections, deterministic paired inputs and separate protocol/evidence operation counts.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Seeded skipped validation, altered outcomes and hidden-allocation mutations must fail the benchmark oracle.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A faster run is not accepted when it performed less protocol work.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.107.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.108.0 - Evidence Measurement And Dispatch Harness

Status: planned; internal signed tag, publication at v0.110.0.

Goal: measure production overhead without instrumentation or dispatch distortion.

Scope: implementation pass. Depends on v0.107.0. The retained
workstream contract at v0.109.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement untimed preparation, uninstrumented timing, separate instrumented conformance and exact requested-capacity dispatch.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Timing-region contamination, optimizer-elision, upward-class mapping and generation-exhaustion regressions.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Absolute thresholds and exact requested bounds remain authoritative.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.108.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.109.0 - Evidence Benchmark Measurement And Dispatch Closure Completion

Status: planned; internal signed tag, publication at v0.110.0.

Goal: make valid/invalid benchmark comparisons, timed regions,
instrumentation, runtime mode dispatch, and generation exhaustion fully
deterministic and fail closed.

Scope: completion and integration pass. Depends on v0.108.0.
The implementation passes v0.107.0 through v0.108.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
measurement/dispatch composition and final integrity thresholds.

Deliverables:

- the fully evidence-disabled baseline is restricted to valid paths, where both
  production and baseline return the same valid outcome without requiring an
  impossible evidence-bearing invalid result;
- a canonical benchmark-only invalid semantic projection containing outcome
  classification, stable reason code, validation stage, object digest, context/
  rules digest, validation version, bounded location, and protocol-work
  counters, excluding optional attachments and slot/arena identity;
- a minimal-evidence invalid baseline that constructs the same fixed
  allocation-free `ObjectInvalidityEvidence` but disables hierarchical arena
  operations and every optional sink/attachment, permitting separate
  measurement of hierarchy and attachment overhead;
- protocol-work counters cover consensus/protocol decode, hash, signature,
  proof, state, and deterministic validation work and must match paired runs;
  evidence-operation counters cover reserve, derive, fill, borrow, finalize,
  attach, and sink work and are intentionally compared rather than required to
  match;
- timed regions contain the complete validator invocation and the evidence
  operations belonging to the selected production/baseline path, including
  polling async validators to completion and finalizing or releasing operation-
  scoped reservations, child handles, leases, cancellation guards, temporary
  arenas, RAII destructors, deferred cleanup, and resource counters; input/
  object/context construction, stable-digest/result hashing, output equality
  checks, report formatting, and benchmark-barrier preparation occur outside
  timing, and only the compact returned outcome may remain alive afterward;
- authoritative production latency/throughput thresholds run uninstrumented;
  allocation, lock, atomic, clone, and sink-spy checks run in separate
  instrumented conformance binaries/configurations;
- instrumentation is caller-owned or preallocated, avoids global locks and
  heap allocation in the measured path, and has self-tests demonstrating that
  it does not create the behavior it detects;
- every result is consumed after timing through a stable digest or equivalent
  benchmark barrier, with seeded optimizer-elision checks;
- the public non-generic mode/configuration rejects zero and values above the
  documented maximum before validation; admitted values dispatch upward to the
  smallest internal capacity class that can hold them while enforcing the
  exact requested collection/isolation stop;
- memory permits, worker accounting, and admission control charge and reserve
  the full selected physical capacity class before validation begins, never
  merely the caller's smaller logical limit; failure to reserve that class is
  a retryable local outcome and no validator work starts;
- internal capacity-class rounding never becomes visible as extra diagnostics,
  cache entries, peer attribution, evidence records, or work beyond the public
  operational limit, and no value is rounded down;
- generation increment exhaustion permanently retires the slot/arena identity
  or returns a retryable local outcome; no wrap, in-place reset, or identity
  resurrection is permitted.

Verification:

- valid-path full-disable comparisons with byte/semantic outcome and protocol-
  work equality;
- invalid-path production/minimal-evidence comparisons using the canonical
  semantic projection, equal protocol-work counters, and expected evidence-
  operation deltas;
- first-invalid, diagnostic, batch-isolation, cancellation, local-exhaustion,
  and optional-sink-failure benchmark cases;
- timed-region boundary tests proving setup, digest/result hashing, equality
  checks, barriers, and report work are excluded while async polling, resource
  release/finalization, RAII destruction, deferred cleanup, and restoration of
  expected post-operation counters are included;
- uninstrumented threshold reports paired with separate instrumented allocation/
  lock/atomic/clone/sink conformance reports;
- instrumentation self-tests and seeded perturbation cases proving counters or
  spies cannot add measured allocation, global lock acquisition, or contention;
- optimizer-elision fixtures caught by consumed result digests/barriers;
- exhaustive public limit tests over zero, one, every internal class boundary,
  between-class values, maximum, and above-maximum values, proving upward class
  mapping with exact requested stop and pre-validation rejection where required;
- constrained-permit boundary tests proving, for example, that logical limit 17
  selects and charges capacity 32, stops collection at 17, and cannot begin
  validation when the capacity-32 memory/worker reservation is unavailable;
- generation-at-maximum, retirement, stale-handle, ABA, concurrent borrow,
  cancellation, transfer, persistence/restart, and retryable-local-outcome tests;
- Kani-ready dispatch and generation state models consumed at `v0.289.0`.

Exit criteria:

- Benchmarks compare semantically equivalent work without timing setup or
  instrumentation artifacts, but include complete async and resource-cleanup
  lifecycle cost; production thresholds remain uninstrumented and authoritative,
  runtime mode dispatch charges its physical class without exceeding the exact
  requested operational limit, and generation identity can never wrap or
  resurrect.
- `v0.109.0 implementation stop reached. Run pentest for this exact
  commit.`

## Roadmap Expansion From The 2026 Gap Analysis

The releases below replace the earlier narrow integration roadmap. They assign
every gap identified by the July 2026 completeness reviews to a version instead
of leaving work as an unversioned deferral. The roadmap may extend beyond
`v0.449.0` when new official Ethereum work or a newly discovered completeness
gap requires another small pass. Reaching a high `0.x` version is preferable
to compressing security-sensitive work into oversized releases.

Every release below inherits these gates:

- check current official Ethereum specifications, EIPs, test fixtures, and
  client behavior before implementation, then pin relevant revisions in
  `spec-lock.toml`;
- preserve the `no_std` core, explicit optional integrations, forbidden
  first-party unsafe code, and the 500-line Rust source limit;
- add unit, integration, negative, property, fuzz, fixture, and differential
  coverage in proportion to the release risk;
- update `SPEC_MATRIX.md`, `current-status.md`, public API documentation,
  examples, migration notes, and release notes;
- run the full release gate, pentest the exact release candidate, remediate all
  findings, and obtain a clean retest before tagging.

Real public API or behavior changes use a new `0.x.0` release. A `0.x.y`
release is reserved for narrow remediation, forced dependency propagation, or
release-process maintenance that does not hide a breaking public change.
Every planned `0.x.y` milestone must include an explicit `Patch rationale:`
before its goal. The release-plan checker rejects missing rationales and
non-increasing `0.x` milestones. If implementation or pentest review discovers
that a planned patch changes public compatibility or Ethereum behavior, that
milestone must be promoted to the next unused `0.x.0` release and every later
unpublished milestone renumbered before release.

The post-`v0.52.7` roadmap was renumbered before implementation:

- former `v0.52.8..=v0.52.37` became `v0.53.0..=v0.82.0`;
- former `v0.53.0..=v0.310.0` became `v0.83.0..=v0.340.0`, preserving
  deliberately planned patch suffixes;
- published `v0.52.0..=v0.52.7`, their reports, release notes, and tags remain
  unchanged.

Roadmap source review date: 2026-07-17. Active fork names and requirements must
come from pinned official sources, not memory:

- <https://ethereum.org/roadmap/>
- <https://ethereum.org/roadmap/statelessness/>
- <https://github.com/ethereum/execution-specs>
- <https://github.com/ethereum/execution-spec-tests>
- <https://github.com/ethereum/consensus-specs>
- <https://github.com/ethereum/execution-apis>
- <https://github.com/ethereum/devp2p>
- <https://github.com/ethereum/portal-network-specs>
- <https://ethereum.github.io/beacon-APIs/>
- <https://ethereum.github.io/keymanager-APIs/>
- <https://ethereum.github.io/builder-specs/>
- <https://github.com/ethereum/hive>
- <https://ethereum.github.io/yellowpaper/paper.pdf>
- <https://eips.ethereum.org/EIPS/eip-3076>
- <https://eips.ethereum.org/EIPS/eip-3540>

## Phase 9: Owned SDK And Shared Domain Foundation

### v0.110.0 - General Integer Primitives

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Ethereum-sized integer work no longer depends on transaction-specific `Wei` helpers or external core types.

Scope: bounded milestone. Depends on v0.109.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- First-party `U256` and `I256`, checked arithmetic, endian conversion, parsing, formatting, and explicit overflow policy.
- Deliberate minor-release removal of the `0.52.x` sanitization compatibility
  names `sanitize_bytes`, `best_effort::sanitize_bytes_best_effort`, and
  `HARDENED_MODE`; retain `wipe` and `HARDENING_FEATURES_ENABLED` as the
  canonical surface, bump the affected support crate for the breaking API
  change, and publish migration guidance.

Verification:

- Arithmetic KATs, boundary/property tests, differential checks, fuzzing.
- Compile fixtures prove the legacy sanitization names are unavailable while
  canonical wipe and runtime-protection-report workflows remain usable through
  both `eth-valkyoth-sanitization` and the `eth` facade.

Exit criteria:

- Ethereum-sized integer work no longer depends on transaction-specific `Wei` helpers or external core types.
- The temporary `0.52.x` sanitization compatibility surface is removed only at
  this deliberate facade minor boundary, with correct support-crate versioning
  and documented replacements.
- `v0.110.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.111.0 - Bytes And Hash Domains

Status: planned; internal signed tag, publication at v0.115.0.

Goal: Raw byte arrays and generic `B256` are not the only public representation for semantically distinct domains.

Scope: bounded milestone. Depends on v0.110.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Owned and borrowed `Bytes`, fixed-byte families, and distinct transaction, block, receipt, state, storage, code, and signing hash newtypes.

Verification:

- Conversion, domain-mismatch compile tests, allocation-limit tests, fuzzing.

Exit criteria:

- Raw byte arrays and generic `B256` are not the only public representation for semantically distinct domains.
- `v0.111.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.112.0 - Session-Bound Traversal API Hardening

Status: planned; internal signed tag, publication at v0.115.0.



Goal: make it difficult to process untrusted borrowed decode models through an
unaccounted compatibility iterator by mistake.

Scope: bounded milestone. Depends on v0.111.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Audit every first-party transaction, proof, RPC, networking, fixture, and
  execution consumer of borrowed RLP-backed models and migrate all untrusted
  paths to the shared `DecodeSession` traversal APIs;
- introduce a compile-time distinction, session-bound wrapper, or equally
  strong API boundary so models admitted through session-aware decoding do not
  silently expose unaccounted traversal as the natural next operation;
- rename or deprecate compatibility traversal with explicit trusted or
  `without_session` terminology for access-list entries, storage keys, blob
  hashes, EIP-7702 authorizations, and inline MPT nodes;
- preserve an intentional compatibility path for already trusted or
  independently bounded data without presenting it as safe for untrusted
  composite work;
- publish migration guidance mapping every legacy iterator to its charged
  session-aware replacement.

Verification:

- Repository-wide static/API guards prove authoritative first-party untrusted
  paths do not call compatibility traversal;
- compile-fail tests prove session-bound models cannot select an unaccounted
  traversal without an explicit trust-boundary conversion;
- exact counter-oracle tests cover repeated and nested access-list, storage-key,
  blob-hash, authorization, and inline-MPT traversal;
- fuzzing repeatedly traverses borrowed models and proves all admitted work is
  conserved by one non-resetting session;
- documentation, examples, and downstream migration fixtures use the charged
  APIs for untrusted input.

Exit criteria:

- No first-party untrusted-data path can bypass the shared decode ledger through
  a legacy iterator.
- Public compatibility traversal is explicitly named and documented as trusted
  or independently bounded, while session-admitted models require an explicit
  charged path or trust-boundary conversion.
- `v0.112.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.113.0 - Ethereum Hex And Address Text

Status: planned; internal signed tag, publication at v0.115.0.

Goal: stabilize canonical Ethereum text independently of serde.

Scope: implementation pass. Depends on v0.112.0. The retained
workstream contract at v0.114.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement quantity/data separation, checked hex and EIP-55/EIP-1191 checksum rules with bounded errors.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official checksum cases, odd lengths, leading zeroes, mixed case and integer overflow.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Canonical text round-trips before optional JSON integration.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.113.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.114.0 - Ethereum Text And Serde Interoperability Completion

Status: planned; internal signed tag, publication at v0.115.0.

Goal: Common Ethereum wire and display forms round-trip canonically without weakening the default graph.

Scope: completion and integration pass. Depends on v0.113.0.
The implementation passes v0.113.0 through v0.113.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
bounded optional serde adapters using the admitted text codecs.

Deliverables:

- Quantity/data hex codecs, EIP-55 and EIP-1191 address checksums, bounded optional serde, JSON-RPC quantity rules, and stable text errors.

Verification:

- Official checksum vectors, serde snapshots, malformed/oversized corpus, differential checks.

Exit criteria:

- Common Ethereum wire and display forms round-trip canonically without weakening the default graph.
- `v0.114.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.115.0 - Owned Transaction Models

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Applications can retain and mutate complete transactions without keeping input buffers alive.

Scope: bounded milestone. Depends on v0.114.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Owned legacy, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 transactions, requests, signatures, access lists, authorization lists, and blob sidecars.

Verification:

- Round trips against borrowed forms and official transaction fixtures.

Exit criteria:

- Applications can retain and mutate complete transactions without keeping input buffers alive.
- `v0.115.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.116.0 - Owned Block And Receipt Models

Status: planned; internal signed tag, publication at v0.120.0.

Goal: Full execution payload data has stable owned SDK models.

Scope: bounded milestone. Depends on v0.115.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Owned headers, blocks, bodies, receipts, logs, withdrawals, ommers, execution requests, and fork-specific optional fields.

Verification:

- Cross-fork fixture round trips, root-input serialization tests, serde snapshots.

Exit criteria:

- Full execution payload data has stable owned SDK models.
- `v0.116.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.117.0 - Owned State And Execution Models

Status: planned; internal signed tag, publication at v0.120.0.

Goal: State and execution APIs no longer require disconnected adapter-only models.

Scope: bounded milestone. Depends on v0.116.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Owned accounts, code, storage slots, state diffs, execution environments, results, logs, refunds, access summaries, and witness references.

Verification:

- State conversion/property tests, deterministic serialization, allocation-limit tests.

Exit criteria:

- State and execution APIs no longer require disconnected adapter-only models.
- `v0.117.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.118.0 - Lossless Model Conversion Matrix

Status: planned; internal signed tag, publication at v0.120.0.

Goal: Every supported representation change is explicit, testable, and documented as lossless or intentionally lossy.

Scope: bounded milestone. Depends on v0.117.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Checked conversions among borrowed, owned, canonical, validated, RPC, signer, and execution representations with preserved evidence.

Verification:

- Conversion matrix tests, lossy-conversion rejection, compile-fail typestate tests.

Exit criteria:

- Every supported representation change is explicit, testable, and documented as lossless or intentionally lossy.
- `v0.118.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.119.0 - Bounded Allocation Convenience

Status: planned; internal signed tag, publication at v0.120.0.

Goal: Ergonomic allocation support does not weaken bounded resource or atomic-output guarantees.

Scope: bounded milestone. Depends on v0.118.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Caller-owned transactional output buffers, scratch arenas, reusable workspaces, bounded collections, and all-or-nothing writer APIs.

Verification:

- OOM/limit simulation, output-unchanged tests, reuse tests, Miri candidates.

Exit criteria:

- Ergonomic allocation support does not weaken bounded resource or atomic-output guarantees.
- `v0.119.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.120.0 - Decode Policies And Error Context

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Integrators can select reviewed policies and diagnose failures without parsing strings.

Scope: bounded milestone. Depends on v0.119.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Named deployment policy builders plus structured, bounded field/index/
  offset/source error context without secret leakage or diagnostic-output
  amplification;
- error paths and excerpts consume the `v0.93.0` evidence/diagnostic budget
  when promoted into persistent or public evidence.

Verification:

- Error snapshot tests, redaction and maximum-path/output tests, nested
  malformed fixtures, compatibility checks.

Exit criteria:

- Integrators can select reviewed policies and diagnose failures without parsing strings.
- `v0.120.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.121.0 - Payload-Bound Typestates

Status: planned; internal signed tag, publication at v0.125.0.

Goal: Validation state cannot become detached from the exact payload it proves.

Scope: bounded milestone. Depends on v0.120.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Transaction/block payloads travel with canonicality, fork, signature, proof,
  execution evidence, and the non-forgeable rules-engine context capability
  from `v0.98.0`; constructors remain proof-gated;
- no typestate accepts a caller-built or deserialized authoritative
  `ValidationContext`.

Verification:

- Compile-fail transition/context-construction tests, evidence preservation
  tests, forged-state and context-substitution rejection.

Exit criteria:

- Validation state cannot become detached from the exact payload it proves.
- `v0.121.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.122.0 - Chain Specification And Fork Rules 2.0

Status: planned; internal signed tag, publication at v0.125.0.

Goal: Consensus behavior never depends on enum ordinal ordering or a hardcoded mainnet chronology.

Scope: bounded milestone. Depends on v0.121.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Separate fork identity, activation schedule, rule capabilities, parameters,
  system hooks, and complete historical/custom-chain configuration;
- validate configuration into a non-forgeable trusted `ChainSpec` capability;
- the sealed rules engine is the only constructor of immutable per-object
  `ValidationContext` values required by `v0.98.0`, accepting only trusted
  chain specification, `VerifiedParent` or `GenesisContext`, candidate
  header/envelope, and implementation version;
- issue and verify parent identity/evidence plus the rules/limits digest, and
  derive sealed transaction/call/precompile child contexts without mutating
  one node-global active profile;
- parent capabilities are bounded non-recursive handles and all chain/rules/
  limits/context digests use canonical versioned encoding plus domain-separated
  cryptographic hashing.

Verification:

- Historical mainnet vectors, custom-chain schedules, monotonicity/property
  tests, concurrent pre/post-fork side-branch validation, historical and
  out-of-order validation, genesis/unknown-parent cases, and direct-context/
  fork/limit/parent/digest substitution tests;
- million-parent constant-size handle tests and digest fixtures stable across
  restart and every supported target.

Exit criteria:

- Consensus behavior never depends on enum ordinal ordering or a hardcoded mainnet chronology.
- `v0.122.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.123.0 - Shared Protocol And Execution Domains

Status: planned; internal signed tag, publication at v0.125.0.

Goal: Equivalent protocol and EVM concepts no longer drift behind parallel types.

Scope: bounded milestone. Depends on v0.122.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- One address, word, gas, account, state, log, access, execution-status, and error vocabulary shared by protocol and native EVM crates.

Verification:

- API conversion audit, compile checks, no-copy path tests, semver baseline.

Exit criteria:

- Equivalent protocol and EVM concepts no longer drift behind parallel types.
- `v0.123.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.124.0 - Native EVM Core Integration

Status: planned; internal signed tag, publication at v0.125.0.

Goal: The optional execution facade is a real first-party path, not a disconnected descriptor layer.

Scope: bounded milestone. Depends on v0.123.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- `eth-valkyoth-evm` consumes the first-party core and shared domains; one validated owned transaction executes through the public boundary.

Verification:

- End-to-end fixture, fail-closed unsupported paths, reference differential test.

Exit criteria:

- The optional execution facade is a real first-party path, not a disconnected descriptor layer.
- `v0.124.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.125.0 - External no_std And SDK Consumer Pilot

Status: planned; public crates.io checkpoint after cumulative review.

Goal: prove the facade is usable outside this workspace.

Scope: implementation pass. Depends on v0.124.0. The retained
workstream contract at v0.126.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Create clean consumer fixtures for no_std decoding/proofs and an alloc-based transaction workflow using only public APIs and packaged crates.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Compile with MSRV/stable, no path patches/private modules, and deliberate feature omission.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A downstream application can replace buffers and providers without workspace internals.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.125.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.126.0 - Facade Prelude And Feature Truth Completion

Status: planned; internal signed tag, publication at v0.130.0.

Goal: Public discovery is simple and no feature name implies functionality it does not provide.

Scope: completion and integration pass. Depends on v0.125.0.
The implementation passes v0.125.0 through v0.125.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
facade prelude, truthful feature tables and downstream discovery ergonomics.

Deliverables:

- Curated prelude, task-oriented modules, truthful feature names, generated feature/dependency tables, and default-graph assertions.

Verification:

- Feature powerset sampling, README snippet generation check, docs tests.

Exit criteria:

- Public discovery is simple and no feature name implies functionality it does not provide.
- `v0.126.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.127.0 - Ecosystem Conversion Adapters

Status: planned; internal signed tag, publication at v0.130.0.

Goal: Interoperability is available without making third-party core models authoritative.

Scope: bounded milestone. Depends on v0.126.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Optional reviewed conversions for Alloy, Reth, and other admitted ecosystem types behind compatibility features.

Verification:

- Version matrix, conversion fixtures, default-graph exclusion checks.

Exit criteria:

- Interoperability is available without making third-party core models authoritative.
- `v0.127.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.128.0 - Owned SDK Hardening

Status: planned; internal signed tag, publication at v0.130.0.

Goal: The owned SDK foundation is stable enough for execution, providers, wallets, and storage to build upon.

Scope: bounded milestone. Depends on v0.127.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Fuzz all owned parsers/conversions, lock serde and display snapshots, establish semver baselines, and audit allocation behavior.

Verification:

- Full SDK fuzz suite, cargo-semver-checks baseline, docs/package checks, pentest.

Exit criteria:

- The owned SDK foundation is stable enough for execution, providers, wallets, and storage to build upon.
- `v0.128.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 10: Complete First-Party Execution

### v0.129.0 - Official State-Test Admission

Status: planned; internal signed tag, publication at v0.130.0.

Goal: Native execution claims are backed by official state-test evidence.

Scope: bounded milestone. Depends on v0.128.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Harness official execution state tests by fork with explicit supported and skipped scopes.

Verification:

- Pinned fixtures, unsupported-reason report, cross-client differential samples.

Exit criteria:

- Native execution claims are backed by official state-test evidence.
- `v0.129.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.130.0 - Native EVM Audit Hardening

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Deeper state transition work rests on an independently reviewed engine.

Scope: bounded milestone. Depends on v0.129.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Broaden bytecode, stack, memory, gas, journal, call-frame, and precompile fuzzing; close audit findings.

Verification:

- Fuzz corpus, load/DoS tests, stack report, clean pentest/retest.

Exit criteria:

- Deeper state transition work rests on an independently reviewed engine.
- `v0.130.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.131.0 - Genesis And Chain Configuration

Status: planned; internal signed tag, publication at v0.135.0.

Goal: A chain can be initialized from explicit configuration without external core logic.

Scope: bounded milestone. Depends on v0.130.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Parse and validate genesis/config data, allocs, fork schedules, terminal conditions, and initial state/header roots.

Verification:

- Mainnet/testnet/custom genesis fixtures and negative cases.

Exit criteria:

- A chain can be initialized from explicit configuration without external core logic.
- `v0.131.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.132.0 - Transaction Validity Rule Families

Status: planned; internal signed tag, publication at v0.135.0.

Goal: implement state-independent and account-dependent validity as separate checked stages.

Scope: implementation pass. Depends on v0.131.0. The retained
workstream contract at v0.133.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement fork-bound intrinsic/envelope checks then nonce/balance/fee validation with exact-input evidence; leave block promotion to integration.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Per-transaction-family positive/negative vectors, boundary gas and state-context substitution.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Every supported transaction family has explicit semantic rejection evidence.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.132.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.133.0 - Semantic Transaction Validity Completion

Status: planned; internal signed tag, publication at v0.135.0.

Goal: Decoded transactions can be proven consensus-valid for a stated chain context.

Scope: completion and integration pass. Depends on v0.132.0.
The implementation passes v0.132.0 through v0.132.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
integration of existing signature/authorization/blob evidence with semantic transaction stages.

Deliverables:

- Complete intrinsic gas, nonce, balance, fee, chain, authorization, blob,
  initcode, sender, and fork checks for every transaction type;
- consume only a sealed transaction child context derived from the containing
  block context; RPC or caller-supplied fork/limit structures have no
  consensus authority;
- reserve a `v0.93.0` minimal `EvidenceSlot` before authoritative semantic
  validation; reservation failure returns local exhaustion before checks begin
  and every invalid path fills the slot allocation-free;
- follow `v0.100.0`: standalone transaction validation reserves its own slot,
  while block-embedded validation accepts only a block-derived child
  reservation and produces equivalent transaction evidence without resetting
  the parent budget;
- use `v0.102.0` `FirstInvalid` for authoritative consensus entry points;
  diagnostic collection may gather more bounded evidence but cannot change the
  transaction result or first authoritative record.

Verification:

- Official transaction tests, cross-type property tests, client differential
  checks, compile-fail caller-context substitution tests, and evidence-slot
  reservation/fill/release fault tests for every invalid reason;
- standalone-versus-embedded equivalence and parent-budget conservation tests;
- cross-mode validity/first-evidence equivalence and diagnostic-sink failure
  tests;
- valid-transaction benchmarks with and without the internal `v0.104.0`
  evidence baseline across transaction types and evidence modes, enforcing no
  allocation, diagnostic hashing/serialization, contention, clone, or sink
  invocation on the valid `FirstInvalid` path.

Exit criteria:

- Decoded transactions can be proven consensus-valid for a stated chain context.
- `v0.133.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.134.0 - Header And Block Validity

Status: planned; internal signed tag, publication at v0.135.0.

Goal: Headers and block envelopes can be validated against parent and chain state.

Scope: bounded milestone. Depends on v0.133.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Parent linkage, gas/base fee, difficulty/TTD, timestamps, ommers,
  withdrawals, blob gas, requests, roots, and fork-field validation;
- consume one immutable `v0.98.0` `ValidationContext` per candidate, so
  canonical, side-branch, historical, and out-of-order blocks can use distinct
  fork rules concurrently;
- reject contexts not issued by the sealed rules engine or whose parent
  evidence, candidate identity, validation version, or rules/limits digest no
  longer matches;
- reserve a `v0.93.0` minimal `EvidenceSlot` before authoritative header/block
  checks, so every invalid path has infallible allocation-free evidence;
- reserve the complete `v0.100.0` parent/child evidence cardinality required by
  the selected block-validation mode before nested authoritative checks; child
  validators consume derived reservations, and simultaneous transaction- and
  block-specific evidence requires an explicit two-entry reservation;
- authoritative import uses `v0.102.0` `FirstInvalid`; explicitly requested
  operational diagnostics may use bounded `CollectUpTo<N>` without changing
  block validity, first evidence, Engine outcome, or bad-block authority;
- derive `v0.106.0` arena capacity from simultaneous authorized block/
  transaction work and evidence mode rather than block transaction count;
  arena exhaustion serializes/backpressures or returns a retryable local
  outcome;
- every failure is classified through `v0.93.0`; only complete
  `ObjectInvalidityEvidence` can permanently reject the block.

Verification:

- Blockchain/header fixtures across all claimed forks plus concurrent
  pre/post-fork, unknown-parent, stale/corrupted-context, and context-
  substitution cases;
- pre-validation slot exhaustion and post-invalid diagnostic/cache/persistence
  failure tests preserving the immediate invalid result;
- block to transaction to proof/precompile/system-operation nesting, child
  identity composition, two-entry boundary, and exactly-once release tests;
- collection-mode and optional-sink fault matrices with invariant block result
  and first authoritative evidence;
- valid block-envelope benchmarks across transaction counts and worker
  configurations, proving one amortized arena reservation, constant child
  bookkeeping, and the `v0.104.0` overhead thresholds;
- attacker-count-independent arena sizing, capability exhaustion, stale-handle,
  cancellation/transfer, and local-backpressure tests.

Exit criteria:

- Headers and block envelopes can be validated against parent and chain state.
- `v0.134.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.135.0 - Ordered Transaction State Journal

Status: planned; public crates.io checkpoint after cumulative review.

Goal: compute transaction state changes with correct nested rollback.

Scope: implementation pass. Depends on v0.134.0. The retained
workstream contract at v0.137.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement ordered block transaction execution, journal leases and commit/revert, consuming existing execution/evidence capabilities.
- Integrate nested precompile admission and terminal outcomes, including
  v0.58.0 G1ADD: protocol input/out-of-gas errors fail CALL with the correct
  child gas and rollback; host capacity/configuration errors remain distinct
  from consensus invalidity. Test exact gas and return-data propagation.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Nested revert, local resource failure, logs/refund rollback and independent state-root comparisons.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Transaction execution is deterministic without yet applying system or delegation extensions.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.135.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.136.0 - EIP-7702 State Application

Status: planned; internal signed tag, publication at v0.140.0.

Goal: apply authorization and delegation effects at the correct checkpoints.

Scope: implementation pass. Depends on v0.135.0. The retained
workstream contract at v0.137.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement authorization order/skip/refund/nonces, persistent pre-execution effects and one-hop delegated call resolution.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official authorization/delegation fixtures, reverted calls, clearing storage and precompile-target cases.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Call reverts cannot undo protocol-persistent authorization effects.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.136.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.137.0 - State Transition And Journaling Completion

Status: planned; internal signed tag, publication at v0.140.0.

Goal: A complete block transition can be computed first party.

Scope: completion and integration pass. Depends on v0.136.0.
The implementation passes v0.135.0 through v0.136.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
block transition composition and reward/system-operation interfaces; request execution is closed by the following system-operation milestone.

Deliverables:

- Execute ordered transactions, commit/revert journaled state, apply rewards/system operations, and emit deterministic outcomes.
- Derive block gas, EVM memory, and precompile work from the immutable parent/
  candidate/fork context and supplied gas; operational caps may suspend
  readiness but cannot redefine validity.
- Transaction, call, and precompile scopes borrow the minimum sealed block
  rules/environment and snapshot lease without cloning parent evidence or
  recursively owning caller contexts; final-scope drop releases leases and
  arenas deterministically.
- Transaction, proof, precompile, and system-operation scopes consume only
  `v0.100.0` parent-authorized evidence reservations; rollback, cancellation,
  local failure, and successful completion release unused child reservations
  exactly once without resetting block-wide accounting.
- evidence-arena child borrows and generation handles follow `v0.106.0` so
  journal rollback, nested calls, cancellation, and worker transfer cannot
  release/reuse storage while a scope still references it.
- Implement complete EIP-7702 state application: process authorization tuples
  before transaction execution after sender-nonce increment, warm recovered
  authorities, apply skip-instead-of-reject tuple rules, account refunds,
  delegation writes or clearing, and authority nonce increments in list order.
- Keep valid EIP-7702 authorization effects outside the transaction-execution
  revert checkpoint, including when the subsequent call fails or reverts.
- Resolve exactly one EIP-7702 delegation hop for transaction destinations and
  `CALL`, `CALLCODE`, `DELEGATECALL`, and `STATICCALL`, with fork-correct warm/
  cold charging, precompile-target behavior, and direct code-inspection rules.

Verification:

- State-transition fixtures, nested revert tests, crash-free bounded execution.
- Million-depth synthetic context construction remains constant-size, child-
  creation clone counters remain zero, and cancellation/unwind/drop releases
  every scoped lease exactly once.
- Focused pre-implementation review of `StateJournal`, context ownership,
  rollback, and evidence-slot interaction.
- Nested evidence-capability conservation, rollback, cancellation, and
  exactly-once release tests across every transition stage;
- live-borrow rollback/cancellation, stale-generation, and cross-worker
  transfer races with unchanged state and evidence outcomes.
- Official EIP-7702 transaction/state fixtures covering duplicate authorities,
  invalid tuple skips, delegation clearing, refunds, persistent effects after
  execution failure, delegated transaction origins/destinations, one-hop loop
  handling, code-reading behavior, and precompile delegation targets.
- Differential state-root, receipt, gas, refund, access-set, code, and nonce
  comparisons against independent clients.

Exit criteria:

- A complete block transition can be computed first party.
- EIP-7702 transactions apply and execute delegation semantics completely
  rather than stopping at decode, signature, or context-validity proof.
- `v0.137.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.138.0 - System Contract Calls

Status: planned; internal signed tag, publication at v0.140.0.

Goal: execute block-system calls in protocol order.

Scope: implementation pass. Depends on v0.137.0. The retained
workstream contract at v0.139.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement beacon-root/history system calls and request-contract invocation with exact caller/gas/error semantics.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Pre/post-block ordering, missing code, call failure and state persistence vectors.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- System calls have a tested first-party path before request-root integration.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.138.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.139.0 - System Operations And Execution Requests Completion

Status: planned; internal signed tag, publication at v0.140.0.



Goal: implement every consensus-critical system operation and execution
request with explicit block-order, rollback, persistence, and header binding.

Scope: completion and integration pass. Depends on v0.138.0.
The implementation passes v0.138.0 through v0.138.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
deposit/withdrawal/consolidation request extraction, canonical ordering and header/Engine root binding.

Deliverables:

- EIP-4788 beacon-root and EIP-2935 historical-block-hash system calls;
- EIP-6110 deposit, EIP-7002 withdrawal, and EIP-7251 consolidation request
  extraction and processing;
- EIP-7685 canonical request ordering, encoding, empty-list behavior, and
  `requestsHash` calculation;
- exact pre-block/post-block placement, system caller/address, gas, code
  validation, rollback, and state-persistence rules;
- Engine payload encoding and execution-header/request-root binding.
- system-operation and execution-request validation consumes explicit
  `v0.100.0` block-derived child reservations and cannot independently mint or
  reset evidence capacity.

Verification:

- Official execution-spec and execution-apis fixtures for each EIP;
- exact ordering tests around transactions, withdrawals, rewards, and other
  system operations;
- child-reservation exhaustion, invalidity composition, rollback, and
  exactly-once release tests for every system operation and request;
- invalid/missing system-contract code, revert, empty request, reordered
  request, wrong header root, and payload mismatch tests;
- differential state, receipt, request, and header-root checks against Geth,
  Besu, and Nethermind.

Exit criteria:

- Current-fork system calls and requests are complete named transition stages,
  not an implied part of generic system-operation prose.
- `v0.139.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.140.0 - Receipts Logs Bloom And Withdrawals

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Post-execution outputs match consensus serialization and accounting rules.

Scope: bounded milestone. Depends on v0.139.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Receipt construction, cumulative gas, status/root rules, logs bloom, withdrawal application, and execution request outputs.

Verification:

- Receipt/blockchain fixtures and root comparison.

Exit criteria:

- Post-execution outputs match consensus serialization and accounting rules.
- `v0.140.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.141.0 - Canonical Mutable MPT Builder

Status: planned; internal signed tag, publication at v0.145.0.

Goal: construct and update canonical trie nodes independently of block roots.

Scope: implementation pass. Depends on v0.140.0. The retained
workstream contract at v0.142.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement insert/delete/branch collapse, inline/hash boundaries and snapshot-bound node storage.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official trie vectors, random mutation oracle, deletion collapse and missing-node tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Mutation roots match an independent trie before all root domains are integrated.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.141.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.142.0 - Trie Construction And Root Computation Completion

Status: planned; internal signed tag, publication at v0.145.0.

Goal: The crate computes all execution-layer Merkle Patricia roots it validates.

Scope: completion and integration pass. Depends on v0.141.0.
The implementation passes v0.141.0 through v0.141.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
account/storage/transaction/receipt root adapters over the admitted mutable trie.

Deliverables:

- First-party account, storage, transaction, and receipt trie builders with canonical node encoding and root calculation.

Verification:

- TrieTests, mutation/property tests, proof round trips, fuzzing.

Exit criteria:

- The crate computes all execution-layer Merkle Patricia roots it validates.
- `v0.142.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.143.0 - KZG Trusted Setup Boundary

Status: planned; internal signed tag, publication at v0.145.0.

Goal: No blob proof runs against implicit or unverified setup material.

Scope: bounded milestone. Depends on v0.142.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Versioned setup format, checksum/provenance, bounded loading, validation, and backend-independent setup handles.

Verification:

- Official setup checks, corruption/truncation tests, reproducibility report.

Exit criteria:

- No blob proof runs against implicit or unverified setup material.
- `v0.143.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.144.0 - KZG Scalar Field And FFT

Status: planned; internal signed tag, publication at v0.145.0.

Goal: admit the polynomial arithmetic used by blob commitments.

Scope: implementation pass. Depends on v0.143.0. The retained
workstream contract at v0.145.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement scalar canonicality, roots of unity, FFT/IFFT and bounded reusable workspace.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent scalar/FFT vectors, round trips, wrong domains and workspace failures.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Polynomial transforms match external oracles before proof construction.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.144.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.145.0 - KZG Field And Polynomial Core Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: KZG arithmetic foundations are first party and independently verified.

Scope: completion and integration pass. Depends on v0.144.0.
The implementation passes v0.144.0 through v0.144.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
polynomial evaluation and blob-domain/workspace integration over admitted scalar/FFT arithmetic.

Deliverables:

- First-party BLS scalar field, polynomial evaluation, roots of unity, FFT/IFFT, and bounded workspace policy.

Verification:

- Algebra properties, independent vectors, constant-bound and performance tests.

Exit criteria:

- KZG arithmetic foundations are first party and independently verified.
- `v0.145.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.146.0 - KZG Single Commitments And Proofs

Status: planned; internal signed tag, publication at v0.150.0.

Goal: create and verify one blob or evaluation proof before batching.

Scope: implementation pass. Depends on v0.145.0. The retained
workstream contract at v0.148.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement first-party commitments, proof creation/verification and versioned hashes using verified setup handles.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official KZG vectors, setup substitution, invalid fields and wrong evaluation points.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Single-proof behavior matches independent implementations.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.146.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.147.0 - KZG Batch Soundness And Isolation

Status: planned; internal signed tag, publication at v0.150.0.

Goal: batch proof verification without unsafe coefficients or member attribution.

Scope: implementation pass. Depends on v0.146.0. The retained
workstream contract at v0.148.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement reviewed transcript/randomness, bounded batch work and member isolation under parent evidence reservations.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Mixed valid/invalid inputs, coefficient reuse, entropy faults and isolation-budget exhaustion.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A failed batch cannot identify a member without individual proof.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.147.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.148.0 - KZG Commitments And Proofs Completion

Status: planned; internal signed tag, publication at v0.150.0.

Goal: Blob commitments and proofs are cryptographically executable, not descriptors.

Scope: completion and integration pass. Depends on v0.147.0.
The implementation passes v0.146.0 through v0.147.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
KZG setup/single/batch evidence integration and consumer admission.

Deliverables:

- Blob commitments, proof creation/verification, batch verification, and
  versioned-hash derivation;
- nonzero attacker-unpredictable random coefficients or a reviewed
  domain-separated transcript covering every statement and fork/domain input;
- fail-closed entropy/transcript handling, bounded batch-failure isolation, and
  prohibition of attacker-controlled or cross-batch coefficient reuse;
- batch failure returns `v0.93.0` `BatchContainsInvalid`; member-specific
  object invalidity requires successful bounded individual isolation, while
  local isolation failure remains retryable and non-attributable;
- follow `v0.100.0`: reserve one batch-result slot plus an explicit configured
  maximum of member child slots before isolation; evidence cardinality is
  independent of batch size, isolation stops locally at capacity, and only
  filled member slots permit caching or attribution;
- expose bounded isolation only through `v0.102.0`
  `BatchIsolateUpTo<N>`; members beyond `N` remain unattributed, and changing
  `N` cannot change the batch's cryptographic validity result;
- cache identities include complete message, setup, domain, fork, and
  validation-level context.

Verification:

- Official EIP-4844 fixtures, differential vectors, malformed/batch fuzzing;
- adversarial coefficient-reuse/control, transcript-collision, entropy-failure,
  mixed-context cache, bounded-isolation, slot-pressure, and maximum-evidence-
  cardinality tests;
- cross-`N` result invariance and beyond-limit non-attribution tests;
- high-contention batch verification/isolation benchmarks against the internal
  `v0.104.0` evidence-disabled baseline, with allocation, contention, retained-
  memory, and evidence-overhead thresholds.

Exit criteria:

- Blob commitments and proofs are cryptographically executable, not descriptors.
- `v0.148.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.149.0 - Point-Evaluation Precompile Execution

Status: planned; internal signed tag, publication at v0.150.0.

Goal: The precompile is consensus-compatible for all claimed forks.

Scope: bounded milestone. Depends on v0.148.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Admit the EIP-4844 point-evaluation precompile through verified KZG setup and exact gas/output/error behavior.

Verification:

- Official precompile vectors, gas ordering, fail-closed setup tests.

Exit criteria:

- The precompile is consensus-compatible for all claimed forks.
- `v0.149.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.150.0 - Blob Transaction And Block Integration

Status: planned; public crates.io checkpoint after cumulative review.

Goal: EIP-4844 validity is complete from transaction through block transition.

Scope: bounded milestone. Depends on v0.149.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Enforce hash version/count, sidecar consistency, blob gas/base fee, commitments, proofs, and block limits.

Verification:

- Transaction/block fixtures and adversarial sidecar tests.

Exit criteria:

- EIP-4844 validity is complete from transaction through block transition.
- `v0.150.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.151.0 - EOF Format And Static Validation

Status: planned; internal signed tag, publication at v0.155.0.

Goal: EOF bytecode is admitted only after complete static validation.

Scope: bounded milestone. Depends on v0.150.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Implement versioned EOF containers, section/type rules, validation stack, and fork gating.

Verification:

- Official EOF validation suite and malformed-structure fuzzing.

Exit criteria:

- EOF bytecode is admitted only after complete static validation.
- `v0.151.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.152.0 - EOF Control Flow And Execution

Status: planned; internal signed tag, publication at v0.155.0.

Goal: Valid EOF containers execute with fork-correct semantics.

Scope: bounded milestone. Depends on v0.151.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Implement EOF instructions, validated jumps/calls, stack contracts, data access, gas, and execution semantics.

Verification:

- Official execution vectors and differential tests.

Exit criteria:

- Valid EOF containers execute with fork-correct semantics.
- `v0.152.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.153.0 - EOF Creation And State Transition

Status: planned; internal signed tag, publication at v0.155.0.

Goal: EOF is complete at transaction and block level for claimed forks.

Scope: bounded milestone. Depends on v0.152.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Integrate EOF deployment, init containers, code validation, creation rules, receipts, and state changes.

Verification:

- Blockchain/state fixtures covering deployment and rejection.

Exit criteria:

- EOF is complete at transaction and block level for claimed forks.
- `v0.153.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.154.0 - Current Fork Manifest Admission

Status: planned; internal signed tag, publication at v0.155.0.

Goal: Every current fork claim maps to pinned rules and fixtures rather than a hand-maintained name list.

Scope: bounded milestone. Depends on v0.153.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Generate a reviewed rule manifest from pinned execution/consensus specs for Prague/Pectra, Osaka/Fusaka/Fulu, and then-active Glamsterdam, Hegotá, Gloas, or successor work as applicable.
- Reconcile the September 2026 admission requirements above, withdrawn
  proposals and separately negotiated networking versions.

Verification:

- Source-lock drift check and feature-by-feature conformance matrix.

Exit criteria:

- Every current fork claim maps to pinned rules and fixtures rather than a hand-maintained name list.
- `v0.154.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.155.0 - Osaka ModExp Fork Rules

Status: planned; public crates.io checkpoint after cumulative review.

Goal: complete the explicitly assigned Osaka ModExp changes.

Scope: implementation pass. Depends on v0.154.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-7823 length admission and EIP-7883 pricing under exact fork context; preserve historical EIP-198/2565 behavior.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official boundary vectors and three-client gas/output differentials around activation.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- ModExp semantics are correct on both sides of the fork.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.155.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.156.0 - Glamsterdam Transaction And Access Pricing

Status: planned; internal signed tag, publication at v0.160.0.

Goal: implement the admitted intrinsic/access/calldata pricing without changing state-gas settlement yet.

Scope: implementation pass. Depends on v0.155.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-2780, 7976, 7981 and 8038 accounting in explicit staged fork modules.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent pricing vectors for new/funded/existing accounts, access lists and calldata floors.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- The new schedule is tested but not globally enabled until integration.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.156.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.157.0 - State Gas Reservoir Accounting

Status: planned; internal signed tag, publication at v0.160.0.

Goal: implement the second gas dimension and cross-frame ownership.

Scope: implementation pass. Depends on v0.156.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-8037 charge timing, reservoirs, spills, child merges, rollback and exceptional halts.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official cross-frame refund/spill vectors, conservation properties and adversarial nested calls.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- State-gas ownership survives revert/halt without creating or losing gas.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.157.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.158.0 - Pre-Refund Block And Receipt Gas

Status: planned; internal signed tag, publication at v0.160.0.

Goal: bind transaction results to EIP-7778 block accounting.

Scope: implementation pass. Depends on v0.157.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement pre-refund block admission and receipt gas projection without confusing execution/state gas or user refunds.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Receipt/accounting vectors, cumulative boundaries, reverted calls and independent client outputs.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Block validity and receipt gas derive from the correct separate counters.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.158.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.159.0 - Current Fork Opcode And Deployment Rules

Status: planned; internal signed tag, publication at v0.160.0.

Goal: implement the admitted opcode and code-size changes.

Scope: implementation pass. Depends on v0.158.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement SLOTNUM, stack extensions, contract/initcode limits and deterministic factory deployment under exact fork configuration.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Per-opcode stack/gas vectors, code-size edges, deployment roots and historical negative tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- New instructions and deployments are executable only for admitted forks.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.159.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.160.0 - Transfer Logs And SELFDESTRUCT Integration

Status: planned; public crates.io checkpoint after cumulative review.

Goal: implement transfer observability and changed balance lifecycle.

Scope: implementation pass. Depends on v0.159.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-7708 log emission/blooms and EIP-8246 balance behavior including system calls and funded CREATE2 remnants.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent receipts/logs, zero-value/revert cases and account-lifecycle fixtures.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No missing/double logs or unintended historical balance changes remain.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.160.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.161.0 - Block Access List Codec And Commitments

Status: planned; internal signed tag, publication at v0.165.0.

Goal: implement canonical EIP-7928 data and root formation.

Scope: implementation pass. Depends on v0.160.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement bounded BAL models, ordering/index encoding and commitment construction; do not treat syntax as execution evidence.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Nonminimal scalars, duplicate/order/index cases and external commitment vectors.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Canonical BAL bytes/roots match independent clients.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.161.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.162.0 - Block Access List Execution Validation

Status: planned; internal signed tag, publication at v0.165.0.

Goal: validate actual block accesses against committed BALs.

Scope: implementation pass. Depends on v0.161.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Track reverted reads, recreated accounts and transaction-indexed changes; reject missing/extra state claims with object evidence.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official omitted-slot/account/max-nonce and recreated-account tests, state-root differentials.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A syntactically valid BAL cannot hide a semantic execution mismatch.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.162.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.163.0 - Frame Transaction Codec And Authorization

Status: planned; internal signed tag, publication at v0.165.0.

Goal: admit EIP-8141 wire and signing domains separately from execution.

Scope: implementation pass. Depends on v0.162.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement version-pinned frame models, canonical codecs, authorization hashes and signature/scope checks without enabling transaction execution.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official frame vectors, wrong domain/signature, malformed atomic-batch scopes and round trips.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Frame parsing never implies execution or consensus validity.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.163.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.164.0 - Frame Execution And Gas Settlement

Status: planned; internal signed tag, publication at v0.165.0.

Goal: execute admitted frame transactions with exact scope and gas ownership.

Scope: implementation pass. Depends on v0.163.0. The retained
workstream contract at v0.165.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement frame entry/approval, dispatch/precompiles, rollback, floors and state-gas settlement using previously admitted journals.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Nested failure, signature-data access, resolved target charging and independent frame execution vectors.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Every supported frame result and gas counter matches the pinned specification.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.164.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.165.0 - Current Fork Execution Changes Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: No current execution-fork rule remains a descriptor or silent unsupported path.

Scope: completion and integration pass. Depends on v0.164.0.
The implementation passes v0.155.0 through v0.164.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
cross-EIP activation and historical-regression integration; newly selected rules require an inserted minor, not implementation in this gate.

Deliverables:

- Implement all opcodes, precompiles, system contracts, gas changes, request
  types, and state-transition changes in the admitted current manifest,
  including Osaka EIP-7823 ModExp input limits and EIP-7883 ModExp repricing.
- Implement the gas, opcode, BAL, transfer-log and frame-execution obligations
  above after revision/fork admission; split into new minor versions before
  coding if this cannot fit one pentest pass.

Verification:

- Official tests per EIP/fork, client differential suite, pentest.

Exit criteria:

- No current execution-fork rule remains a descriptor or silent unsupported path.
- `v0.165.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.166.0 - Complete Execution Fixture Gate

Status: planned; internal signed tag, publication at v0.170.0.

Goal: All claimed historical and current execution behavior has fixture evidence.

Scope: bounded milestone. Depends on v0.165.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Run TransactionTests, BlockchainTests, GenesisTests, TrieTests, DifficultyTests, EOF tests, state tests, and current successor suites.

Verification:

- Generated pass/fail/skip report with zero unexplained skips.

Exit criteria:

- All claimed historical and current execution behavior has fixture evidence.
- `v0.166.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.167.0 - Execution Differential And Performance Gate

Status: planned; internal signed tag, publication at v0.170.0.

Goal: The first-party engine is correct and operationally bounded enough for higher layers.

Scope: bounded milestone. Depends on v0.166.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Differentially compare execution status, exceptional halt, gas used, refunds,
  logs, bloom, receipts, receipt root, transaction root, withdrawal root,
  request root, state root, created/deleted accounts, code, state diffs, and
  traces;
- use official execution-spec fixtures plus an implementation-diverse Geth,
  Besu, and Nethermind matrix; REVM and Reth remain useful development oracles
  but do not count as the only independent implementations;
- establish CPU, memory, stack, deterministic work, and gas benchmarks;
- benchmark complete valid blocks across empty, small, typical, high-count,
  and maximum admitted work profiles against the internal `v0.104.0`
  evidence-disabled baseline, enforcing reservation, allocation, contention,
  clone, code-size, peak/retained-memory, and overhead thresholds.

Verification:

- Reproducible differential corpus and regression thresholds;
- evidence-enabled/evidence-disabled full-block reports with one amortized
  parent reservation and constant child-bookkeeping evidence;
- zero unexplained field-level mismatches or skipped official vectors.

Exit criteria:

- The first-party engine is correct and operationally bounded enough for higher layers.
- `v0.167.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.168.0 - Snapshot-Bound Execution Caches And Prefetch

Status: planned; internal signed tag, publication at v0.170.0.



Goal: add node-scale state and code acceleration without allowing cache
identity, staleness, or prefetched trust to affect validity.

Scope: bounded milestone. Depends on v0.167.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Account, storage, and code caches bound to chain/genesis, fork rules,
  state-root/snapshot, object key, and validation level;
- code entries keyed by code hash and storage entries keyed by snapshot/root,
  address, and slot;
- deterministic bounded eviction, retained-memory ceilings, and
  transaction/block reset policy;
- atomic invalidation or rebinding on reorg and fork-rule changes;
- access-list and trace-driven prefetch whose results remain untrusted until
  validated against the active snapshot;
- cache/prefetch scopes hold bounded snapshot handles rather than recursive
  validation contexts, and expired leases cannot be revived by persisted hints.

Verification:

- Cross-root, cross-fork, cross-chain, and validation-level substitution tests;
- reorg/fork-transition races, deterministic eviction, memory-pressure, and
  stale-prefetch tests, deterministic lease-release checks, and expired-handle
  non-revival across restart;
- cached/uncached differential execution over the complete fixture corpus.

Exit criteria:

- Caching and prefetch improve throughput without creating a path for stale or
  weakly validated data to influence consensus execution.
- `v0.168.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.169.0 - Speculative Read Write Sets And Conflicts

Status: planned; internal signed tag, publication at v0.170.0.

Goal: prove conflict detection before parallel block execution.

Scope: implementation pass. Depends on v0.168.0. The retained
workstream contract at v0.170.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement complete snapshot-bound dependencies and a sequential simulator for worker conflict/commit decisions.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Coinbase, nonce, storage, delegation, creation and system-call conflict mutations compared to serial order.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- The model detects every seeded state dependency before parallel workers are enabled.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.169.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.170.0 - Deterministic Speculative Parallel Execution Completion

Status: planned; public crates.io checkpoint after cumulative review.



Goal: make parallel transaction execution an optional optimization that is
provably equivalent to sequential block order.

Scope: completion and integration pass. Depends on v0.169.0.
The implementation passes v0.169.0 through v0.169.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
bounded worker scheduling, original-order commit and deterministic sequential fallback using proven dependency sets.

Deliverables:

- Snapshot-bound speculative execution with explicit read/write sets;
- deterministic conflict detection, sequential-order validation and commit,
  bounded worker/memory reservations, cancellation, and cache integration;
- evidence arenas consume the same `v0.72.0` worker/memory capability tree,
  use `v0.106.0` generation-safe transfer handles where work crosses workers,
  and backpressure/fall back sequentially before arena exhaustion can affect
  validity;
- automatic deterministic fallback to sequential execution on conflict,
  ambiguity, resource exhaustion, or scheduler failure;
- read/write sets include sender nonce/balance, coinbase fee credit, account
  creation/collision, code deposit, fork-specific `SELFDESTRUCT`, EIP-7702
  delegation/authorization nonces, transient storage, access warmth, original
  storage values, block-level system contracts/requests, precompile-visible
  state/environment, logs, receipts, cumulative gas, and request ordering;
- an optional deferred coinbase fee-delta reduction commits in original
  transaction order and falls back when equivalence cannot be proven;
- no parallel scheduling decision or conflict-detection failure can alter
  transaction validity or final block outcome.

Verification:

- Parallel/sequential differential status, gas, receipts, logs, requests,
  traces, and final-root comparisons;
- adversarial conflict graphs, hidden dependencies, reverts, selfdestruct,
  EIP-7702, system-operation, reorg, cancellation, and fault tests;
- dedicated dependency tests for every read/write-set class plus direct versus
  deferred coinbase-credit equivalence;
- deterministic replay across worker counts and scheduling seeds;
- arena-capacity, live-borrow cancellation/transfer, stale-generation, worker-
  pool-absent/default, and sequential-fallback equivalence tests.

Exit criteria:

- Every admitted parallel result matches sequential execution exactly, and
  every uncertain case falls back without changing consensus behavior.
- `v0.170.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.171.0 - Inspector And Hook Framework

Status: planned; internal signed tag, publication at v0.175.0.

Goal: Tooling can observe execution without changing consensus results.

Scope: bounded milestone. Depends on v0.170.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded opcode/call/state/log inspectors, cancellation, filtering, and no-op zero-cost path.

Verification:

- Hook-order tests, cancellation tests, overhead benchmarks, fuzzing.

Exit criteria:

- Tooling can observe execution without changing consensus results.
- `v0.171.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.172.0 - Trace And State-Diff Models

Status: planned; internal signed tag, publication at v0.175.0.

Goal: Execution evidence is usable by debuggers and analysis tools.

Scope: bounded milestone. Depends on v0.171.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Call traces, opcode traces, state/access diffs, gas profiles, revert data, and client-compatible trace projections.

Verification:

- Cross-client trace fixtures, redaction and size-limit tests.

Exit criteria:

- Execution evidence is usable by debuggers and analysis tools.
- `v0.172.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.173.0 - Deterministic Simulation And Overrides

Status: planned; internal signed tag, publication at v0.175.0.

Goal: Transactions and bundles can be simulated safely before signing or broadcast.

Scope: bounded milestone. Depends on v0.172.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- `eth_call`-style execution, block/state overrides, bundles, access-list generation, trace/debug RPC models, and deterministic reports.

Verification:

- Override fixtures, repeatability tests, bounded workload tests.

Exit criteria:

- Transactions and bundles can be simulated safely before signing or broadcast.
- `v0.173.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 11: Providers And Transaction Lifecycle

### v0.174.0 - Typed Standard Execution RPC Methods

Status: planned; internal signed tag, publication at v0.175.0.

Goal: make core eth methods strongly typed and policy-bound.

Scope: implementation pass. Depends on v0.173.0. The retained
workstream contract at v0.176.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement standard execution parameters/results and sealed method metadata, canonical JSON and anchored trust classification.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Pinned execution-api fixtures, duplicate keys, quantities, oversize results and type substitution.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Core methods require neither arbitrary JSON construction nor caller-controlled security metadata.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.174.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.175.0 - Typed Diagnostic RPC Namespaces

Status: planned; public crates.io checkpoint after cumulative review.

Goal: separate debug trace and txpool schemas from ordinary reads.

Scope: implementation pass. Depends on v0.174.0. The retained
workstream contract at v0.176.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement bounded diagnostic methods and explicit administrative/test classification; share canonical parsing but not admission policy.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Namespace/schema fixtures, secret redaction, forbidden method exposure and response-work accounting.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Privileged diagnostics cannot become public through generic dispatch.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.175.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.176.0 - Typed RPC Method Surface Completion

Status: planned; internal signed tag, publication at v0.180.0.

Goal: Callers no longer assemble core RPC methods from untyped JSON values.

Scope: completion and integration pass. Depends on v0.175.0.
The implementation passes v0.174.0 through v0.175.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
method catalogue coverage and trust/policy integration; concrete Engine transport remains in its own milestones.

Deliverables:

- Sealed typed method contracts for execution, debug, trace, txpool, Engine,
  and supported extension namespaces;
- each method binds its parameter type, response type, trust classification,
  response-size ceiling, idempotence, retry policy, and fork availability;
- method response and proof-size ceilings are provider/RPC
  `OperationalLimits`; exceeding them refuses that operation without declaring
  the underlying account, block, trie data, or cryptographic proof invalid;
- JSON request/response parsing consumes the `v0.88.0` parent ledger, rejects
  duplicate object keys, and bounds structural depth, node count, strings,
  arrays, allocations, and output in addition to raw bytes;
- canonical JSON-RPC quantity parsing rejects leading zeroes except `0x0`,
  distinguishes quantities from byte strings, and defines exact odd-length
  hex and casing policy;
- reject overflow, invalid hex, floats, exponents, and type-domain
  substitutions; charge decoded byte length before hex allocation;
- callers cannot assemble core methods from untyped JSON values or override a
  method's security metadata.

Verification:

- Official execution-apis fixtures, serde snapshots, duplicate/unknown-field
  policy tests, structural-depth/node complexity oracles, and allocation-before-
  reservation rejection tests;
- quantity/byte-string canonicality, odd hex, casing, overflow, float,
  exponent, and decoded-byte accounting matrices;
- valid-proof-over-endpoint-policy and retry-with-larger-work-budget tests.

Exit criteria:

- Callers no longer assemble core RPC methods from untyped JSON values.
- `v0.176.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.177.0 - Runtime-Neutral Transport Traits

Status: planned; internal signed tag, publication at v0.180.0.

Goal: Provider logic is independent of HTTP stack and async runtime choice.

Scope: bounded milestone. Depends on v0.176.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded request, response, batch, subscription, timeout, cancellation, and
  transport-error contracts without selecting a runtime;
- associated-future, generic-future, or poll-based service interfaces that do
  not require `async_trait` allocation;
- adapters prove cancellation and backpressure semantics without granting the
  transport authority to validate or promote Ethereum data.

Verification:

- Mock transport conformance suite and object-safety/no_std checks.

Exit criteria:

- Provider logic is independent of HTTP stack and async runtime choice.
- `v0.177.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.178.0 - HTTP Provider

Status: planned; internal signed tag, publication at v0.180.0.

Goal: A production HTTP provider exists without entering the default graph.

Scope: bounded milestone. Depends on v0.177.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Optional reviewed HTTP/TLS adapters, authentication redaction, payload
  limits, timeout policy, and endpoint allowlists;
- redirects disabled by default; when enabled, every hop repeats scheme,
  origin, hostname, DNS, address-range, credential, and allowlist validation;
- DNS rebinding and hostname re-resolution policy plus explicit opt-in proxy
  and environment-proxy behavior;
- credentials are never forwarded across origins, and URL credentials,
  authorization headers, cookies, proxy credentials, and logs are redacted.

Verification:

- Malicious server fixtures, redirect/rebinding/proxy/credential-leak cases,
  TLS/config matrix, cancellation/load tests.

Exit criteria:

- A production HTTP provider exists without entering the default graph.
- `v0.178.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.179.0 - WebSocket Provider

Status: planned; internal signed tag, publication at v0.180.0.

Goal: Long-lived subscriptions fail explicitly and cannot grow memory without bound.

Scope: bounded milestone. Depends on v0.178.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Subscription lifecycle, bounded queues, reconnect/resubscribe policy,
  missed-event signaling, backpressure, explicit WebSocket origin, redirect,
  DNS re-resolution, proxy, and cross-origin credential policy.

Verification:

- Disconnect/reorder/flood, origin/redirect/rebinding/proxy/credential tests and
  local-node integration.

Exit criteria:

- Long-lived subscriptions fail explicitly and cannot grow memory without bound.
- `v0.179.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.180.0 - Native IPC Transport Adapters

Status: planned; public crates.io checkpoint after cumulative review.

Goal: admit native socket and pipe trust checks independently of browsers.

Scope: implementation pass. Depends on v0.179.0. The retained
workstream contract at v0.181.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement Unix socket ownership/peer identity and Windows named-pipe ACL/impersonation policy behind optional adapters.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Path/symlink swaps, untrusted parent directories, peer substitution and cancellation tests on actual targets.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Local IPC cannot silently trust an attacker-owned endpoint.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.180.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.181.0 - IPC Custom And EIP-1193 Transports Completion

Status: planned; internal signed tag, publication at v0.185.0.

Goal: Desktop, mobile, browser, and embedded integrators can supply an appropriate transport.

Scope: completion and integration pass. Depends on v0.180.0.
The implementation passes v0.180.0 through v0.180.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
caller-supplied and browser EIP-1193/WASM adapters over the existing transport contract.

Deliverables:

- Unix IPC with socket ownership/mode checks, parent-directory trust, symlink/
  path-swap protection, and peer credential verification;
- Windows named-pipe ACL, owner, peer identity, impersonation, and path policy;
- caller-supplied transport, browser EIP-1193, and WASM adapter boundaries.

Verification:

- Platform matrix, Unix ownership/symlink race tests, Windows ACL/peer identity
  tests, browser mock tests, framing/flood tests.

Exit criteria:

- Desktop, mobile, browser, and embedded integrators can supply an appropriate transport.
- `v0.181.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.182.0 - RPC IDs Batching And Cancellation

Status: planned; internal signed tag, publication at v0.185.0.

Goal: Concurrent and batched calls cannot be confused or left unbounded.

Scope: bounded milestone. Depends on v0.181.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Collision-safe typed ID domains, duplicate request/response ID rejection,
  invalid/null/fractional/out-of-range ID policy, batch correlation, partial
  failures, cancellation races, concurrency caps, and response size limits.

Verification:

- Reordering/duplication/invalid-ID/flood fuzzing and race tests.

Exit criteria:

- Concurrent and batched calls cannot be confused or left unbounded.
- `v0.182.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.183.0 - Method Validation And Block Consistency

Status: planned; internal signed tag, publication at v0.185.0.

Goal: Typed RPC data is structurally and contextually checked before promotion.

Scope: bounded milestone. Depends on v0.182.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Method-specific response validation, chain identity checks, EIP-1898 block
  references, and one immutable chain/block anchor across every multi-call
  operation;
- explicit promotion from untrusted wire responses into anchored validated
  values, with mixed-head and mixed-chain results rejected before composition.

Verification:

- Malicious/inconsistent provider fixtures and quorum disagreement cases.

Exit criteria:

- Typed RPC data is structurally and contextually checked before promotion.
- `v0.183.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.184.0 - Provider Middleware

Status: planned; internal signed tag, publication at v0.185.0.

Goal: Operational policy is composable without hidden retries or data leakage.

Scope: bounded milestone. Depends on v0.183.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded retries, rate limits, circuit breakers, caches, metrics, redaction, request classification, and policy composition.

Verification:

- Failure-injection and retry-amplification tests, cache correctness tests.

Exit criteria:

- Operational policy is composable without hidden retries or data leakage.
- `v0.184.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.185.0 - Quorum Verified And Traced Providers

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Trust policy changes the return type and evidence, not only a boolean setting.

Scope: bounded milestone. Depends on v0.184.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Distinct `Untrusted<T>`, `Quorum<T, Evidence>`, and `Verified<T, Anchor>`
  result domains rather than trust booleans;
- multi-provider quorum, proof-backed reads, finalized/safe policies,
  disagreement evidence, and tracing metadata;
- proof verification consumes an explicit `WorkBudget`; provider/RPC size
  policy remains separate from cryptographic proof validity, so a locally
  refused proof may be retried under a larger admitted budget;
- disagreement and trace evidence consumes bounded `EvidenceBudget` entries
  with compact provider identities, reason codes, and redacted diagnostics;
- no implicit conversion from trusted or quorum data into cryptographically
  verified data.

Verification:

- Byzantine provider simulations, proof/quorum fixtures, endpoint-policy/
  proof-validity separation, and bounded-work retry tests.

Exit criteria:

- Trust policy changes the return type and evidence, not only a boolean setting.
- `v0.185.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.186.0 - Transaction Request Builders

Status: planned; internal signed tag, publication at v0.190.0.

Goal: Invalid field combinations are rejected before RPC or signing.

Scope: bounded milestone. Depends on v0.185.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Fork-aware builders for every transaction family with explicit unset/derived/user-supplied field states.

Verification:

- Compile-fail builder tests and cross-type round trips.

Exit criteria:

- Invalid field combinations are rejected before RPC or signing.
- `v0.186.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.187.0 - Transaction Fillers

Status: planned; internal signed tag, publication at v0.190.0.

Goal: Automatic filling is observable, bounded, and never silently overwrites user intent.

Scope: bounded milestone. Depends on v0.186.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Chain ID, nonce, gas, fees, access list, blob fields, and authorization fillers with source/evidence records.

Verification:

- Concurrent nonce tests, hostile RPC fixtures, deterministic fill snapshots.

Exit criteria:

- Automatic filling is observable, bounded, and never silently overwrites user intent.
- `v0.187.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.188.0 - Blob Sidecars And Fee Markets

Status: planned; internal signed tag, publication at v0.190.0.

Goal: Blob transactions can be prepared end to end with first-party validation.

Scope: bounded milestone. Depends on v0.187.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Sidecar construction, fee-history interpretation, blob base-fee calculation, replacement policy, and KZG workflow integration.

Verification:

- Local-node blob tests and fee boundary vectors.

Exit criteria:

- Blob transactions can be prepared end to end with first-party validation.
- `v0.188.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.189.0 - Build Simulate Sign Broadcast Workflow

Status: planned; internal signed tag, publication at v0.190.0.

Goal: The common transaction lifecycle is available without bypassing validation evidence.

Scope: bounded milestone. Depends on v0.188.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- High-level `Draft -> Filled -> Simulated -> Approved -> SigningRequest ->
  SignedEnvelope -> Broadcast -> Confirmed` typestate workflow;
- validation and evidence remain attached at every transition;
- contract/provider bindings never hold signing authority or raw key material.

Verification:

- End-to-end local-node tests and fault injection at every transition.

Exit criteria:

- The common transaction lifecycle is available without bypassing validation evidence.
- `v0.189.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.190.0 - Pending Transaction Watcher

Status: planned; public crates.io checkpoint after cumulative review.

Goal: A broadcast transaction reaches a final, replaced, dropped, or timed-out terminal state explicitly.

Scope: bounded milestone. Depends on v0.189.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Receipt watching, configurable confirmations, safe/finalized heads, reorg detection, timeout, cancellation, and evidence.

Verification:

- Reorg/restart/disconnect simulations.

Exit criteria:

- A broadcast transaction reaches a final, replaced, dropped, or timed-out terminal state explicitly.
- `v0.190.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.191.0 - Replacement Cancellation And Drop Recovery

Status: planned; internal signed tag, publication at v0.195.0.

Goal: Stuck transactions can be managed without unsafe nonce assumptions.

Scope: bounded milestone. Depends on v0.190.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Fee-bump rules, cancellation transaction construction, competing hashes, dropped transaction detection, and nonce reconciliation.

Verification:

- Local-node replacement/reorg tests and adversarial provider cases.

Exit criteria:

- Stuck transactions can be managed without unsafe nonce assumptions.
- `v0.191.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.192.0 - Offline Signing Packages

Status: planned; internal signed tag, publication at v0.195.0.

Goal: Air-gapped and remote signers can participate without trusting provider serialization.

Scope: bounded milestone. Depends on v0.191.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Deterministic signing packages, human-review summaries, policy hooks, QR/file-safe encoding, and signed-result verification.

Verification:

- Golden packages, tamper tests, secret-redaction review.

Exit criteria:

- Air-gapped and remote signers can participate without trusting provider serialization.
- `v0.192.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.193.0 - Live Node Integration Matrix

Status: planned; internal signed tag, publication at v0.195.0.

Goal: Provider and lifecycle claims pass against real nodes, not only mocks.

Scope: bounded milestone. Depends on v0.192.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Self-managed Podman execution clients covering HTTP, WS, subscriptions, reorgs, blobs, traces, and transaction lifecycles.

Verification:

- Repeatable bring-up/tear-down scripts and CI/manual matrix.

Exit criteria:

- Provider and lifecycle claims pass against real nodes, not only mocks.
- `v0.193.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 12: Signers Wallets And Account Abstraction

### v0.194.0 - Signer Interface 2.0

Status: planned; internal signed tag, publication at v0.195.0.

Goal: Every signing request states exactly what domain and policy is being authorized.

Scope: bounded milestone. Depends on v0.193.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Runtime-neutral capability contracts consuming the sealed `v0.95.0`
  scheme separation: execution requests for transactions, EIP-712 data,
  personal messages, and EIP-7702 authorizations remain distinct from BLS
  consensus-duty requests;
- every request binds chain, signing domain, digest, review metadata, policy
  context, and signer capability evidence;
- signers return only public signatures and metadata, never raw secret access
  or `with_secret(&[u8])` callbacks.

Verification:

- Mock execution/consensus signer suites, compile-fail cross-scheme and
  transport-identity substitution tests, wrong-domain/chain tests, redaction
  audit.

Exit criteria:

- Every signing request states exactly what domain and policy is being authorized.
- `v0.194.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.195.0 - Local Secret Signer

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Local signing is usable but remains opt-in and security-reviewed.

Scope: bounded milestone. Depends on v0.194.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Optional local signer using the already admitted `v0.75.0..=v0.87.0`
  first-party secp256k1/ECDSA path and `v0.94.0` proof evidence as an
  `ExecutionSigner` only, in a separate opt-in crate or isolated worker,
  with locked/sanitized opaque secret ownership, deterministic signatures, and
  explicit export prohibition;
- in-place crypto operations, RAII sanitation guards, redacted non-Clone/
  non-Copy secret types, disabled core dumps, restricted IPC, and no raw keys
  retained in asynchronous futures;
- documentation promises isolation and reviewed best-effort erasure rather
  than absolute compiler/backend/process-wide zeroization.

Verification:

- KATs, low-s/recovery checks, memory-sanitization evidence, pentest.

Exit criteria:

- Local signing is usable but remains opt-in and security-reviewed.
- `v0.195.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.196.0 - Encrypted Keystore

Status: planned; internal signed tag, publication at v0.200.0.

Goal: Keystore handling is compatible and cannot silently admit unsafe cost settings.

Scope: bounded milestone. Depends on v0.195.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Web3 Secret Storage compatible import/export, parameter validation, bounded KDF work policy, password handling, and migration.

Verification:

- Official/independent vectors, malformed/KDF DoS tests, interoperability checks.

Exit criteria:

- Keystore handling is compatible and cannot silently admit unsafe cost settings.
- `v0.196.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.197.0 - BIP-39 Mnemonics

Status: planned; internal signed tag, publication at v0.200.0.

Goal: Mnemonic workflows are standards-compatible and explicitly secret-bearing.

Scope: bounded milestone. Depends on v0.196.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Entropy, checksum, language policy, seed derivation, passphrase handling, and sanitization boundaries.

Verification:

- Official vectors, normalization tests, memory handling review.

Exit criteria:

- Mnemonic workflows are standards-compatible and explicitly secret-bearing.
- `v0.197.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.198.0 - BIP-32 And BIP-44 Derivation

Status: planned; internal signed tag, publication at v0.200.0.

Goal: HD Ethereum accounts can be derived without external wallet-core logic.

Scope: bounded milestone. Depends on v0.197.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Hardened/non-hardened derivation, Ethereum paths, extended key handling, watch-only support, and path policy.

Verification:

- Official/independent vectors, invalid-child/path tests.

Exit criteria:

- HD Ethereum accounts can be derived without external wallet-core logic.
- `v0.198.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.199.0 - Remote Execution Signer Protocol

Status: planned; internal signed tag, publication at v0.200.0.

Goal: bind remote transaction signing to an authenticated exact request.

Scope: implementation pass. Depends on v0.198.0. The retained
workstream contract at v0.200.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement opaque signer handles, authenticated request context, replay/idempotency and verified public signature responses.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Wrong chain/key/digest, cancelled retries and compromised response tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Remote signing never exports keys or signs an unreviewed replacement request.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.199.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.200.0 - Remote Hardware HSM And KMS Signers Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: External key custody integrates through one auditable signer boundary.

Scope: completion and integration pass. Depends on v0.199.0.
The implementation passes v0.199.0 through v0.199.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
one named audited hardware/HSM/KMS adapter per admitted backend class; any additional backend gets a separate admission pass.

Deliverables:

- Capability-based adapters for hardware wallets, HSMs, KMS, and remote signing services with attestation metadata.

Verification:

- Mock protocol matrices, cancellation/timeouts, wrong-key/chain tests.

Exit criteria:

- External key custody integrates through one auditable signer boundary.
- `v0.200.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.201.0 - Signing Policy ERC-1271 And Multisig

Status: planned; internal signed tag, publication at v0.205.0.

Goal: Contract and policy authorization are first-class, not forced into EOA assumptions.

Scope: bounded milestone. Depends on v0.200.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Policy engine, spend/domain allowlists, ERC-1271 verification, threshold signature collections, and audit records.

Verification:

- Contract-wallet fixtures, policy bypass tests, malformed signature corpus.

Exit criteria:

- Contract and policy authorization are first-class, not forced into EOA assumptions.
- `v0.201.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.202.0 - Safe Workflows

Status: planned; internal signed tag, publication at v0.205.0.

Goal: Common multisig transactions can be built, reviewed, signed, and followed end to end.

Scope: bounded milestone. Depends on v0.201.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Safe transaction hashing, nonce/module/guard modeling, signature packing, service adapters, and execution tracking.

Verification:

- Safe reference vectors and local-contract integration.

Exit criteria:

- Common multisig transactions can be built, reviewed, signed, and followed end to end.
- `v0.202.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.203.0 - ERC-4337 Core

Status: planned; internal signed tag, publication at v0.205.0.

Goal: User operations have complete typed and cryptographic foundations.

Scope: bounded milestone. Depends on v0.202.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- UserOperation versions, hashing, validation data, gas fields, aggregation boundaries, and EntryPoint models.

Verification:

- Official account-abstraction vectors and EntryPoint integration.

Exit criteria:

- User operations have complete typed and cryptographic foundations.
- `v0.203.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.204.0 - Bundler RPC And UserOperation Lifecycle

Status: planned; internal signed tag, publication at v0.205.0.

Goal: implement submission and tracking separately from paymaster policy.

Scope: implementation pass. Depends on v0.203.0. The retained
workstream contract at v0.205.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement pinned bundler method versions, simulation and UserOperation tracking with EntryPoint/domain binding.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent bundler fixtures, gas estimates, replacement/reorg and invalid EntryPoint responses.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- UserOperations can complete a lifecycle without granting the bundler signing authority.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.204.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.205.0 - Bundler EntryPoint And Paymasters Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: ERC-4337 works end to end with explicit third-party trust boundaries.

Scope: completion and integration pass. Depends on v0.204.0.
The implementation passes v0.204.0 through v0.204.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
paymaster/aggregator policy, reputation and EntryPoint integration over the admitted bundler lifecycle.

Deliverables:

- Bundler RPC, simulation, submission/watch flows, paymaster data/policy, aggregator handling, and reputation/error models.

Verification:

- Local bundler/EntryPoint tests and hostile paymaster fixtures.

Exit criteria:

- ERC-4337 works end to end with explicit third-party trust boundaries.
- `v0.205.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.206.0 - Session Keys And Delegated Accounts

Status: planned; internal signed tag, publication at v0.210.0.

Goal: Delegated authorization is usable without weakening base signature and policy guarantees.

Scope: bounded milestone. Depends on v0.205.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Session-key policies, scoped permissions, revocation, EIP-7702 delegated-account workflows, and wallet/account-abstraction composition.

Verification:

- Expiry/revocation/domain tests and local-node workflows.

Exit criteria:

- Delegated authorization is usable without weakening base signature and policy guarantees.
- `v0.206.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 13: ABI Contracts And Application Standards

### v0.207.0 - ABI Type System

Status: planned; internal signed tag, publication at v0.210.0.

Goal: All standard ABI type shapes are represented without untyped strings.

Scope: bounded milestone. Depends on v0.206.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Canonical Solidity ABI types, tuples, arrays, functions, events, errors, selectors, and bounded dynamic-size policy.

Verification:

- Solidity differential vectors and type parser fuzzing.

Exit criteria:

- All standard ABI type shapes are represented without untyped strings.
- `v0.207.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.208.0 - ABI Encode Decode

Status: planned; internal signed tag, publication at v0.210.0.

Goal: ABI values encode/decode canonically under explicit resource limits.

Scope: bounded milestone. Depends on v0.207.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- First-party head/tail encoding and strict decoding with offset, overlap,
  padding, depth, count, string/byte, allocation, and output checks;
- ABI decoding consumes the `v0.88.0` parent work ledger and charges every
  offset traversal and structural node before following or allocating it.

Verification:

- Official/reference vectors, malformed-offset fuzzing, round trips, and
  offset/node/allocation complexity oracles.

Exit criteria:

- ABI values encode/decode canonically under explicit resource limits.
- `v0.208.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.209.0 - Artifact And Metadata Ingestion

Status: planned; internal signed tag, publication at v0.210.0.

Goal: Common build artifacts enter the SDK through validated owned models.

Scope: bounded milestone. Depends on v0.208.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded duplicate-key-rejecting JSON ingestion for ABI, bytecode, deployed
  bytecode, link references, compiler metadata, and source maps through the
  `v0.88.0` structural and allocation ledger.

Verification:

- Foundry/Hardhat/Solc artifact corpus and hostile JSON tests.

Exit criteria:

- Common build artifacts enter the SDK through validated owned models.
- `v0.209.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.210.0 - Contract Macros And Code Generation

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Users can obtain typed bindings without hand-written field glue.

Scope: bounded milestone. Depends on v0.209.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Audited procedural/codegen path for typed calls, returns, events, errors, and contract interfaces.

Verification:

- Compile tests, generated-code snapshots, semver and size checks.

Exit criteria:

- Users can obtain typed bindings without hand-written field glue.
- `v0.210.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.211.0 - Deployment And Linking

Status: planned; internal signed tag, publication at v0.215.0.

Goal: Contracts and libraries can be deployed through the validated transaction lifecycle.

Scope: bounded milestone. Depends on v0.210.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Constructor encoding, library linking, CREATE/CREATE2 address prediction, deployment simulation, broadcast, and receipt verification.

Verification:

- Local-node deployments and link-reference negative tests.

Exit criteria:

- Contracts and libraries can be deployed through the validated transaction lifecycle.
- `v0.211.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.212.0 - Events Filters And Reorg Streams

Status: planned; internal signed tag, publication at v0.215.0.

Goal: Event consumers can resume and handle reorganizations correctly.

Scope: bounded milestone. Depends on v0.211.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Typed event decoding, indexed topics, filter builders, log pagination, subscriptions, removed-log/reorg semantics, and checkpoints.

Verification:

- Local-node reorg/filter tests and malformed log fixtures.

Exit criteria:

- Event consumers can resume and handle reorganizations correctly.
- `v0.212.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.213.0 - Errors Multicall And Overrides

Status: planned; internal signed tag, publication at v0.215.0.

Goal: Common read/simulation workflows are typed and diagnostically complete.

Scope: bounded milestone. Depends on v0.212.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Custom error registry, revert decoding, Multicall workflows, batched calls, state/block overrides, and per-call evidence.

Verification:

- Contract fixture suite, partial-failure and ambiguity tests.

Exit criteria:

- Common read/simulation workflows are typed and diagnostically complete.
- `v0.213.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.214.0 - Token And NFT Standards

Status: planned; internal signed tag, publication at v0.215.0.

Goal: Common asset interactions are available without assuming compliant return behavior.

Scope: bounded milestone. Depends on v0.213.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Typed ERC-20, ERC-721, ERC-1155, metadata, approval, safe-transfer, and interface-detection helpers.

Verification:

- Reference contract integration and nonconforming-token fixtures.

Exit criteria:

- Common asset interactions are available without assuming compliant return behavior.
- `v0.214.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.215.0 - ENS Resolution And Normalization

Status: planned; public crates.io checkpoint after cumulative review.

Goal: implement name resolution without conflating names and addresses.

Scope: implementation pass. Depends on v0.214.0. The retained
workstream contract at v0.217.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement admitted normalization/namehash, forward/reverse records, wildcard/offchain lookup policy and anchored resolution.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official normalization/resolver fixtures, malicious offchain endpoints, recursion limits and reverse/forward disagreement.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Only correctly normalized, validated and anchored names yield trusted resolution evidence.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.215.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.216.0 - Permit And Contract Signature Wrappers

Status: planned; internal signed tag, publication at v0.220.0.

Goal: support permit and counterfactual signature domains independently of ENS.

Scope: implementation pass. Depends on v0.215.0. The retained
workstream contract at v0.217.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-2612 variants, ERC-1271 and EIP-6492 encoding/verification with explicit chain/contract context.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Cross-domain/replay, untrusted contract responses and counterfactual side-effect fixtures.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Signature wrappers cannot authorize a different contract, chain or request.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.216.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.217.0 - ENS Permit And Signature Standards Completion

Status: planned; internal signed tag, publication at v0.220.0.

Goal: Naming and permit workflows are first-class and domain-safe.

Scope: completion and integration pass. Depends on v0.216.0.
The implementation passes v0.215.0 through v0.216.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
ENS/permit/signature wrapper composition with wallet and provider trust evidence.

Deliverables:

- ENS resolution, namehash, reverse records, EIP-2612/permit variants, EIP-1271, EIP-6492, and supported signature wrappers.

Verification:

- Mainnet-fork/local fixtures and cross-standard domain tests.

Exit criteria:

- Naming and permit workflows are first-class and domain-safe.
- `v0.217.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.218.0 - Contract Tooling Hardening

Status: planned; internal signed tag, publication at v0.220.0.

Goal: Contract tooling is stable enough for production SDK use.

Scope: bounded milestone. Depends on v0.217.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- ABI/artifact/codegen fuzzing, binding ergonomics review, compatibility matrix, examples, and independent pentest.

Verification:

- Full contract suite, package/docs checks, clean retest.

Exit criteria:

- Contract tooling is stable enough for production SDK use.
- `v0.218.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 14: Storage Canonical Chain And Client Runtime

### v0.219.0 - Database Traits And Schema

Status: planned; internal signed tag, publication at v0.220.0.

Goal: Higher layers depend on a first-party storage contract, not one database API.

Scope: bounded milestone. Depends on v0.218.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Transactional key-value traits, column/schema identifiers, versioning, iterators, snapshots, durability capabilities, and explicit error contracts.

Verification:

- In-memory conformance backend, crash/error injection, no_std trait checks.

Exit criteria:

- Higher layers depend on a first-party storage contract, not one database API.
- `v0.219.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.220.0 - Production Storage Pilot

Status: planned; public crates.io checkpoint after cumulative review.



Goal: exercise the storage contract against durable production behavior before
mass synchronization and node-scale state are built on it.

Scope: bounded milestone. Depends on v0.219.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- One optional reviewed durable backend pilot implementing the `v0.219.0`
  contract without entering the default `no_std` graph;
- atomic batches, snapshots, restart recovery, schema versioning, bounded
  caches, checksums, and state-healing probes;
- documented backend assumptions, unsafe/dependency inventory, corruption
  model, and replacement boundary.

Verification:

- Process-kill injection at every persistent write boundary;
- torn-write, corruption, migration, snapshot restore, and concurrent-reader
  tests;
- deterministic comparison with the in-memory conformance backend.

Exit criteria:

- Storage abstractions have survived real durability and recovery behavior
  before full sync can amplify a flawed contract.
- `v0.220.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.221.0 - Chain Content Stores

Status: planned; internal signed tag, publication at v0.225.0.

Goal: Canonical chain content can be retained and queried consistently.

Scope: bounded milestone. Depends on v0.220.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Headers, bodies, transactions, receipts, total difficulty, execution requests, blobs, and hash/number indexes.

Verification:

- Round trips, corruption detection, incomplete-write tests.

Exit criteria:

- Canonical chain content can be retained and queried consistently.
- `v0.221.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.222.0 - State Trie Flat State And Indexes

Status: planned; internal signed tag, publication at v0.225.0.

Goal: Persisted state representations have explicit consistency invariants.

Scope: bounded milestone. Depends on v0.221.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Account/storage trie nodes, code, flat state, changesets, history indexes, and root/version association.

Verification:

- Root reconstruction, index consistency, corruption fixtures.

Exit criteria:

- Persisted state representations have explicit consistency invariants.
- `v0.222.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.223.0 - Atomic Batches And Crash Consistency

Status: planned; internal signed tag, publication at v0.225.0.

Goal: A committed block is either fully durable or detectably absent.

Scope: bounded milestone. Depends on v0.222.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Multi-column atomic commits, write-ahead/recovery contract, idempotent replay, and partial-transition detection.

Verification:

- Process-kill and torn-write simulations across admitted backends.

Exit criteria:

- A committed block is either fully durable or detectably absent.
- `v0.223.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.224.0 - Storage Migrations And Snapshots

Status: planned; internal signed tag, publication at v0.225.0.

Goal: separate durable format transitions from optional cache behavior.

Scope: implementation pass. Depends on v0.223.0. The retained
workstream contract at v0.225.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement schema migration, snapshot import/export, rollback boundaries and canonical persisted hints with atomic authority restoration.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Process-kill, mixed schema, corrupt snapshot and rollback tests on the admitted backend.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Only completed checked migrations/snapshots restore authoritative state.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.224.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.225.0 - Migrations Snapshots And Cache Policy Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Storage upgrades and restores are reproducible and fail closed.

Scope: completion and integration pass. Depends on v0.224.0.
The implementation passes v0.224.0 through v0.224.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
cache/evidence persistence rules composed with admitted migrations and snapshots.

Deliverables:

- Forward migrations, rollback limits, snapshot import/export, cache sizing/
  eviction, and schema compatibility reports;
- persisted validation contexts are non-authoritative digest/hint records and
  must be rederived through the `v0.122.0` rules engine before use;
- stored rules/limits/context digests use canonical versioned encodings and
  domain-separated cryptographic hashes; process-local hash/layout/pointer
  identities are rejected and expired snapshot/arena handles are never stored;
- persistent invalidity and peer evidence obeys the `v0.93.0` entry-size,
  witness, observation, serialization, and retention budgets; full malformed
  objects require a separately bounded object store;
- evidence persistence follows `v0.100.0`: only atomically committed filled
  records are authoritative, while reserved/derived/abandoned lifecycle state
  is non-authoritative and discarded during recovery;
- cache, migration, snapshot, and persistence adapters follow `v0.102.0` by
  borrowing immutable evidence views; no sink owns or clones slot authority,
  and sink failure cannot prevent the final return/commit transition.

Verification:

- Upgrade/downgrade fixtures, snapshot checksums, cache-pressure and evidence-
  retention tests, corrupted/forged stored-context rejection, and oversized-
  evidence migration tests;
- cross-restart/platform digest fixtures, encoding/version invalidation, and
  corrupted-handle/expired-lease non-revival tests;
- crash-at-every-transition recovery tests proving abandoned reservations never
  become invalidity evidence;
- sink borrow-order, failure, cancellation, and final-ownership-transition tests.

Exit criteria:

- Storage upgrades and restores are reproducible and fail closed.
- `v0.225.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.226.0 - Persistent Fault And Recovery Gate

Status: planned; internal signed tag, publication at v0.230.0.



Goal: make crash consistency, corruption recovery, reorg replay, migration,
and rollback evidence mandatory before network synchronization.

Scope: bounded milestone. Depends on v0.225.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Deterministic process-kill and I/O-fault injection across every persistent
  write phase admitted so far;
- corruption localization, quarantine, repair, replay, snapshot restore, and
  upgrade rollback workflows;
- durable operation IDs and idempotency records for restart-safe orchestration;
- immutable reports covering data-loss boundaries and operator action.

Verification:

- Fault matrix over atomic batches, indexes, snapshots, migrations, state
  healing, canonical changes, and reorg replay;
- repeated crash/restart campaigns with root and index comparison;
- zero unexplained divergence after recovery.

Exit criteria:

- A process kill or recoverable corruption at any reviewed write boundary has
  a deterministic detected outcome and tested recovery path.
- `v0.226.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.227.0 - Pruning Archive And History Expiry

Status: planned; internal signed tag, publication at v0.230.0.

Goal: Operators know exactly which historical guarantees each mode provides.

Scope: bounded milestone. Depends on v0.226.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Configurable archive/pruned modes, retention proofs, ancient/history separation, expiry scheduling, and explicit unavailable-data errors.

Verification:

- Long-chain simulation, prune/reorg interactions, historical query tests.

Exit criteria:

- Operators know exactly which historical guarantees each mode provides.
- `v0.227.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.228.0 - Canonical Import And Reorg

Status: planned; internal signed tag, publication at v0.230.0.

Goal: Canonical chain changes preserve state and index consistency.

Scope: bounded milestone. Depends on v0.227.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Block import pipeline, validation stages, total-difficulty/fork-choice
  inputs, canonical indexes, unwind, and re-execution;
- every stage preserves `v0.93.0` outcome/evidence, and only proven
  `ObjectInvalidityEvidence` enters persistent bad-block state;
- bounded negative-cache identity, invalidation, retention, and anti-flood
  rules are transactional with canonical import/reorg changes;
- cache insertion reserves the complete evidence entry and serialization
  budget before mutation and stores compact digest/reason/location/witness
  records rather than implicit full malformed objects;
- failure to serialize or insert optional persistent/cache attachments never
  changes the immediate minimal `ProtocolInvalid` result returned by import;
- import stages consume one `v0.100.0` hierarchical reservation tree, and
  durable writes expose only committed filled leaves rather than reservation
  bookkeeping;
- bad-block/cache/persistence/logging consumers borrow the `v0.102.0`
  immutable evidence record and cannot consume slot authority; only the final
  import outcome performs its one ownership transition.

Verification:

- Competing-chain and deep-reorg simulations, crash recovery, local-fault
  injection, bad-block-cache poisoning/flood tests, and evidence reservation/
  partial-write/oversized-witness tests;
- post-invalid serialization, persistence, and cache-insertion failures with
  unchanged immediate result and clean transactional rollback;
- process-kill tests across slot derivation/fill/persist/commit/release proving
  recovery cannot promote abandoned reservations;
- optional-sink permutation and fault tests proving import always returns the
  same invalid result and finalizes slot ownership exactly once.

Exit criteria:

- Canonical chain changes preserve state and index consistency.
- `v0.228.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.229.0 - Heads Fork Choice And Orphans

Status: planned; internal signed tag, publication at v0.230.0.

Goal: Head state is explicit and cannot advance through invalid ancestry.

Scope: bounded milestone. Depends on v0.228.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Unsafe/safe/finalized heads, orphan queues, ancestry checks, invalid
  ancestors backed by `v0.93.0` `ObjectInvalidityEvidence`, checkpoint
  constraints, and chain events; unknown/local-failure ancestry remains
  unresolved rather than invalid.

Verification:

- Engine/fork-choice sequences and orphan/finality property tests.

Exit criteria:

- Head state is explicit and cannot advance through invalid ancestry.
- `v0.229.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.230.0 - Payload Orchestration And Invalidation

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Payload work terminates consistently under reorgs and invalid blocks.

Scope: bounded milestone. Depends on v0.229.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Payload build/import state machines, optimistic execution, invalidation propagation, cancellation, and cache cleanup.

Verification:

- Engine API sequence fixtures, concurrent invalidation tests.

Exit criteria:

- Payload work terminates consistently under reorgs and invalid blocks.
- `v0.230.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.231.0 - Operational Client Runtime

Status: planned; internal signed tag, publication at v0.235.0.

Goal: Node-adjacent services have a coherent lifecycle and observable failure model.

Scope: bounded milestone. Depends on v0.230.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Supervised bounded tasks for import, providers, peers, sync, pruning, metrics,
  shutdown, and restart without imposing one async runtime on core crates;
- startup validates static implementation capacity against each advertised
  supported network profile, refusing unsafe validating configurations or
  entering an explicit non-validating/light mode;
- publish the production static/dynamic resource envelope and withdraw
  validation readiness on physical exhaustion until capacity is restored;
- runtime resource faults preserve `v0.93.0` local outcomes and cannot poison
  validation state;
- evidence arenas use `v0.106.0` capability-backed simultaneous-work sizing;
  runtime scheduling serializes/backpressures or withdraws readiness on
  exhaustion and admits per-worker pools only when benchmark evidence and
  explicit lifecycle configuration exist.

Verification:

- Failure injection, graceful shutdown/restart, validating/light mode startup
  matrices, advertised static-maximum admission, dynamic parent/candidate work,
  readiness withdrawal/recovery, and resource-cap tests;
- arena capability exhaustion, backpressure, cancellation/transfer, optional-
  pool lifecycle, and retained-memory tests.

Exit criteria:

- Node-adjacent services have a coherent lifecycle and observable failure model.
- `v0.231.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.232.0 - Storage And Client Performance Gate

Status: planned; internal signed tag, publication at v0.235.0.

Goal: Storage/client foundations meet documented correctness and operational budgets.

Scope: bounded milestone. Depends on v0.231.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Benchmark import, state access, roots, reorgs, snapshots, pruning, memory,
  disk amplification, and startup recovery;
- carry the `v0.104.0` evidence-overhead baseline through valid canonical
  import, reorg validation, bad-object handling, persistence/cache sinks, stack,
  context/arena size, and retained-memory measurements.

Verification:

- Reproducible hardware profile and regression thresholds;
- evidence-enabled/evidence-disabled import comparisons and sink-disabled valid-
  path instrumentation.

Exit criteria:

- Storage/client foundations meet documented correctness and operational budgets.
- `v0.232.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 15: Consensus Engine And Light Client

### v0.233.0 - SSZ Canonical Wire Codec

Status: planned; internal signed tag, publication at v0.235.0.

Goal: implement bounded first-party SSZ encoding and decoding before rooting.

Scope: implementation pass. Depends on v0.232.0. The retained
workstream contract at v0.235.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement basic/composite encodings, offsets, lists/vectors and bitfields under the shared decode ledger.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Standalone SSZ vectors, malformed offsets/padding and rejection-before-allocation fuzzing.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- SSZ canonicality is established independently of Merkle proofs.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.233.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.234.0 - SSZ Merkle Roots And Branches

Status: planned; internal signed tag, publication at v0.235.0.

Goal: derive authenticated roots from canonical SSZ objects.

Scope: implementation pass. Depends on v0.233.0. The retained
workstream contract at v0.235.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement generalized indices, tree hashing and baseline branch construction/verification with explicit work budgets.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent roots/proofs, wrong-index/container substitutions and maximum-depth tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Immutable SSZ roots and branches match official vectors.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.234.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.235.0 - SSZ Foundational Codec And Merkleization Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: establish the immutable SSZ wire and Merkle foundation required by
light-client and protocol-type work without claiming the later mutable,
cached, full-client surface.

Scope: completion and integration pass. Depends on v0.234.0.
The implementation passes v0.233.0 through v0.234.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
codec/root/branch integration and immutable consensus object conformance.

Deliverables:

- First-party basic and composite SSZ type rules;
- bounded canonical encode/decode consuming the `v0.88.0` parent ledger;
- charged offset traversal, container/list/vector elements, bitlists,
  bitvectors, allocation capacity, hashes, Merkleization work, and output;
- offset validation and reject-before-allocation/hash behavior;
- generalized indices;
- baseline Merkleization, branches, and hash-tree roots;
- explicit exclusions for incremental mutation, cached trees, and
  multiproofs assigned to `v0.308.0`.

Verification:

- Official standalone SSZ-specs vectors plus fork-specific consensus vectors;
- malformed-offset fuzzing;
- list/bitlist/offset/Merkleization complexity oracles and nested-ledger
  conservation tests;
- baseline root differentials;
- cross-check that later full-client APIs cannot be inferred from this
  foundational release.

Exit criteria:

- Immutable consensus objects can be encoded, decoded, rooted, and proven
  without external SSZ core logic, while mutable production operations remain
  explicitly assigned to `v0.308.0`.
- `v0.235.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.236.0 - Beacon Types And Fork Domains

Status: planned; internal signed tag, publication at v0.240.0.

Goal: Consensus data has complete owned/borrowed/fork-aware models.

Scope: bounded milestone. Depends on v0.235.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Fork-versioned beacon blocks, states, execution payloads, withdrawals, blobs/data columns, requests, domains, and signing roots.

Verification:

- Official consensus fixtures across all claimed forks.

Exit criteria:

- Consensus data has complete owned/borrowed/fork-aware models.
- `v0.236.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.237.0 - Engine Method And Status Contracts

Status: planned; internal signed tag, publication at v0.240.0.

Goal: freeze the small Engine semantic interface before wire adapters.

Scope: implementation pass. Depends on v0.236.0. The retained
workstream contract at v0.238.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement fork-bound payload IDs, method versions and status/evidence rules; model retries and sequencing independently of sockets.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Pinned method fixtures, status substitution and executable duplicate/reorder/invalidity models.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No transport can mint authoritative INVALID or reinterpret a payload ID.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.237.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.238.0 - Engine API Types And Validation Completion

Status: planned; internal signed tag, publication at v0.240.0.

Goal: Engine messages are fully typed and version/fork checked.

Scope: completion and integration pass. Depends on v0.237.0.
The implementation passes v0.237.0 through v0.237.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
versioned Engine wire schemas and semantic-model integration including admitted SSZ transport contracts.

Deliverables:

- All pinned Engine API versions, payload attributes, execution status, capabilities, transition configuration, and strict validation.
- Engine `INVALID` and `latestValidHash` are constructible only from
  `v0.93.0` `ObjectInvalidityEvidence`; syncing, missing dependencies,
  resource exhaustion, cancellation, backend/storage errors, and internal
  faults map to non-invalid statuses/errors.
- authoritative payload validation reserves minimal evidence first; once
  `INVALID` is established, failure of diagnostics, tracing, persistence, or
  negative-cache insertion cannot downgrade or erase that response.
- Engine consensus validation uses `v0.102.0` `FirstInvalid`; diagnostic,
  tracing, persistence, and cache consumers only borrow immutable evidence and
  cannot consume slot authority or alter `INVALID`/`latestValidHash`.
- Small typed semantic surface for capability negotiation, `newPayload`,
  `forkchoiceUpdated`, and `getPayload`, including typed payload IDs,
  deadlines, cancellation, evidence-rich status, and exact
  `latestValidHash` semantics.
- A design-time executable model for Engine sequencing, idempotency,
  invalidation, retries, cancellation, and `latestValidHash`, maintained before
  transport and coordinator implementations consume it.

Verification:

- execution-apis fixtures and client interoperability snapshots;
- model-check traces, seeded invariant violations, and trace-to-Rust
  regressions;
- post-invalid diagnostic/cache/persistence fault injection preserving exact
  `INVALID` and `latestValidHash` semantics;
- sink borrow-order and collection-mode tests proving exactly one final slot
  transition and invariant Engine status.

Exit criteria:

- Engine messages are fully typed and version/fork checked.
- `v0.238.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.239.0 - Engine Transport And Protocol Boundary

Status: planned; internal signed tag, publication at v0.240.0.

Goal: define and test the reusable Engine API protocol and authenticated
transport boundary without claiming beacon-node coordination policy.

Scope: bounded milestone. Depends on v0.238.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Runtime-neutral Engine client/server traits;
- authenticated transport adapter;
- request/response sequencing primitives;
- idempotency, cancellation, timeout, and error mapping;
- conformance to the `v0.238.0` sequencing model across in-process and
  authenticated transport paths;
- explicit statement that beacon fork-choice and payload orchestration belong
  to the Beacon Engine Coordinator at `v0.355.0`.

Verification:

- Protocol sequence tests;
- JWT and redaction review;
- transport conformance tests independent of beacon-node policy;
- retry/reorder/duplicate/cancellation traces generated from the design model.

Exit criteria:

- Engine messages can travel through an authenticated, runtime-neutral
  boundary in either embedding direction without assigning beacon-node
  coordination ownership to this layer.
- `v0.239.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.240.0 - Early Engine Vertical Devnet

Status: planned; public crates.io checkpoint after cumulative review.



Goal: establish an expanding end-to-end EL/Engine/CL path before late product
integration hides incompatible assumptions.

Scope: bounded milestone. Depends on v0.239.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- A minimal deterministic devnet using the same typed `ExecutionEngine`
  contract through both in-process and authenticated JSON-RPC adapters;
- one native execution path plus independent execution/consensus client
  combinations where the implemented scope permits;
- durable request/idempotency records, timeout/cancellation, invalid payload,
  restart, and split-brain scenarios;
- a versioned scenario register expanded by every later execution, Engine,
  consensus, storage, networking, and fork release.

Verification:

- Repeatable Podman bring-up/tear-down and immutable scenario report;
- adapter-equivalence tests for status, payload IDs, invalidation,
  `latestValidHash`, deadlines, and cancellation;
- mixed-client smoke tests with zero unexplained semantic differences.

Exit criteria:

- The project has a continuously growing vertical interoperability path rather
  than waiting until integrated-node milestones for first composition.
- `v0.240.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.241.0 - Beacon API Provider Client

Status: planned; internal signed tag, publication at v0.245.0.

Goal: provide a typed outbound Beacon API client/provider boundary without
claiming the later beacon-node server implementation.

Scope: bounded milestone. Depends on v0.240.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Typed outbound Beacon REST methods;
- event-stream client handling;
- pagination and version negotiation;
- finality, blob, and data-column responses;
- bounded transport policy;
- duplicate-key-rejecting Beacon JSON parsing through the `v0.88.0` parent
  ledger with structural depth/node, string, array, allocation, and output
  accounting;
- explicit server-side ownership assigned to `v0.362.0`.

Verification:

- Beacon API client fixtures;
- local independent consensus-client integration;
- duplicate-key, structural-depth/node, oversized-string/array, and allocation-
  before-reservation tests;
- compile and documentation checks separating provider and server roles.

Exit criteria:

- Consensus data can be acquired through a production typed provider
  boundary, while serving the Beacon API remains a distinct beacon-node
  responsibility.
- `v0.241.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.242.0 - Light Client Bootstrap And Weak Subjectivity

Status: planned; internal signed tag, publication at v0.245.0.

Goal: A light client starts only from explicit, valid trust roots.

Scope: bounded milestone. Depends on v0.241.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Trusted checkpoint/bootstrap validation, fork/genesis binding,
  weak-subjectivity periods, stale-checkpoint rejection, and clock policy using
  the monotonic/UTC/evidence domains from `v0.89.0`.

Verification:

- Official bootstrap vectors and stale/adversarial checkpoint tests.

Exit criteria:

- A light client starts only from explicit, valid trust roots.
- `v0.242.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.243.0 - BLS Message Hashing And Ciphersuite

Status: planned; internal signed tag, publication at v0.245.0.

Goal: supply full message hashing rather than reuse field-to-curve maps incorrectly.

Scope: implementation pass. Depends on v0.242.0. The retained
workstream contract at v0.245.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement the pinned consensus BLS ciphersuite, expand-message/hash-to-field, domain separation and point encoding using admitted group arithmetic.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Consensus BLS and RFC 9380 vectors, domain mismatch, infinity/non-subgroup and encoding cases.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Message-to-curve results match independent implementations before signature verification.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.243.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.244.0 - BLS Aggregate Verification Kernel

Status: planned; internal signed tag, publication at v0.245.0.

Goal: verify public BLS signatures before light-client policy.

Scope: implementation pass. Depends on v0.243.0. The retained
workstream contract at v0.245.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement strict key/signature validation, aggregate and fast-aggregate verification with exact empty/duplicate/domain rules.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official consensus vectors, rogue/invalid point cases and independent aggregate verification.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Light-client policy receives verified signatures from first-party cryptography.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.244.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.245.0 - BLS Sync Committee Verification Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Sync committee attestations are cryptographically verified first party or through an audited explicit backend.

Scope: completion and integration pass. Depends on v0.244.0.
The implementation passes v0.243.0 through v0.244.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
sync-committee membership, participant bits and fork/domain policy over admitted BLS verification.

Deliverables:

- Aggregate BLS signatures, participant bits, signing domains, committee membership, and cryptographic backend admission.

Verification:

- Official BLS/light-client vectors, malformed/subgroup fuzzing.

Exit criteria:

- Sync committee attestations are cryptographically verified first party or through an audited explicit backend.
- `v0.245.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.246.0 - Committee Rotation And Persistence

Status: planned; internal signed tag, publication at v0.250.0.

Goal: Trust state survives rotation and restart without accepting stale committees.

Scope: bounded milestone. Depends on v0.245.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Period transitions, next-committee proofs, durable store, rollback/recovery, fork upgrades, and checkpoint export.

Verification:

- Multi-period vectors, crash/restart tests, conflicting-update cases.

Exit criteria:

- Trust state survives rotation and restart without accepting stale committees.
- `v0.246.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.247.0 - Finality Optimistic Scoring And Misbehavior

Status: planned; internal signed tag, publication at v0.250.0.

Goal: Update selection and finality are deterministic under conflicting inputs.

Scope: bounded milestone. Depends on v0.246.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Update ranking, optimistic/finalized headers, participation thresholds, duplicate/conflict handling, and misbehavior evidence.

Verification:

- Official update-processing vectors and Byzantine peer simulations.

Exit criteria:

- Update selection and finality are deterministic under conflicting inputs.
- `v0.247.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.248.0 - Execution Proof Binding

Status: planned; internal signed tag, publication at v0.250.0.

Goal: Verified RPC/state evidence can anchor to a light-client trust root.

Scope: bounded milestone. Depends on v0.247.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bind finalized beacon execution payload roots to execution headers/state/receipts through SSZ and MPT proofs.

Verification:

- End-to-end consensus-to-execution proof fixtures.

Exit criteria:

- Verified RPC/state evidence can anchor to a light-client trust root.
- `v0.248.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.249.0 - Checkpoint Recovery And Multi-Source Acquisition

Status: planned; internal signed tag, publication at v0.250.0.

Goal: Light-client operation can recover without silently replacing its trust root.

Scope: bounded milestone. Depends on v0.248.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Multiple bootstrap/update sources, quorum/evidence, stale-source isolation, checkpoint rotation, and recovery procedures.

Verification:

- Offline/recovery and malicious-source simulations.

Exit criteria:

- Light-client operation can recover without silently replacing its trust root.
- `v0.249.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.250.0 - Complete Light-Client Conformance

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Complete light-client claims are fixture-backed and operationally documented.

Scope: bounded milestone. Depends on v0.249.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Run all official light-client suites for historical/current forks and publish support/skip evidence.

Verification:

- Generated conformance report with zero unexplained skips.

Exit criteria:

- Complete light-client claims are fixture-backed and operationally documented.
- `v0.250.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.251.0 - PeerDAS Threat Model And Admission Plan

Status: planned; internal signed tag, publication at v0.255.0.

Goal: define PeerDAS trust, cryptographic, custody, sampling, networking, and
resource requirements before implementation begins.

Scope: bounded milestone. Depends on v0.250.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- PeerDAS threat model;
- data-column and sampling boundaries;
- custody policy;
- cryptographic and trusted-setup requirements;
- CPU, memory, bandwidth, and retention ceilings;
- versioned implementation assignments beginning at `v0.315.0`;
- fail-closed rules that prevent this planning release from implying an
  executable PeerDAS implementation.

Verification:

- Review against pinned PeerDAS/current-fork specifications;
- abuse-case and dependency review;
- traceability check proving every admitted requirement has a later release.

Exit criteria:

- PeerDAS implementation cannot begin with ambiguous trust, cryptographic,
  custody, networking, or resource boundaries, and no consumer can claim
  support before the `v0.315.0` core exists.
- `v0.251.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 16: Networking Txpool And Synchronization

### v0.252.0 - Networking Threat And Dependency Gate

Status: planned; internal signed tag, publication at v0.255.0.

Goal: No live peer code lands before trust and resource boundaries are approved.

Scope: bounded milestone. Depends on v0.251.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Protocol threat model, crypto/transport dependency review, identity/key
  policy, resource ceilings, and wire-spec locks;
- assign every protocol/version pair an explicit `WireLimits` profile from
  `v0.98.0` and every sanction path an evidence type from `v0.93.0`;
- first-party ownership rule for discovery, RLPx, `eth`, `snap`, request
  scheduling, peer scoring, and validation state machines;
- reviewed optional socket, runtime, and cryptographic adapters only when they
  cannot decide Ethereum protocol validity or policy;
- enforce separate execution DevP2P and consensus libp2p protocol planes:
  shared clocks, sockets, tasks, metrics, and resource-governor traits are
  allowed, but identity, scoring, banning, compatibility, and peer state are
  not collapsed into one generic Ethereum peer abstraction;
- bind every connection, peer, protocol, and request to child capabilities
  from the `v0.72.0` resource governor;
- admit Snappy and SSZ-Snappy only through `v0.88.0` compressed-byte,
  decompressed-byte, ratio, allocation, structural-work, and output budgets;
- design-time models for retry/backpressure and peer scheduling must exist
  before live request scheduling.

Verification:

- cargo-deny/audit, protocol corpus plan, architecture review, decompression-
  bomb fixtures, and seeded scheduling-model violations.

Exit criteria:

- No live peer code lands before trust and resource boundaries are approved.
- `v0.252.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.253.0 - Node Records And Discovery V4

Status: planned; internal signed tag, publication at v0.255.0.

Goal: implement execution peer discovery without live encrypted sessions.

Scope: implementation pass. Depends on v0.252.0. The retained
workstream contract at v0.257.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement ENR identity/sequence validation and discv4 packet, endpoint-proof and routing behavior with bounded tables.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official wire vectors, stale records, amplification and endpoint spoofing tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Discovered endpoints remain untrusted until the protocol proves reachability.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.253.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.254.0 - Discovery V5 Session State Machine

Status: planned; internal signed tag, publication at v0.255.0.

Goal: implement authenticated discovery sessions with explicit replay handling.

Scope: implementation pass. Depends on v0.253.0. The retained
workstream contract at v0.257.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement WHOAREYOU/handshake, AES-GCM messaging, challenge reuse and bounded routing/admission with immutable peer identity.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official handshake vectors, duplicate challenge, nonce/replay and pending-session floods.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Discovery sessions interoperate without amplification or unbounded pending state.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.254.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.255.0 - DNS Discovery Trees

Status: planned; public crates.io checkpoint after cumulative review.

Goal: verify signed DNS node trees before dialing.

Scope: implementation pass. Depends on v0.254.0. The retained
workstream contract at v0.257.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-1459 signed roots, sequence/TTL policy, links, traversal bounds and address policy.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Signature/sequence rollback, cycles, DNS rebinding and excessive-tree fixtures.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Unverified or cyclic DNS trees cannot produce trusted dial candidates.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.255.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.256.0 - RLPx Authenticated Framing

Status: planned; internal signed tag, publication at v0.260.0.

Goal: implement encrypted RLPx sessions independently of discovery.

Scope: implementation pass. Depends on v0.255.0. The retained
workstream contract at v0.257.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement auth/ack, rolling encryption/MAC, frame sizing and hello/capability negotiation using admitted transcript crypto.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Two-client session fixtures, truncation/MAC failures, replay and frame-work bounds.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No unauthenticated frame reaches protocol message dispatch.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.256.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.257.0 - Discovery And RLPx Completion

Status: planned; internal signed tag, publication at v0.260.0.

Goal: Peers can be discovered and authenticated through bounded first-party protocol logic.

Scope: completion and integration pass. Depends on v0.256.0.
The implementation passes v0.253.0 through v0.256.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
discovery and RLPx runtime-neutral peer configuration/integration.

Deliverables:

- First-party Discovery v4/v5 as admitted, ENR, EIP-1459 DNS discovery, node
  records, handshakes, framing, capability negotiation, encryption/MAC state
  machines, and replay protections;
- use the admitted `v0.75.0..=v0.86.0` secp256k1, ECDH, AES-CTR,
  HMAC-SHA256, ECIES/KDF, and transcript contracts rather than unnamed crypto;
- consume only the non-signing `TransportIdentity` capability from `v0.95.0`;
  node/transport keys cannot satisfy `ExecutionSigner` or sign transactions,
  messages, EIP-712 data, or EIP-7702 authorizations;
- bootnode, static-peer, trusted-peer, node-key, listen-address, NAT, and
  advertised-address policy behind runtime-neutral I/O boundaries.

Verification:

- Official/reference vectors, packet/frame fuzzing, interoperability tests,
  and compile-fail transport-key-to-execution-signer substitution tests.

Exit criteria:

- Peers can be discovered and authenticated through bounded first-party protocol logic.
- `v0.257.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.258.0 - Baseline Eth Wire Protocol

Status: planned; internal signed tag, publication at v0.260.0.

Goal: implement the admitted baseline chain-data messages before newer capabilities.

Scope: implementation pass. Depends on v0.257.0. The retained
workstream contract at v0.260.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement status, header/body/transaction/receipt messages and request correlation using negotiated wire budgets.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Cross-client message fixtures, invalid correlations, historical announcement policy and resource ceilings.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Baseline peers exchange typed bounded data without granting consensus authority.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.258.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.259.0 - Modern Eth Capability Extensions

Status: planned; internal signed tag, publication at v0.260.0.

Goal: implement negotiated receipt BAL and sparse-blob messages as wire protocols.

Scope: implementation pass. Depends on v0.258.0. The retained
workstream contract at v0.260.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement separately versioned eth/70, eth/71 and eth/72 schemas, correlation and custody metadata using pinned accepted specs.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Mixed-version peers, partial receipts, BAL requests and null/empty custody metadata.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A negotiated capability never silently activates a consensus fork.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.259.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.260.0 - Eth Protocol Messages Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Execution chain data can be exchanged through typed wire messages.

Scope: completion and integration pass. Depends on v0.259.0.
The implementation passes v0.258.0 through v0.259.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
baseline/modern capability negotiation and typed dispatch integration.

Deliverables:

- Status negotiation and all admitted `eth` protocol request/response/
  announcement messages with fork capability checks;
- negotiated `WireLimits<Eth, Version>` profiles whose violation creates peer
  protocol evidence without declaring transported chain data invalid.

Verification:

- Cross-client devp2p fixtures, per-version boundary tests, malformed message
  fuzzing, and wire-violation/object-invalidity separation tests.

Exit criteria:

- Execution chain data can be exchanged through typed wire messages.
- `v0.260.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.261.0 - Snap Range Consumption

Status: planned; internal signed tag, publication at v0.265.0.

Goal: validate downloaded ranges before storage promotion.

Scope: implementation pass. Depends on v0.260.0. The retained
workstream contract at v0.263.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement account/storage ranges, code/node retrieval and snapshot-bound proof validation under wire and work budgets.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent range proofs, missing/overlap/continuation cases and proof bombs.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Unverified snapshot data cannot enter canonical storage.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.261.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.262.0 - Snap Range Serving And BAL Healing

Status: planned; internal signed tag, publication at v0.265.0.

Goal: serve consistent ranges without starving validation.

Scope: implementation pass. Depends on v0.261.0. The retained
workstream contract at v0.263.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement snapshot leases, range/proof construction, fair bounded serving and admitted snap/2 BAL healing.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Consumer/server interoperability, expired snapshots, malicious queries and mixed-version healing.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Serving cannot mix snapshots or exhaust unreserved validation resources.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.262.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.263.0 - Snap Protocol Completion

Status: planned; internal signed tag, publication at v0.265.0.

Goal: Snapshot data is validated before storage or state promotion.

Scope: completion and integration pass. Depends on v0.262.0.
The implementation passes v0.261.0 through v0.262.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
cross-client consume/serve/healing integration and fairness acceptance.

Deliverables:

- Account ranges, storage ranges, bytecodes, trie nodes, proofs, continuation
  rules, and response limits for both consumption and serving;
- negotiated `WireLimits<Snap, Version>` separated from snapshot/proof
  validity and local serving policy;
- MPT/range-proof verification consumes an explicit `WorkBudget`; wire-size or
  local serving rejection cannot create invalid-proof evidence and a locally
  exhausted verification can be retried under a larger admitted budget;
- snapshot-pinned account/storage range generation and canonical range-proof
  construction without mixed-snapshot nodes;
- serving-side database-read, proof-generation, compression, allocation,
  bandwidth, and output budgets plus fair scheduling against local sync;
- cancellation when a serving snapshot becomes unavailable;
- all compressed and decompressed work consumes the `v0.88.0` parent ledger.

Verification:

- Cross-client consume/serve fixtures, proof verification, response-bomb and
  compression-bomb tests, mixed-snapshot rejection, snapshot-loss
  cancellation, serving-fairness simulations, oversized-wire/valid-proof
  separation, and larger-budget verification retry.

Exit criteria:

- Snapshot data is validated before storage or state promotion.
- `v0.263.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.264.0 - Peer Service

Status: planned; internal signed tag, publication at v0.265.0.

Goal: Peer selection and isolation are explicit and bounded.

Scope: bounded milestone. Depends on v0.263.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Execution-plane peer lifecycle, capability scoring, quotas, diversity, bans,
  disconnect reasons, persistence, and metrics without sharing consensus-plane
  identity or scoring state;
- hierarchical resource capabilities and peer accountability for malformed,
  wasteful, timed-out, and inconsistent work;
- bounded evidence counters/windows and compact witnesses consume
  `v0.93.0` budgets; peer histories cannot grow per-event vectors;
- rate windows and in-process sanctions use monotonic time, persisted records
  carry `v0.89.0` boot/session identity, and UTC rollback cannot extend bans;
- penalties and bans require `v0.93.0` object-invalidity evidence composed
  with an authenticated peer-delivery observation, peer-protocol evidence, or
  peer-policy evidence; local validation failures never accuse the supplying
  peer and peer evidence never enters object-negative caches.

Verification:

- Churn/eclipse/flood simulations, restart tests, and evidence substitution/
  time-window/policy-version tests plus maximum-evidence, session-change,
  clock-rollback, and expired-ban replay cases.

Exit criteria:

- Peer selection and isolation are explicit and bounded.
- `v0.264.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.265.0 - Request Scheduler And Backpressure

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Network work cannot create unbounded queues or retry amplification.

Scope: bounded milestone. Depends on v0.264.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Correlated requests, deadlines, retries, hierarchical per-request/per-peer/
  per-protocol/node budgets, cancellation, fair scheduling, queue slots,
  bandwidth limits, and evidence-bound invalid-response penalties;
- cancellation propagates when fork choice invalidates work and refunds only
  reservations whose resources were released, never consumed work;
- executable retry/backpressure and fairness models refined from `v0.252.0`.

Verification:

- Loss/reorder/timeout/load simulations, model traces, retry-amplification
  checks, conservation assertions, and race tests.

Exit criteria:

- Network work cannot create unbounded queues or retry amplification.
- `v0.265.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.266.0 - Transaction Pool Admission And Replacement

Status: planned; internal signed tag, publication at v0.270.0.

Goal: separate consensus validity from bounded local pool policy.

Scope: implementation pass. Depends on v0.265.0. The retained
workstream contract at v0.268.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement nonce lanes, fee classes, replacement/eviction and explicit deferred/local outcomes using validated transactions.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Executable replacement model, nonce gaps, policy versus invalidity and adversarial sender floods.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Pool rejection does not manufacture consensus invalidity.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.266.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.267.0 - Pool Reorg Blob And Delegation Lifecycle

Status: planned; internal signed tag, publication at v0.270.0.

Goal: revalidate every retained transaction when its assumptions change.

Scope: implementation pass. Depends on v0.266.0. The retained
workstream contract at v0.268.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement reorg reinjection, blob custody/expiry, authorization conflicts and historical-only persisted admission hints.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Head/fork/state changes, restart, late sidecars and protected-local transaction regressions.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No retained or protected entry bypasses context revalidation.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.267.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.268.0 - Transaction Pool Completion

Status: planned; internal signed tag, publication at v0.270.0.

Goal: Pending transaction policy is deterministic and resource bounded.

Scope: completion and integration pass. Depends on v0.267.0.
The implementation passes v0.266.0 through v0.267.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
pool lifecycle model conformance and evidence/propagation integration.

Deliverables:

- Strict separation of consensus transaction validity from local pool admission
  and propagation policy;
- sender/nonce lanes, nonce gaps, base-fee and blob-fee repricing, bounded
  replacement, eviction, and competing-hash tracking;
- blob-sidecar lifecycle and EIP-7702 authority-conflict handling;
- reorg journaling/reinjection, bounded local-transaction protection,
  persistence boundaries, and propagation-privacy policy;
- a design-time replacement/eviction/reorg model with counterexamples promoted
  into deterministic implementation regressions;
- complete revalidation on canonical-head change, fork activation, base-fee or
  blob-fee class change, restart/reload, account nonce/balance/code/delegation
  change, blob-sidecar arrival/expiry, and EIP-7702 authority-state change;
- persisted admission evidence is historical only; local/protected status can
  affect eviction/propagation policy but never exempt consensus revalidation;
- only `v0.93.0` object-invalidity evidence permits permanent transaction
  rejection or bad-transaction caching; peer sanctions additionally require
  an authenticated delivery observation or peer protocol/policy evidence, and
  local failures remain retryable.

Verification:

- Client differential cases, adversarial pool loads, restart/persistence,
  fee/fork/head/account/blob/authorization revalidation, reorg tests, and
  model-checked replacement/eviction invariants.

Exit criteria:

- Pending transaction policy is deterministic and resource bounded.
- `v0.268.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.269.0 - Sync Orchestration

Status: planned; internal signed tag, publication at v0.270.0.

Goal: Sync progresses or fails with explicit recoverable state.

Scope: bounded milestone. Depends on v0.268.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Backpressured header, body, blob, receipt, execution, and state-healing stages
  with bounded queues and shared resource capabilities;
- durable checkpoints, bad-block caches, peer accountability, progress
  persistence, invalidation, restart, and strategy selection;
- bad-block and invalid-proof caches accept only `v0.93.0`
  `ObjectInvalidityEvidence` and follow its identity, invalidation, retention,
  and anti-flood rules; peer evidence remains in separately scoped peer state;
- immediate cancellation of obsolete work when fork choice changes.

Verification:

- Interrupted sync and competing-chain simulations.

Exit criteria:

- Sync progresses or fails with explicit recoverable state.
- `v0.269.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.270.0 - Multi-Peer Full And Snap Sync

Status: planned; public crates.io checkpoint after cumulative review.

Goal: A node can reach verified canonical state without trusting one peer.

Scope: bounded milestone. Depends on v0.269.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Peer assignment, snapshot-bound shared-node multiproofs, deduplication,
  proof-backed ranges, healing, pivot changes, finalization, and canonical
  import integration;
- all promoted state must use the composed account/storage proof capabilities
  from `v0.52.5` and immutable resolver snapshots from `v0.52.6`.

Verification:

- Multi-client local network, malicious peer, restart, and reorg tests.

Exit criteria:

- A node can reach verified canonical state without trusting one peer.
- `v0.270.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.271.0 - Portal And Historical Data Acquisition

Status: planned; internal signed tag, publication at v0.275.0.

Goal: Expired historical data has an explicit verified acquisition path.

Scope: bounded milestone. Depends on v0.270.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Portal/history network boundary, content keys, proofs, provider fallback, history-expiry aware retrieval, and provenance.

Verification:

- Portal/reference fixtures and unavailable/corrupt source tests.

Exit criteria:

- Expired historical data has an explicit verified acquisition path.
- `v0.271.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.272.0 - Builder Validator And Network Hardening

Status: planned; internal signed tag, publication at v0.275.0.

Goal: Networking, sync, and node-adjacent boundaries are production candidates.

Scope: bounded milestone. Depends on v0.271.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Payload-builder/validator boundaries, gossip/pool interactions, load/DoS suite, interoperability matrix, and pentest.

Verification:

- Cross-client network matrix and clean retest.

Exit criteria:

- Networking, sync, and node-adjacent boundaries are production candidates.
- `v0.272.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 17: Statelessness Commitment Evolution And Future Forks

### v0.273.0 - Proof Format Abstraction

Status: planned; internal signed tag, publication at v0.275.0.

Goal: MPT is no longer hardwired into every proof consumer.

Scope: bounded milestone. Depends on v0.272.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Commitment/proof traits, domain-separated roots/keys, batch proofs,
  capability negotiation, and migration-safe evidence types;
- authority-tagged proof categories: consensus-embedded proofs consume
  `ConsensusRules`, network-carried MPT/range proofs consume `WireLimits` plus
  verification `WorkBudget`, and provider proofs consume RPC
  `OperationalLimits` plus verification work;
- no proof adapter may convert wire or operational rejection into consensus or
  cryptographic invalidity.

Verification:

- Compile-fail authority-domain conversions and cross-format fixtures proving
  valid proofs survive RPC policy refusal, oversized transport envelopes, and
  retry after local work exhaustion.
- Backend conformance suite and domain-substitution compile tests.

Exit criteria:

- MPT is no longer hardwired into every proof consumer.
- `v0.273.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.274.0 - Execution Witness Model

Status: planned; internal signed tag, publication at v0.275.0.

Goal: State dependencies of execution can be represented explicitly.

Scope: bounded milestone. Depends on v0.273.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Owned/borrowed witness data for accounts, storage, code, block context, accesses, writes, and missing-node diagnostics.

Verification:

- Canonical encoding, limit tests, mutation/property tests.

Exit criteria:

- State dependencies of execution can be represented explicitly.
- `v0.274.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.275.0 - MPT Witness Construction And Verification

Status: planned; public crates.io checkpoint after cumulative review.

Goal: MPT-backed execution inputs can be proven complete.

Scope: bounded milestone. Depends on v0.274.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Build minimal MPT witnesses, verify completeness/correctness, deduplicate nodes, and bind them to roots and transactions/blocks.

Verification:

- State-test derived witnesses, omission/substitution fuzzing.

Exit criteria:

- MPT-backed execution inputs can be proven complete.
- `v0.275.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.276.0 - Stateless Execution

Status: planned; internal signed tag, publication at v0.280.0.

Goal: Claimed execution can run without a full local state database.

Scope: bounded milestone. Depends on v0.275.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Execute transactions/blocks from witnesses, reject missing/extraneous invalid evidence, and emit post-state commitments/deltas.

Verification:

- Stateful-versus-stateless differential fixtures.

Exit criteria:

- Claimed execution can run without a full local state database.
- `v0.276.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.277.0 - Verkle Or Successor Commitment Boundary

Status: planned; internal signed tag, publication at v0.280.0.

Goal: Future state commitments fit the shared proof model without pretending unfinished cryptography is implemented.

Scope: bounded milestone. Depends on v0.276.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- First-party format/rule model and audited cryptographic backend boundary for the officially selected successor commitment scheme.

Verification:

- Pinned official vectors and backend-admission review.

Exit criteria:

- Future state commitments fit the shared proof model without pretending unfinished cryptography is implemented.
- `v0.277.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.278.0 - Successor Commitment Scheme Admission

Status: planned; internal signed tag, publication at v0.280.0.

Goal: choose an exact adopted commitment scheme before implementing a speculative backend.

Scope: implementation pass. Depends on v0.277.0. The retained
workstream contract at v0.281.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Pin selection status, cryptographic assumptions, canonical formats, required operations and independent oracle; isolate unselected research.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Source-status review, parameter/setup substitution and prototype interoperability vectors.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No successor claim relies on a placeholder or an unspecified future scheme.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.278.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.279.0 - Successor Commitment Arithmetic

Status: planned; internal signed tag, publication at v0.280.0.

Goal: implement the cryptographic operations selected by the admission manifest.

Scope: implementation pass. Depends on v0.278.0. The retained
workstream contract at v0.281.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement the exact field/group/hash or polynomial operations required by the selected scheme in first-party bounded modules; enumerate the closed operation inventory before coding.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent arithmetic vectors, canonical encodings, malformed inputs and side-channel/resource tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Every admitted primitive has independent evidence; unsupported research is not enabled.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.279.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.280.0 - Successor Commitment Proof Construction

Status: planned; public crates.io checkpoint after cumulative review.

Goal: make successor commitments and proofs executable before state migration.

Scope: implementation pass. Depends on v0.279.0. The retained
workstream contract at v0.281.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement commitment and proof construction/verification, key mapping and serialization using the admitted arithmetic and immutable parameters.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent proof vectors, wrong root/key/parameter substitution and adversarial work bounds.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Commitments and proofs interoperate before any canonical-state migration.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.280.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.281.0 - Successor Commitment Backend Completion

Status: planned; internal signed tag, publication at v0.285.0.

Goal: The selected successor proof scheme is cryptographically executable.

Scope: completion and integration pass. Depends on v0.280.0.
The implementation passes v0.278.0 through v0.280.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
proof backend/format integration and readmission before migration.

Deliverables:

- Implement/admit polynomial/vector commitment arithmetic, key mapping, proof creation/verification, and canonical serialization required by the selected fork.

Verification:

- Official and independent vectors, differential checks, performance/pentest.

Exit criteria:

- The selected successor proof scheme is cryptographically executable.
- `v0.281.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.282.0 - Successor Witness And State Integration

Status: planned; internal signed tag, publication at v0.285.0.

Goal: Historical MPT and successor states coexist with explicit fork rules.

Scope: bounded milestone. Depends on v0.281.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Trie/state migration, witness generation, stateless execution, sync/storage, and root validation for the successor scheme.

Verification:

- Transition and mixed-era fixtures, crash/reorg tests.

Exit criteria:

- Historical MPT and successor states coexist with explicit fork rules.
- `v0.282.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.283.0 - State Expiry And Address Evolution

Status: planned; internal signed tag, publication at v0.285.0.

Goal: State-lifecycle evolution is implemented when specified, not left as an architectural surprise.

Scope: bounded milestone. Depends on v0.282.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Implement officially adopted state-expiry, address-extension, resurrection, access-list, or migration rules with archive/provider policy.

Verification:

- Official fork fixtures and long-horizon state simulations.

Exit criteria:

- State-lifecycle evolution is implemented when specified, not left as an architectural surprise.
- `v0.283.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.284.0 - ZK Execution Proof Boundary

Status: planned; internal signed tag, publication at v0.285.0.

Goal: ZK proof systems can integrate without becoming an implicit consensus dependency.

Scope: bounded milestone. Depends on v0.283.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Versioned proof/public-input types, verifier trait, fork/circuit binding, recursion/batch policy, and explicit trust/error evidence.

Verification:

- Mock and admitted verifier conformance, malformed proof corpus.

Exit criteria:

- ZK proof systems can integrate without becoming an implicit consensus dependency.
- `v0.284.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.285.0 - Future Fork Automation

Status: planned; public crates.io checkpoint after cumulative review.

Goal: New hard forks cannot silently outrun the support matrix.

Scope: bounded milestone. Depends on v0.284.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Monitor official specs/EIPs/fixtures, generate drift reports and candidate
  manifests, and require a named maintenance release for every adopted change;
- maintain an emergency security lane and fork-readiness lane independently of
  the next feature milestone;
- expand the early `v0.240.0` vertical devnet whenever an adopted fork changes
  execution, Engine, consensus, networking, storage, or validator behavior.

Verification:

- Scheduled checker tests and simulated upstream fork changes.

Exit criteria:

- New hard forks cannot silently outrun the support matrix.
- `v0.285.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 18: Foundation Assurance Before Full Consensus Client

### v0.286.0 - Platform And Target Matrix

Status: planned; internal signed tag, publication at v0.290.0.

Goal: Every promised platform has repeatable evidence or an explicit limitation.

Scope: bounded milestone. Depends on v0.285.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Linux, Windows, BSD, macOS, Android, iOS, WASM where applicable, big/little-
  endian review, and Aesynx-readiness constraints;
- target-specific stack ceilings for `v0.106.0` slot/index/arena placement,
  requiring caller/external storage above each audited bound.

Verification:

- Cross-target builds/tests and documented unsupported combinations;
- stack-usage reports and boundary fixtures proving large evidence arenas are
  never placed on undersized target stacks.

Exit criteria:

- Every promised platform has repeatable evidence or an explicit limitation.
- `v0.286.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.287.0 - Whole-System Performance Program

Status: planned; internal signed tag, publication at v0.290.0.

Goal: Performance and DoS budgets are release-blocking rather than anecdotal.

Scope: bounded milestone. Depends on v0.286.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Benchmarks and budgets for codec, crypto, EVM, proofs, providers, storage,
  sync, ABI, wallets, and end-to-end workflows;
- integrate every `v0.104.0` evidence benchmark: valid transactions, complete
  blocks across transaction counts, gossip across worker counts, high-
  contention batches, `FirstInvalid`, admitted diagnostic modes, stack/context/
  reservation/code sizes, allocation/lock/atomic/clone/sink counts, and peak/
  retained memory;
- preserve an internal benchmark-only evidence-disabled baseline and fail on
  relative or absolute threshold regressions; production features can never
  disable evidence authority;
- enforce `v0.106.0` paired-run integrity: identical input/context/schedule/
  validator/work counters/output consumption, only evidence operations
  replaced on valid paths, and separate evidence-attributable versus total
  allocation reports;
- enforce `v0.109.0` invalid semantic projections/minimal-evidence baselines,
  protocol-versus-evidence operation counters, untimed setup/result hashing/
  equality/barrier preparation, uninstrumented production thresholds, and
  separate non-perturbing instrumented conformance runs;
- absolute production thresholds remain authoritative and cannot be waived by
  a favorable relative disabled-baseline delta.

Verification:

- Reproducible benchmark runner and regression thresholds;
- cross-platform/toolchain variance policy and seeded evidence-overhead
  regressions proving each threshold is release-blocking;
- harness structural-equivalence and dead-code-elimination audit reports;
- timed-boundary, instrumentation-self-test, invalid-projection, and minimal-
  evidence-baseline reports.

Exit criteria:

- Performance and DoS budgets are release-blocking rather than anecdotal.
- `v0.287.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.288.0 - Structure-Aware Fuzz And Complexity Oracles

Status: planned; internal signed tag, publication at v0.290.0.



Goal: test valid deep behavior and enforce work bounds, not merely compile
fuzz targets or reject random bytes at the first root mismatch.

Scope: bounded milestone. Depends on v0.287.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Structure-aware RLP, transaction, MPT, proof, EVM, precompile, Engine, and
  protocol generators that can construct valid objects and recompute roots;
- mutation stages targeting deep traversal after validity has been established;
- complexity oracles for bytes scanned, headers, nodes, hashes, allocations,
  gas-related work, queue operations, and output;
- continuous sanitizer/coverage fuzz jobs, mutation testing, corpus
  minimization, and a permanent regression fixture for every discovered defect.

Verification:

- Published coverage and complexity-threshold reports;
- seeded findings proving each oracle detects deliberate superlinear or
  budget-reset defects;
- zero unexplained skipped or quarantined regression fixtures.

Exit criteria:

- Fuzzing reaches authenticated deep paths and fails releases on excess work,
  not only on panics or incorrect return values.
- `v0.288.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.289.0 - Kani Codec Primitive And Typestate Proofs

Status: planned; internal signed tag, publication at v0.290.0.

Goal: Selected foundational invariants have machine-checked evidence in addition to tests.

Scope: bounded milestone. Depends on v0.288.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded proofs for arithmetic, canonical decoding, budget accounting, writers,
  conversions, impossible typestate transitions, and the `v0.100.0` evidence-
  slot state machine;
- slot-conservation proofs across reserve, derive, fill, return, and release,
  including one-entry/two-entry parent-child composition and bounded batch
  cardinality;
- `v0.102.0` mode proofs showing collection-limit changes preserve validity and
  first evidence, immutable sink borrows cannot transition authority, and each
  additional authoritative record consumes a distinct slot;
- `v0.106.0` capacity/handle proofs showing attacker counts cannot size arenas,
  partial capability reservation conserves resources, stale generations cannot
  access reused slots, and live borrows prevent release/reset within documented
  bounds;
- `v0.109.0` dispatch/generation proofs showing upward internal class mapping
  enforces the exact requested stop, rejected values cannot begin validation,
  and maximum-generation identities retire rather than wrap.

Verification:

- Pinned Kani toolchain and reproducible proof report;
- seeded double-fill, double-release, child-minting, use-after-return, and slot-
  leak models rejected within documented bounds;
- seeded mode-dependent-validity, sink-consumes-authority, and slot-reuse models
  rejected within documented bounds;
- seeded stale-generation, ABA, release-with-live-borrow, and partial-
  reservation-leak models rejected within documented bounds;
- seeded over-collection, rounding-down, above-maximum admission, and
  generation-resurrection models rejected within documented bounds.

Exit criteria:

- Selected foundational invariants have machine-checked evidence in addition to tests.
- `v0.289.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.290.0 - Kani EVM Trie And State Proofs

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Selected consensus-critical execution invariants have machine-checked evidence.

Scope: bounded milestone. Depends on v0.289.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded proofs for stack/gas/memory arithmetic, journal rollback, trie paths, proof verification, and state-transition invariants.

Verification:

- Reproducible proof harnesses with documented bounds/assumptions.

Exit criteria:

- Selected consensus-critical execution invariants have machine-checked evidence.
- `v0.290.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.291.0 - Kani Cryptographic Arithmetic Proofs

Status: planned; internal signed tag, publication at v0.295.0.



Goal: add machine-checked implementation evidence for the highest-risk
first-party field, scalar, curve, signature, and pairing arithmetic.

This release consolidates and extends the early secp256k1 subset admitted at
`v0.94.0`; it does not defer secret-bearing secp proof evidence until here.

Scope: bounded milestone. Depends on v0.290.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded carry/borrow proofs for limb addition and subtraction;
- wide multiplication and modular-reduction invariants;
- canonical field/scalar conversion and serialization round trips with
  non-canonical rejection;
- inversion and square-root postconditions;
- point addition/doubling exceptional cases, including infinity, equality, and
  inverses;
- secret scalar multiplication equivalence against a small mathematical model;
- documented proof bounds where full 256-bit state exploration is infeasible,
  paired with independent arithmetic specifications and differential tests.

Verification:

- Pinned Kani harnesses and reproducible reports for Keccak/secp256k1, BN254,
  BLS12-381, KZG, and related fixed-width helpers as applicable;
- compatibility and coverage checks proving the `v0.94.0` harnesses remain
  attached to the production secp implementation;
- seeded carry, reduction, exceptional-point, and serialization defects caught
  by the proof suite;
- independent review of the mathematical models, assumptions, and uncovered
  domains.

Exit criteria:

- Critical arithmetic implementation invariants have explicit machine-checked
  evidence, and every unproved full-width claim has documented differential,
  property, vector, fuzz, and audit coverage.
- `v0.291.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.292.0 - Miri Sanitizers And Undefined-Behavior Gate

Status: planned; internal signed tag, publication at v0.295.0.

Goal: Dynamic memory/UB evidence complements the first-party unsafe-code ban.

Scope: bounded milestone. Depends on v0.291.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Miri for applicable crates/tests, address/thread/memory sanitizers where supported, stack-use analysis, and unsafe-dependency review.

Verification:

- Reproducible tool reports and zero unexplained failures.

Exit criteria:

- Dynamic memory/UB evidence complements the first-party unsafe-code ban.
- `v0.292.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.293.0 - Protocol And Concurrency Model Checking

Status: planned; internal signed tag, publication at v0.295.0.



Goal: complement Rust-level bounded proofs with explicit distributed and
concurrent state-machine models.

Scope: bounded milestone. Depends on v0.292.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- TLA+, Quint, or Apalache models for fork activation, Engine sequencing,
  fork choice, txpool replacement, and slashing invariants;
- Loom models for request IDs, schedulers, txpool coordination, resource-token
  conservation, hierarchical evidence-slot derivation/fill/return/release, and
  slashing-database concurrency;
- `v0.106.0` arena models for capability reservation, scoped/generation handle
  reuse, live borrows, cancellation, timeout, cross-worker transfer, reset,
  drain, and optional pool ownership;
- trace-to-test adapters that turn model counterexamples into deterministic
  Rust regressions.

Verification:

- Pinned model-checker versions and reproducible reports;
- seeded broken invariants detected by each model family;
- concurrent evidence tests covering racing child derivation, fill versus
  cancellation, return versus release, and exactly-once parent accounting;
- seeded stale-handle/ABA, release-with-live-borrow, transfer/reset, and pool-
  generation defects detected within documented bounds;
- zero unexplained counterexamples within documented bounds.

Exit criteria:

- Critical protocol and concurrency claims have explicit machine-checked
  state models in addition to implementation tests.
- `v0.293.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.294.0 - Side-Channel And Gas-To-Cycles Assurance

Status: planned; internal signed tag, publication at v0.295.0.



Goal: apply the correct assurance model separately to secrets and public
consensus computation.

Scope: bounded milestone. Depends on v0.293.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- dudect/ctgrind-style tests and assembly review for secret-bearing key,
  nonce, scalar, KDF, derivation, and signing paths;
- adversarial cycles-per-gas and maximum-output benchmarks for every public
  precompile and other gas-bearing cryptographic path;
- backend sanitization and constant-time admission reports plus compiler and
  platform caveats;
- documentation that promises bounded public work and best-effort secret
  erasure/isolation rather than identical public-input timing or absolute
  zeroization.

Verification:

- Reproducible side-channel and adversarial benchmark reports on named
  platforms/toolchains;
- release thresholds with reviewed variance and no unexplained outliers.

Exit criteria:

- Secret paths have fixed-work/constant-time evidence, while public paths have
  enforceable worst-case work per charged gas.
- `v0.294.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.295.0 - Compatibility And Semver Gate

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Accidental breaking or stale publication metadata blocks release.

Scope: bounded milestone. Depends on v0.294.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- cargo-semver-checks, feature powerset, minimal/default/all-feature graphs,
  README dependency versions, serde/text snapshots, and MSRV/stable checks;
- public API guards from `v0.104.0`: stable validation outcome shapes, no
  internal evidence lifetime/slot/arena/sink/worker-pool exposure, no arbitrary
  cardinality monomorphization, and no feature- or mode-dependent consensus
  behavior or return type;
- enforce the `v0.106.0` public non-generic validated mode/configuration value
  and private admitted-implementation dispatch; benchmark-disabled baselines
  cannot appear in production features or public APIs;
- enforce `v0.109.0` zero/above-maximum rejection, upward internal capacity-
  class mapping with exact requested stop, stable mode-independent
  `ValidationOutcome<T, E>`, and private benchmark semantic projections;
- code-size and monomorphization budgets for every admitted collection mode and
  feature graph.

Verification:

- Automated compatibility report for every published crate;
- API snapshots, compile-fail containment cases, feature-power-set behavior
  equivalence, and generated-code-size reports;
- public mode boundary/configuration compatibility tests and proof that
  benchmark-only projections/baselines are absent from production APIs.

Exit criteria:

- Accidental breaking or stale publication metadata blocks release.
- `v0.295.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.296.0 - Task-Oriented Documentation

Status: planned; internal signed tag, publication at v0.300.0.

Goal: Public functionality is discoverable without reading internal source.

Scope: bounded milestone. Depends on v0.295.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Complete guides for decoding, verification, execution, providers, wallets, contracts, storage, light clients, sync, stateless operation, and migration.

Verification:

- Doctests, link checks, fresh-user task exercises.

Exit criteria:

- Public functionality is discoverable without reading internal source.
- `v0.296.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.297.0 - Core SDK API Stability Baseline

Status: planned; internal signed tag, publication at v0.300.0.

Goal: Later consensus-client work builds on deliberate foundation contracts without pretending the complete 1.0 API is frozen.

Scope: bounded milestone. Depends on v0.296.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Stabilize naming, ownership, errors, features, deprecation, compatibility, and migration policy for the core, SDK, execution, provider, wallet, contract, storage, light-client, and networking foundations.

Verification:

- Public API review and semver baseline for admitted foundation crates.

Exit criteria:

- Later consensus-client work builds on deliberate foundation contracts without pretending the complete 1.0 API is frozen.
- `v0.297.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.298.0 - Core Cryptography And Codec Audit

Status: planned; internal signed tag, publication at v0.300.0.

Goal: No unresolved critical/high core finding remains.

Scope: bounded milestone. Depends on v0.297.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent review of primitives, hashing, signatures, BLS/KZG, RLP/SSZ/ABI, proofs, and sanitization boundaries.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high core finding remains.
- `v0.298.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.299.0 - Execution Storage And Light-Client Audit

Status: planned; internal signed tag, publication at v0.300.0.

Goal: No unresolved critical/high finding remains in the execution/client foundation or light-client scope.

Scope: bounded milestone. Depends on v0.298.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent review of EVM, execution state transition, tries, storage, execution fork choice, Engine boundaries, light-client paths, and stateless execution.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high finding remains in the execution/client foundation or light-client scope.
- `v0.299.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.300.0 - Provider Wallet And Contract Audit

Status: planned; public crates.io checkpoint after cumulative review.

Goal: No unresolved critical/high SDK or key-management finding remains.

Scope: bounded milestone. Depends on v0.299.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent review of transports, trust layers, transaction lifecycle, signers, keystores, account abstraction, ABI, and codegen.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high SDK or key-management finding remains.
- `v0.300.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.301.0 - Execution Networking Sync And Runtime Audit

Status: planned; internal signed tag, publication at v0.305.0.

Goal: No unresolved critical/high finding remains in the execution-network or runtime foundation.

Scope: bounded milestone. Depends on v0.300.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent review of RLPx/discovery, execution peer management, txpool, execution sync, runtime supervision, pruning, and operational DoS controls.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high finding remains in the execution-network or runtime foundation.
- `v0.301.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.302.0 - Foundation Remediation Release

Status: planned; internal signed tag, publication at v0.305.0.

Goal: The SDK, execution, storage, light-client, and execution-network foundation is ready to host the full consensus client.

Scope: bounded milestone. Depends on v0.301.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Resolve residual foundation audit, conformance, compatibility, documentation, and performance findings; record accepted low risks.

Verification:

- Full gate, all foundation retests, zero unexplained skips, updated SBOM/provenance.

Exit criteria:

- The SDK, execution, storage, light-client, and execution-network foundation is ready to host the full consensus client.
- `v0.302.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.303.0 - Full-Stack Foundation Integration Baseline

Status: planned; internal signed tag, publication at v0.305.0.

Goal: Full beacon-node and validator work starts from a reviewed integrated foundation rather than an assumed 1.0 candidate.

Scope: bounded milestone. Depends on v0.302.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Exercise all foundation layers together, lock component interoperability contracts, rehearse publication order, and publish the pre-consensus-client evidence set.

Verification:

- Exact-candidate pentest, reproducible packages, local execution-node matrix, green CI.

Exit criteria:

- Full beacon-node and validator work starts from a reviewed integrated foundation rather than an assumed 1.0 candidate.
- `v0.303.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 19: Full Consensus Client Foundation

This phase extends the consensus types and light-client work into a production
beacon-node foundation. The beacon node coordinates with, but does not replace,
an execution client through the authenticated Engine API. Stable fork support
must follow pinned official consensus releases; experimental Gloas, Heze, or
successor work remains generated, modular, and feature-gated until officially
admitted.

Consensus-client source review date: 2026-07-16:

- <https://github.com/ethereum/consensus-specs>
- <https://ethereum.github.io/consensus-specs/phase0/beacon-chain/>
- <https://ethereum.github.io/consensus-specs/phase0/fork-choice/>
- <https://ethereum.github.io/consensus-specs/phase0/p2p-interface/>
- <https://ethereum.github.io/consensus-specs/phase0/weak-subjectivity/>
- <https://ethereum.github.io/consensus-specs/sync/optimistic/>
- <https://ethereum.github.io/consensus-specs/fulu/validator/>

### v0.304.0 - Consensus Client Architecture And Threat Model

Status: planned; internal signed tag, publication at v0.305.0.

Goal: No consensus-client implementation begins with ambiguous ownership, trust, or persistence boundaries.

Scope: bounded milestone. Depends on v0.303.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Define beacon-node, validator, signer, slashing, builder, Engine, storage, network, sync, and data-availability trust boundaries; allocate focused crates and resource ceilings.

Verification:

- Architecture review, dependency classification, abuse-case register, crate-cycle check.

Exit criteria:

- No consensus-client implementation begins with ambiguous ownership, trust, or persistence boundaries.
- `v0.304.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.305.0 - Consensus Configuration And Fork Registry

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Consensus behavior is source-generated and fork-modular rather than spread through optional-field conditionals.

Scope: bounded milestone. Depends on v0.304.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Network presets, chain configurations, genesis data, fork schedules, fork digests, domain constants, typed stable-fork variants, and generated experimental-fork modules.

Verification:

- Official preset/config fixtures, fork-digest vectors, source-lock drift tests.

Exit criteria:

- Consensus behavior is source-generated and fork-modular rather than spread through optional-field conditionals.
- `v0.305.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.306.0 - Progressive SSZ Containers And Proofs

Status: planned; internal signed tag, publication at v0.310.0.

Goal: add admitted progressive data structures without changing legacy SSZ roots.

Scope: implementation pass. Depends on v0.305.0. The retained
workstream contract at v0.308.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement forward-compatible field/list limits, indices and proof rules from standalone SSZ and fork manifests.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Progressive soft limits, mixed-version containers and legacy-root regressions.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Progressive and legacy objects have unambiguous authenticated identities.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.306.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.307.0 - Mutable SSZ Trees And Cached Roots

Status: planned; internal signed tag, publication at v0.310.0.

Goal: support transactional state mutation without stale Merkle evidence.

Scope: implementation pass. Depends on v0.306.0. The retained
workstream contract at v0.308.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement incremental roots, multiproofs, cache identities and rollback-safe mutation under bounded work.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Full-recompute oracle, interleaved mutation, cache invalidation and rollback faults.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No cached root survives a mutation with an invalid identity.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.307.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.308.0 - Complete SSZ Client Surface Completion

Status: planned; internal signed tag, publication at v0.310.0.

Goal: extend the immutable `v0.235.0` SSZ foundation into the complete mutable,
cached, proof-capable surface required by production beacon state and
networking.

Scope: completion and integration pass. Depends on v0.307.0.
The implementation passes v0.306.0 through v0.307.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
mutable/progressive/legacy SSZ compatibility and public client surface integration.

Deliverables:

- Complete production containers, lists, vectors, bitlists, and bitvectors;
- incremental hash-tree roots;
- mutable generalized-index operations;
- branches and multiproofs;
- cached trees and cache-invalidation rules;
- cached SSZ/Merkle entries follow chain/genesis, fork, state-root/generalized-
  index, and validation-level identity;
- bounded transactional mutation APIs;
- all decoding, offset traversal, bitlist/list work, hashing, Merkleization,
  mutation, allocation, and proof output consume `v0.88.0` child ledgers;
- compatibility with the canonical encoding and baseline roots from
  `v0.235.0`.

Verification:

- Official SSZ vectors;
- incremental-versus-full-root differential tests;
- cache invalidation and mutation rollback tests;
- malformed-offset and proof fuzzing;
- compatibility tests against `v0.235.0` encodings and roots.

Exit criteria:

- Beacon state and network objects can use first-party SSZ without missing
  production mutation, caching, container, or proof operations, and without
  redefining the foundational codec.
- `v0.308.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.309.0 - Consensus BLS Secret Signing

Status: planned; internal signed tag, publication at v0.310.0.

Goal: admit secret-bearing BLS operations independently of public verification.

Scope: implementation pass. Depends on v0.308.0. The retained
workstream contract at v0.311.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement secret key generation/ownership and ciphersuite signing using fixed-work arithmetic and wiped scratch.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official key/signature vectors, domain separation, fault/entropy failure and timing review.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Signing keys never enter public verification caches or escape after failure.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.309.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.310.0 - BLS Batch Verification And Isolation

Status: planned; public crates.io checkpoint after cumulative review.

Goal: batch consensus verification without unsound attribution.

Scope: implementation pass. Depends on v0.309.0. The retained
workstream contract at v0.311.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement nonzero transcript/random coefficients, bounded latency and individually evidenced failure isolation.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Mixed-source batches, coefficient reuse, queue starvation and local failure tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Batch failures cannot slash or penalize unidentified participants.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.310.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.311.0 - BLS Signing Aggregation And Batch Verification Completion

Status: planned; internal signed tag, publication at v0.315.0.

Goal: Consensus and validator paths have a complete first-party BLS surface, not verification-only light-client hooks.

Scope: completion and integration pass. Depends on v0.310.0.
The implementation passes v0.309.0 through v0.310.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
aggregation/proof-of-possession policy, signing and batch-queue integration.

Deliverables:

- First-party BLS signing, verification, aggregation, aggregate verification,
  randomized batch verification, proof of possession where required,
  secret/public key domains, and optional acceleration only behind audited
  adapters;
- nonzero attacker-unpredictable coefficients or a reviewed domain-separated
  transcript covering every public key, message, domain, fork, and signature;
- entropy/transcript failure fails closed, coefficient reuse across contexts is
  prohibited, and bounded batch-failure isolation cannot become an unbounded
  fallback attack;
- failed mixed-source batches return `v0.93.0` `BatchContainsInvalid` and
  never identify or penalize members/peers until bounded individual
  verification establishes member-specific object-invalidity evidence;
- follow `v0.100.0`: reserve one batch slot and a configured batch-size-
  independent maximum of member slots before isolation; capacity exhaustion
  stops isolation locally, and only filled member slots authorize caching or
  attribution;
- member isolation uses `v0.102.0` `BatchIsolateUpTo<N>` and treats `N` only
  as an operational attribution/work limit; members beyond it remain
  unattributed without changing aggregate verification validity;
- bounded verification-queue latency and maximum batch age so attackers cannot
  delay block processing by preventing batches from filling;
- cache keys contain complete message/domain/fork/validation context.

Verification:

- Official BLS vectors, independent differential tests, subgroup/fault/batch
  fuzzing, coefficient/transcript/cache attacks, queue-latency simulations,
  entropy-failure tests, evidence-capacity-pressure/cardinality tests, and
  timing review;
- cross-`N` aggregate-result invariance and beyond-limit non-attribution tests;
- high-contention verification/isolation benchmarks across batch sizes and
  worker counts against the `v0.104.0` baseline, including allocation, global-
  contention, stack, arena, and retained-memory thresholds.

Exit criteria:

- Consensus and validator paths have a complete first-party BLS surface, not verification-only light-client hooks.
- `v0.311.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.312.0 - PeerDAS Cell Proofs

Status: planned; internal signed tag, publication at v0.315.0.

Goal: construct and verify cells before reconstruction or networking.

Scope: implementation pass. Depends on v0.311.0. The retained
workstream contract at v0.315.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement blob extension, cell encoding, column commitments and single-cell KZG proof creation/verification.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official cell vectors, coordinate/setup substitution and invalid field tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Cell evidence is bound to exact blob, coordinate and setup.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.312.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.313.0 - PeerDAS Erasure Reconstruction

Status: planned; internal signed tag, publication at v0.315.0.

Goal: recover missing data from verified cells under bounded work.

Scope: implementation pass. Depends on v0.312.0. The retained
workstream contract at v0.315.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement erasure decoding and reusable workspace with explicit insufficient-data and malformed-input outcomes.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Independent reconstruction oracle, adversarial erasures, duplicate coordinates and allocation/work exhaustion.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Recovered data matches commitments and insufficient data remains retryable.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.313.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.314.0 - PeerDAS Batch Proof Admission

Status: planned; internal signed tag, publication at v0.315.0.

Goal: integrate sound batch verification without changing cell authority.

Scope: implementation pass. Depends on v0.313.0. The retained
workstream contract at v0.315.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement coefficient policy, cache identities and bounded member isolation with independent evidence reservations.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Mixed columns/sources, entropy faults, failed isolation and setup/domain changes.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Only individually established evidence can identify an invalid cell.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.314.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.315.0 - PeerDAS Cell And Reconstruction Core Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: implement the first-party cryptographic and erasure-coding core before
any state-transition, storage, networking, synchronization, or validator path
consumes PeerDAS data.

Scope: completion and integration pass. Depends on v0.314.0.
The implementation passes v0.312.0 through v0.314.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
cell/reconstruction/batch workspace and availability-domain integration.

Deliverables:

- Blob-to-cell conversion;
- cell KZG proof creation and verification;
- data-column construction;
- erasure coding and bounded reconstruction;
- batch verification;
- sound nonzero coefficient/transcript generation, fail-closed entropy,
  context-complete cache identities, and bounded failure isolation;
- mixed-source batch failures return `v0.93.0` `BatchContainsInvalid`; local
  isolation failures remain local and member/peer attribution requires
  individual object-invalidity evidence;
- `v0.100.0` batch reservations bound the number of member evidence records
  independently of cell/column count and prohibit cache or peer attribution
  for members whose child slots were not filled;
- `v0.102.0` `BatchIsolateUpTo<N>` bounds attribution work and leaves cells or
  columns beyond `N` unattributed without changing the batch result;
- bounded reusable workspaces;
- explicit acceleration/backend boundaries;
- canonical failure and partial-output behavior.

Verification:

- Official EIP-7594 and pinned current-fork vectors;
- independent differential checks;
- malformed proof, cell, and reconstruction fuzzing;
- corruption and insufficient-column tests;
- CPU, memory, and workspace ceilings;
- multi-invalid isolation under evidence-slot pressure and maximum-result-
  cardinality tests;
- cross-`N` validity invariance and beyond-limit non-attribution tests;
- high-contention cell/column verification and isolation benchmarks against the
  `v0.104.0` evidence-disabled baseline, including allocation, contention,
  arena, stack, and retained-memory thresholds;
- default-graph and backend-admission checks.

Exit criteria:

- Data columns can be created, verified, and reconstructed first party before
  any downstream milestone treats PeerDAS evidence as actionable.
- `v0.315.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.316.0 - Committees Shuffling Domains And Signing Roots

Status: planned; internal signed tag, publication at v0.320.0.

Goal: Every duty and signature domain is derived from pinned consensus rules.

Scope: bounded milestone. Depends on v0.315.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Validator shuffling, proposer/committee selection, subnet assignments, fork-aware domains, signing roots, RANDAO, aggregator selection, and cached epoch context.

Verification:

- Official shuffling/committee vectors, property tests, cross-fork domain checks.

Exit criteria:

- Every duty and signature domain is derived from pinned consensus rules.
- `v0.316.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 20: Complete Beacon State Transition

### v0.317.0 - Beacon Transition Shell And Per-Slot Processing

Status: planned; internal signed tag, publication at v0.320.0.

Goal: Per-slot processing is complete and failed transitions cannot partially mutate caller-visible state.

Scope: bounded milestone. Depends on v0.316.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Mutable/transactional BeaconState, slot processing, historical roots/summaries, state/block roots, cache policy, and rollback-safe transition errors.

Verification:

- Official slot-transition vectors, output-unchanged failure tests, state-root differential checks.

Exit criteria:

- Per-slot processing is complete and failed transitions cannot partially mutate caller-visible state.
- `v0.317.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.318.0 - Epoch Registry And Balance Processing

Status: planned; internal signed tag, publication at v0.320.0.

Goal: Epoch-wide validator and balance bookkeeping matches the specification.

Scope: bounded milestone. Depends on v0.317.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Registry updates, effective balances, justification/finalization inputs, slashings vectors, inactivity scores, participation rotation, and epoch caches.

Verification:

- Official epoch-component vectors and large-validator-set resource tests.

Exit criteria:

- Epoch-wide validator and balance bookkeeping matches the specification.
- `v0.318.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.319.0 - Activation Exit Churn Withdrawal And Consolidation

Status: planned; internal signed tag, publication at v0.320.0.

Goal: The full validator lifecycle is state-transition complete.

Scope: bounded milestone. Depends on v0.318.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Eligibility, activation queue, exits, churn limits, withdrawals, credential changes, consolidation requests, and fork-specific lifecycle rules.

Verification:

- Official lifecycle vectors, queue/churn properties, historical/current fork tests.

Exit criteria:

- The full validator lifecycle is state-transition complete.
- `v0.319.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.320.0 - Rewards Penalties Participation And Inactivity

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Balance outcomes match official vectors across normal and non-finalizing periods.

Scope: bounded milestone. Depends on v0.319.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Base rewards, attestation deltas, proposer rewards, sync rewards, inactivity leaks, participation flags, and fork-specific accounting.

Verification:

- Official reward/penalty vectors, arithmetic proofs/properties, inactivity simulations.

Exit criteria:

- Balance outcomes match official vectors across normal and non-finalizing periods.
- `v0.320.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.321.0 - Deposits Slashings And Credential Operations

Status: planned; internal signed tag, publication at v0.325.0.

Goal: Every consensus operation that changes validator state is implemented and checked.

Scope: bounded milestone. Depends on v0.320.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Deposit processing and proofs, proposer/attester slashings, voluntary exits, BLS-to-execution changes, pending requests, and duplicate/conflict policy.

Verification:

- Official operation vectors, malformed proof/signature tests, slashing edge cases.

Exit criteria:

- Every consensus operation that changes validator state is implemented and checked.
- `v0.321.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.322.0 - Attestations Sync Committees And Block Operations

Status: planned; internal signed tag, publication at v0.325.0.

Goal: Beacon blocks can process all stable-fork consensus operations.

Scope: bounded milestone. Depends on v0.321.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Attestation validation/processing, indexed attestations, sync aggregates, block header/body operations, operation ordering, and bounded block-operation limits.

Verification:

- Official operation/block vectors and malformed aggregate fuzzing.

Exit criteria:

- Beacon blocks can process all stable-fork consensus operations.
- `v0.322.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.323.0 - ePBS Builder Registry And Requests

Status: planned; internal signed tag, publication at v0.325.0.

Goal: implement protocol-native builder state separately from relay APIs.

Scope: implementation pass. Depends on v0.322.0. The retained
workstream contract at v0.325.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-7732/8282 registry, onboarding/exits, requests and payment accounting under admitted Gloas rules.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official same-key slot reuse, exited-builder, payment equivocation and request-order tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Builder accounting has a tested first-party state transition.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.323.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.324.0 - ePBS Payload Bids And Timeliness

Status: planned; internal signed tag, publication at v0.325.0.

Goal: bind payload envelopes and committee evidence to the correct parent.

Scope: implementation pass. Depends on v0.323.0. The retained
workstream contract at v0.325.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement bid/header/payload validation, payload-timeliness committee transitions and unavailable-parent handling.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Equal parent/block hashes, empty parents, withholding and independent Gloas vectors.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- A bid is not a valid or available payload without all required evidence.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.324.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.325.0 - Execution Payload And Request Processing Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Consensus transition is correctly bound to execution validity and current request types.

Scope: completion and integration pass. Depends on v0.324.0.
The implementation passes v0.323.0 through v0.324.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
historical Merge-through-current payload/request transitions composed with admitted ePBS state.

Deliverables:

- Merge transition rules, execution payload/header processing, withdrawals, deposit receipts, execution requests, consolidations, payload status binding, and Engine evidence.

Verification:

- Official Bellatrix-through-current vectors and invalid execution-status cases.

Exit criteria:

- Consensus transition is correctly bound to execution validity and current request types.
- `v0.325.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.326.0 - Data Availability State Transition

Status: planned; internal signed tag, publication at v0.330.0.

Goal: Consensus transition does not accept data-dependent blocks without the required availability evidence.

Scope: bounded milestone. Depends on v0.325.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Blob/data-column commitments, custody requirements, availability status, block acceptance dependencies, retention metadata, and stable-fork DA rules.

Verification:

- Official Deneb/Fulu/current vectors, missing/invalid sidecar tests, custody calculations.

Exit criteria:

- Consensus transition does not accept data-dependent blocks without the required availability evidence.
- `v0.326.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.327.0 - Explicit Consensus Fork Upgrades

Status: planned; internal signed tag, publication at v0.330.0.

Goal: Every supported fork transition is explicit, tested, and free of implicit optional-field reinterpretation.

Scope: bounded milestone. Depends on v0.326.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- State upgrade functions between every supported stable fork, migration invariants, typed pre/post states, and experimental-fork admission policy.

Verification:

- Official fork-upgrade vectors and round-trip migration audits.

Exit criteria:

- Every supported fork transition is explicit, tested, and free of implicit optional-field reinterpretation.
- `v0.327.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.328.0 - Complete State-Transition Vector Gate

Status: planned; internal signed tag, publication at v0.330.0.

Goal: The complete beacon state transition is fixture-backed for every claimed stable fork.

Scope: bounded milestone. Depends on v0.327.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Run all official operation, epoch, transition, fork-upgrade, sanity, finality, random, and current stable-fork suites with generated coverage reports.

Verification:

- Zero unexplained skips for claimed forks and cross-client state-root samples.

Exit criteria:

- The complete beacon state transition is fixture-backed for every claimed stable fork.
- `v0.328.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 21: Production Consensus Fork Choice And Beacon Chain

### v0.329.0 - Transactional Fork-Choice Store

Status: planned; internal signed tag, publication at v0.330.0.

Goal: Fork-choice updates are transactional as required by the specification.

Scope: bounded milestone. Depends on v0.328.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Store contract, atomic handlers, tick/block/attestation/slashing inputs, checkpoint states, latest messages, invalid-input rollback, and bounded caches.

Verification:

- Official handler tests, mutation-failure injection, property tests proving invalid calls preserve store state.

Exit criteria:

- Fork-choice updates are transactional as required by the specification.
- `v0.329.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.330.0 - LMD-GHOST And Latest Messages

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Head computation matches LMD-GHOST under competing branches and votes.

Scope: bounded milestone. Depends on v0.329.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Ancestry, latest-message tracking, vote weights, filtered trees, head selection, equivocation handling, and efficient incremental updates.

Verification:

- Official fork-choice vectors, randomized tree differential tests, scale benchmarks.

Exit criteria:

- Head computation matches LMD-GHOST under competing branches and votes.
- `v0.330.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.331.0 - Fork Choice FFG And Proposer Boost

Status: planned; internal signed tag, publication at v0.335.0.

Goal: complete checkpoint and proposer weighting before newer PBS rules.

Scope: implementation pass. Depends on v0.330.0. The retained
workstream contract at v0.334.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement justified/finalized/unrealized checkpoints, boost and historical reorg rules on the transactional store.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official fork-choice vectors, conflicting checkpoints and weight-removal/reorg tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Historical head selection matches the pinned specification.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.331.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.332.0 - ePBS Fork Choice Integration

Status: planned; internal signed tag, publication at v0.335.0.

Goal: integrate payload timeliness and builder outcomes into head selection.

Scope: implementation pass. Depends on v0.331.0. The retained
workstream contract at v0.334.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement admitted Gloas fork-choice rules, invalid/unavailable payload propagation and safe fallback.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Withheld/late payloads, target equivocation, empty parents and independent client traces.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Builder or availability faults cannot silently change finalized safety.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.332.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.333.0 - FOCIL Lists And Inclusion Enforcement

Status: planned; internal signed tag, publication at v0.335.0.

Goal: make inclusion lists executable consensus rules rather than metadata.

Scope: implementation pass. Depends on v0.332.0. The retained
workstream contract at v0.334.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement EIP-7805 list eligibility, timeliness/dependent roots, storage and block inclusion validation under exact fork context.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Wrong roots, reorgs, withheld/conflicting lists, list-size limits and independent inclusion vectors.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Missing or invalid inclusion obligations cannot be bypassed by local block production.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.333.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.334.0 - Casper FFG Proposer Boost And Reorg Policy Completion

Status: planned; internal signed tag, publication at v0.335.0.

Goal: Finality and proposer policies match pinned stable-fork rules.

Scope: completion and integration pass. Depends on v0.333.0.
The implementation passes v0.331.0 through v0.333.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
FFG/ePBS/FOCIL state-machine composition and complete head-selection conformance.

Deliverables:

- Justified/finalized and unrealized checkpoints, proposer boost, proposer reorgs, weak-head/strong-parent rules, and configurable safety policy.

Verification:

- Official vectors, reorg-threshold properties, delayed-attestation simulations.

Exit criteria:

- Finality and proposer policies match pinned stable-fork rules.
- `v0.334.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.335.0 - Optimistic Execution And Invalidation

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Execution-invalid ancestry cannot remain canonical or authorize validator duties.

Scope: bounded milestone. Depends on v0.334.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Optimistic statuses, `latestValidHash`, invalid subtree removal, weight removal, poisoning defenses, Engine failure states, and validator-safety signals.

Verification:

- Official optimistic-sync cases, execution invalidation/reorg simulations, multi-engine fault injection.

Exit criteria:

- Execution-invalid ancestry cannot remain canonical or authorize validator duties.
- `v0.335.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.336.0 - Fork-Choice Persistence And Recovery

Status: planned; internal signed tag, publication at v0.340.0.

Goal: Restarted fork choice returns the same safe/finalized/head state or fails closed.

Scope: bounded milestone. Depends on v0.335.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Durable fork-choice snapshots/logs, atomic import coupling, restart reconstruction, checkpoint recovery, corruption detection, and deterministic replay.

Verification:

- Crash/restart/torn-write simulations and replay equivalence tests.

Exit criteria:

- Restarted fork choice returns the same safe/finalized/head state or fails closed.
- `v0.336.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.337.0 - Beacon Operation Pools

Status: planned; internal signed tag, publication at v0.340.0.

Goal: Block production has complete, bounded, reorg-aware operation sources.

Scope: bounded milestone. Depends on v0.336.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Attestation aggregation, sync contributions, slashings, exits, credential changes, execution/consolidation requests, duplicate suppression, expiry, and block packing.

Verification:

- Pool property/fuzz tests, reorg handling, bounded-memory and packing tests.

Exit criteria:

- Block production has complete, bounded, reorg-aware operation sources.
- `v0.337.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.338.0 - Hot And Finalized Beacon Storage

Status: planned; internal signed tag, publication at v0.340.0.

Goal: Beacon blocks and states survive restart and finalization atomically.

Scope: bounded milestone. Depends on v0.337.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Hot blocks/states/sidecars, finalized/cold storage, canonical indexes, finalized migration, root/slot lookup, and atomic fork-choice coupling.

Verification:

- Long-chain/reorg/finality/crash simulations and corruption fixtures.

Exit criteria:

- Beacon blocks and states survive restart and finalization atomically.
- `v0.338.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.339.0 - State Snapshots And Reconstruction

Status: planned; internal signed tag, publication at v0.340.0.

Goal: Required historical states can be reconstructed within documented resource bounds.

Scope: bounded milestone. Depends on v0.338.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Incremental snapshots, state diffs, epoch-boundary checkpoints, historical-root/summary access, replay reconstruction, and cache eviction.

Verification:

- Random-state reconstruction, snapshot corruption, memory/disk benchmark tests.

Exit criteria:

- Required historical states can be reconstructed within documented resource bounds.
- `v0.339.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.340.0 - Sidecar Custody Pruning And Retention

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Data availability obligations persist correctly across restarts and pruning.

Scope: bounded milestone. Depends on v0.339.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Blob/data-column storage, custody-group history, backfill markers, hot/cold retention, pruning, archive policy, and availability provenance.

Verification:

- Fulu retention/custody simulations, prune/reorg/restart tests.

Exit criteria:

- Data availability obligations persist correctly across restarts and pruning.
- `v0.340.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.341.0 - Beacon Database Migration And Repair

Status: planned; internal signed tag, publication at v0.345.0.

Goal: Beacon storage upgrades and repairs are reproducible and fail closed.

Scope: bounded milestone. Depends on v0.340.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Versioned schemas, online/offline migrations, checkpoint/genesis imports, consistency scanning, repair plans, backup/restore, and operator-safe tooling contracts.

Verification:

- Multi-version migration fixtures, corruption recovery drills, checksum verification.

Exit criteria:

- Beacon storage upgrades and repairs are reproducible and fail closed.
- `v0.341.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 22: Consensus Networking And Synchronization

Consensus networking is separate from execution-layer DevP2P/RLPx. It uses the
transport and protocols required by the pinned consensus P2P specification and
must remain behind explicit optional features.

### v0.342.0 - Consensus Networking Threat And Dependency Gate

Status: planned; internal signed tag, publication at v0.345.0.

Goal: No live consensus networking lands before its dependencies and abuse controls are approved.

Scope: bounded milestone. Depends on v0.341.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- libp2p/discv5/crypto/runtime dependency review, identity/key separation,
  protocol limits, eclipse/amplification model, and network resource budgets;
- first-party ownership rule for Ethereum topic derivation, SSZ framing,
  gossip validation, ReqResp semantics, peer scoring, fork/custody
  compatibility, and subnet policy;
- generic transport/runtime adapters may be optional dependencies, but cannot
  own consensus validity or Ethereum-specific protocol state.

Verification:

- Dependency audit, threat-model review, transport prototype load tests.

Exit criteria:

- No live consensus networking lands before its dependencies and abuse controls are approved.
- `v0.342.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.343.0 - Discv5 ENR And Secure Transport

Status: planned; internal signed tag, publication at v0.345.0.

Goal: Consensus peers can be discovered and authenticated with current fork/custody metadata.

Scope: bounded milestone. Depends on v0.342.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Discovery v5, ENR fork/custody fields, identity persistence, secure
  multiplexed transport, fork compatibility, and address policy;
- first-party protocol codecs and state machines around reviewed optional
  socket, Noise/QUIC, multiplexing, and cryptographic adapters;
- bootnode, static-peer, trusted-peer, node-key, listen-address, NAT, and
  advertised-address policy.

Verification:

- Official/reference vectors, cross-client discovery/handshake tests, packet fuzzing.

Exit criteria:

- Consensus peers can be discovered and authenticated with current fork/custody metadata.
- `v0.343.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.344.0 - Fork-Aware Gossip Topics And Mesh

Status: planned; internal signed tag, publication at v0.345.0.

Goal: separate topic lifecycle from object-validation policy.

Scope: implementation pass. Depends on v0.343.0. The retained
workstream contract at v0.345.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement topic/subnet derivation, join/leave rotation and bounded mesh behavior for admitted fork message families.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Boundary slots/forks, FOCIL/ePBS topics, malformed SSZ-Snappy and churn simulations.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Topic participation never grants validity to received objects.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.344.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.345.0 - GossipSub Topics And Subnet Management Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: The node joins and leaves every required gossip domain at the correct time.

Scope: completion and integration pass. Depends on v0.344.0.
The implementation passes v0.344.0 through v0.344.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
message-family SSZ-Snappy admission and fork/topic validation integration.

Deliverables:

- Fork-digest topics, beacon blocks, aggregate/attestation subnets, sync
  committees, data columns, operation topics, subscription rotation, and mesh
  policy;
- fork-digest/topic-specific `WireLimits<Gossip, ForkDigest>` contexts kept
  separate from consensus-object validation contexts and local mesh policy;
- GossipSub payload and SSZ-Snappy decoding consume compressed/decompressed,
  ratio, structural, signature, allocation, and output budgets from
  `v0.88.0` before promotion.

Verification:

- Cross-client topic/subnet tests, fork-boundary simulations, bounded
  subscription tests, decompression bombs, and structural complexity oracles.

Exit criteria:

- The node joins and leaves every required gossip domain at the correct time.
- `v0.345.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.346.0 - Staged Gossip Validation And Seen Caches

Status: planned; internal signed tag, publication at v0.350.0.

Goal: Gossip reaches pools or fork choice only after all required validation stages pass.

Scope: bounded milestone. Depends on v0.345.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Decode/signature/state/fork-choice validation stages, dependency deferral,
  duplicate suppression, seen caches, invalid-message penalties, and no partial
  promotion;
- object-invalid GossipSub `REJECT` and object-negative caches require
  `v0.93.0` `ObjectInvalidityEvidence`; peer penalties additionally admit
  peer-protocol or peer-policy evidence bound to the peer and observation
  window, never as proof that the gossip object is invalid;
- one duplicate remains `Duplicate`/ignore, while repeated announced-quota
  abuse may create peer-policy evidence without a bad-object cache entry;
- missing dependencies, stale time, local budgets, cancellation, or backend
  faults remain ignore/defer/retry outcomes;
- seen/deferred caches follow the global chain/fork/root/object/validation-level
  identity invariant and never mix untrusted with verified entries;
- invalid-message and peer evidence consumes bounded `v0.93.0` entry,
  observation, witness, serialization, and retention budgets before cache or
  scoring mutation;
- every authoritative object-validation stage reserves minimal invalidity
  evidence before execution; seen-cache, negative-cache, scoring, logging, or
  persistence failure cannot erase an immediate object-invalid result;
- nested gossip decode/signature/state/fork-choice checks share one
  `v0.100.0` parent-authorized reservation tree; a child cannot mint capacity,
  and only filled child evidence can enter caches or peer attribution;
- authoritative gossip validation uses `v0.102.0` `FirstInvalid`; bounded
  operational diagnostics and member isolation cannot change accept/reject/
  ignore validity, and scoring/cache/logging sinks only borrow evidence;
- worker arenas follow `v0.106.0`: capacity derives from authorized concurrent
  validation and mode, queue pressure triggers bounded backpressure/local
  outcomes, and generation-safe handles prevent cancellation/transfer reuse.

Verification:

- Official gossip validation functions, malformed/future/dependency fuzzing,
  cache-pressure tests, object/peer evidence non-interchangeability tests, and
  oversized-evidence/observation-flood fault injection;
- post-invalid cache/scoring/logging/persistence failure tests preserving the
  immediate validation result while suppressing only failed side effects;
- cancellation and concurrent staged-validation tests proving child slots are
  returned or released exactly once and parent accounting is conserved;
- cross-mode GossipSub result invariance, beyond-limit non-attribution, sink-
  failure, and final-slot-ownership tests;
- valid gossip benchmarks across worker counts, topics, and scheduling seeds
  against the `v0.104.0` evidence-disabled baseline, proving no optional sink,
  allocation, parent/context clone, global mutex, or globally contended per-
  child atomic operation on the common path;
- arena-capability pressure, attacker message-count independence, stale-handle/
  ABA, live-borrow cancellation/transfer, and optional-pool-absent tests.

Exit criteria:

- Gossip reaches pools or fork choice only after all required validation stages pass.
- `v0.346.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.347.0 - Consensus Req Resp Protocols

Status: planned; internal signed tag, publication at v0.350.0.

Goal: Required sync and serving protocols are complete and bounded.

Scope: bounded milestone. Depends on v0.346.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Status, metadata, blocks, blobs, data columns, light-client data,
  states/checkpoints where admitted, chunk framing, context bytes, error
  responses, and cancellation;
- negotiated `WireLimits<ReqResp, Version>` profiles separated from consensus
  object validity and local serving willingness;
- each chunk consumes compressed/decompressed bytes, ratio, framing,
  structural, allocation, hash, and output work from `v0.88.0` before
  decompression or ownership conversion.

Verification:

- Cross-client protocol matrix, per-version wire boundaries, malformed chunk
  fuzzing, decompression-bomb and complexity-oracle cases, unavailable-data
  cases, and wire-violation/object-invalidity separation tests.

Exit criteria:

- Required sync and serving protocols are complete and bounded.
- `v0.347.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.348.0 - Consensus Peer Scoring And Backpressure

Status: planned; internal signed tag, publication at v0.350.0.

Goal: Malicious or slow peers cannot create unbounded work or dominate peer selection.

Scope: bounded milestone. Depends on v0.347.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Peer reputation, topic scores, custody-response scoring, bans, diversity,
  request budgets, rate limits, fair queues, clock disparity through
  `v0.89.0` evidence, and eclipse defenses;
- monotonic peer windows and in-process sanctions, boot/session-bound persisted
  observations, and rollback-safe UTC expiry that cannot extend bans or revive
  expired evidence;
- all score changes and sanctions consume object evidence composed with an
  authenticated delivery observation, peer-protocol evidence, or peer-policy
  evidence from `v0.93.0`; duplicate or policy outcomes alone cannot
  manufacture peer evidence.

Verification:

- Byzantine peer/flood/partition simulations, resource-ceiling benchmarks,
  evidence expiry/policy-version tests, and object/peer evidence substitution
  failures;
- restart/session-change, backward/stale-clock, ban-extension, and expired-
  evidence replay simulations.

Exit criteria:

- Malicious or slow peers cannot create unbounded work or dominate peer selection.
- `v0.348.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.349.0 - Checkpoint And Weak-Subjectivity Sync

Status: planned; internal signed tag, publication at v0.350.0.

Goal: Checkpoint sync either reaches the required anchor or terminates as a critical safety failure.

Scope: bounded milestone. Depends on v0.348.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Trusted checkpoint/state acquisition, checkpoint-root enforcement, stale-checkpoint checks, fatal mismatch policy, bootstrap persistence, and provider quorum.

Verification:

- Official weak-subjectivity cases, malicious source tests, restart recovery.

Exit criteria:

- Checkpoint sync either reaches the required anchor or terminates as a critical safety failure.
- `v0.349.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.350.0 - Head And Range Sync

Status: planned; public crates.io checkpoint after cumulative review.

Goal: A node reaches current head under bounded resources and adversarial peers.

Scope: bounded milestone. Depends on v0.349.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Peer selection, finalized/head range download, block/sidecar validation pipeline, target updates, progress persistence, and failover.

Verification:

- Multi-peer local networks, missing/reordered/invalid range tests, restart tests.

Exit criteria:

- A node reaches current head under bounded resources and adversarial peers.
- `v0.350.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.351.0 - Finalized Backfill And State Reconstruction

Status: planned; internal signed tag, publication at v0.355.0.

Goal: Historical data and states are reconstructed without weakening checkpoint trust.

Scope: bounded milestone. Depends on v0.350.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Historical block/sidecar backfill, state reconstruction, checkpoint gaps, archive/history provider use, and consistency proofs.

Verification:

- Long-range backfill, pruned-provider, corruption, and interruption tests.

Exit criteria:

- Historical data and states are reconstructed without weakening checkpoint trust.
- `v0.351.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.352.0 - Optimistic Sync And Execution Recovery

Status: planned; internal signed tag, publication at v0.355.0.

Goal: Optimistic progress cannot authorize duties and recovers correctly when execution rejects payloads.

Scope: bounded milestone. Depends on v0.351.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Optimistic import policy, execution validation queues, `latestValidHash` invalidation, poisoned-fork recovery, reorgs, and validator-duty safety flags.

Verification:

- Official optimistic-sync scenarios and multi-execution-client fault injection.

Exit criteria:

- Optimistic progress cannot authorize duties and recovers correctly when execution rejects payloads.
- `v0.352.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.353.0 - PeerDAS Custody And Sampling

Status: planned; internal signed tag, publication at v0.355.0.

Goal: derive availability obligations before storage/backfill orchestration.

Scope: implementation pass. Depends on v0.352.0. The retained
workstream contract at v0.354.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement custody history, sampling schedules and admitted cell-level deltas with parent resource budgets.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Custody transitions, malformed deltas, unavailable columns and restart identity tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Local readiness reflects fulfilled custody rather than merely downloaded bytes.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.353.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.354.0 - PeerDAS Sync Custody And Backfill Completion

Status: planned; internal signed tag, publication at v0.355.0.

Goal: Node and attached-validator custody obligations are met before availability-dependent acceptance or duties.

Scope: completion and integration pass. Depends on v0.353.0.
The implementation passes v0.353.0 through v0.353.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
custody-bound sync/backfill/serving persistence over admitted cell sampling.

Deliverables:

- Custody-group calculation/history, data-column sampling, column sync/backfill, serving obligations, malicious proof handling, supernode policy, and progress persistence.

Verification:

- Official Fulu/current DA fixtures, custody changes, unavailable-column and restart tests.

Exit criteria:

- Node and attached-validator custody obligations are met before availability-dependent acceptance or duties.
- `v0.354.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 23: Engine Coordination Data Availability And Beacon Service

### v0.355.0 - Beacon Engine Coordinator

Status: planned; public crates.io checkpoint after cumulative review.

Goal: build on the `v0.239.0` authenticated protocol/transport boundary and
own beacon-node fork-choice, payload-building, and execution-status
coordination policy.

Scope: bounded milestone. Depends on v0.354.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Reuse the authenticated Engine transport from `v0.239.0`;
- capability negotiation;
- all supported `newPayload`, `forkchoiceUpdated`, and `getPayload` versions;
- payload-attribute construction;
- beacon fork-choice to Engine sequencing;
- timeout, retry, cancellation, and execution-status state machines;
- durable request/idempotency records, exact `latestValidHash` invalidation,
  split-brain detection, and multi-EL reconciliation policy;
- explicit evidence passed to block import and production services;
- the in-process execution adapter must pass the exact same semantic contract
  suite as the authenticated JSON-RPC adapter.

Verification:

- Execution-apis fixtures;
- at least two independent execution-client integrations;
- authentication, timeout, invalid-payload, and sequencing tests;
- checks proving transport concerns remain in `v0.239.0`;
- adapter-equivalence, duplicate/reorder, restart, split-brain, and durable
  idempotency tests.

Exit criteria:

- The beacon node can coordinate every claimed fork with an execution client
  through the previously admitted authenticated boundary without duplicating
  transport or JWT ownership.
- `v0.355.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.356.0 - Multi-Execution-Client Failover

Status: planned; internal signed tag, publication at v0.360.0.

Goal: Execution failover is explicit and cannot silently mix incompatible payload state.

Scope: bounded milestone. Depends on v0.355.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Multiple endpoints, health/latency metrics, method/version capabilities, failover policy, disagreement evidence, circuit breaking, retry bounds, and recovery.

Verification:

- Independent execution-client outage/disagreement/latency simulations.

Exit criteria:

- Execution failover is explicit and cannot silently mix incompatible payload state.
- `v0.356.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.357.0 - Deposit Contract Tracking And Deposit Tree

Status: planned; internal signed tag, publication at v0.360.0.

Goal: maintain a reorg-safe execution-layer deposit view and canonical deposit
tree for historical beacon operation.

Scope: bounded milestone. Depends on v0.356.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Deposit-contract log acquisition through the reviewed provider boundary;
- deposit event validation and deduplication;
- incremental deposit tree and cache maintenance;
- execution-layer block/hash checkpoints;
- reorg rollback and replay;
- finalized deposit snapshots;
- bounded historical backfill and corruption recovery.

Verification:

- Official deposit-contract and deposit-tree vectors;
- execution reorg, duplicate log, missing range, and restart simulations;
- differential roots against independent consensus clients;
- bounded backfill and cache-pressure tests.

Exit criteria:

- The beacon service can derive a canonical, restart-safe deposit tree from
  execution history without trusting unordered or reorged logs.
- `v0.357.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.358.0 - Genesis Construction Eth1 Voting And Genesis Sync

Status: planned; internal signed tag, publication at v0.360.0.

Goal: support historical `eth1_data` behavior and construct a beacon genesis
state from verified deposits for public, private, and test networks.

Scope: bounded milestone. Depends on v0.357.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Historical `eth1_data` voting and deposit inclusion;
- deposit-count and deposit-root selection;
- genesis-validator activation rules;
- genesis-state construction from the verified deposit tree;
- minimum-genesis-time and validator-count policy;
- genesis synchronization mode;
- chain/genesis identity persistence and mismatch refusal.

Verification:

- Official phase0 genesis and deposit-processing vectors;
- historical eth1-voting fixtures;
- private/test-network genesis workflows;
- execution reorgs before and after genesis;
- cross-client genesis-root comparison.

Exit criteria:

- A beacon node can follow historical deposit voting or build and synchronize
  a new network genesis without external consensus core logic.
- `v0.358.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.359.0 - Availability Tracking And Block Admission

Status: planned; internal signed tag, publication at v0.360.0.

Goal: A block becomes fully available only from sufficient verified evidence under the active fork rules.

Scope: bounded milestone. Depends on v0.358.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Per-block availability states, custody/sample evidence, gossip/ReqResp integration, acceptance gates, retention, invalidation, and recovery.

Verification:

- Partition, missing-column, malicious-proof, reorg, and restart simulations.

Exit criteria:

- A block becomes fully available only from sufficient verified evidence under the active fork rules.
- `v0.359.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.360.0 - Beacon Node Orchestration

Status: planned; public crates.io checkpoint after cumulative review.

Goal: The focused crates operate as one coherent beacon node with explicit terminal states.

Scope: bounded milestone. Depends on v0.359.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Slot clock, import pipeline, transition/fork-choice/storage/network/Engine/DA coordination, operation pools, bounded task supervision, shutdown, and restart.

Verification:

- Deterministic in-process scenarios, fault injection at every subsystem boundary.

Exit criteria:

- The focused crates operate as one coherent beacon node with explicit terminal states.
- `v0.360.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.361.0 - Beacon Read And Event Servers

Status: planned; internal signed tag, publication at v0.365.0.

Goal: serve versioned beacon data before validator-production endpoints.

Scope: implementation pass. Depends on v0.360.0. The retained
workstream contract at v0.362.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement JSON/SSZ reads, events, pagination and bounded authenticated server infrastructure.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Pinned Beacon API schemas, progressive responses, cancellation and slow-consumer tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Read/event handlers expose no signing or builder authority.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.361.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.362.0 - Beacon Node REST And Event APIs Completion

Status: planned; internal signed tag, publication at v0.365.0.

Goal: External tooling can operate the beacon node through complete versioned server APIs.

Scope: completion and integration pass. Depends on v0.361.0.
The implementation passes v0.361.0 through v0.361.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
bounded node/config/pool/debug administrative endpoints and read/event integration; validator production remains separately owned.

Deliverables:

- Versioned JSON/SSZ Beacon API server, events, node identity/peers,
  config/spec, blocks/states, pools, light-client, debug, authentication, TLS,
  and rate limits;
- JSON parsing rejects duplicate keys and both JSON/SSZ routes consume the
  `v0.88.0` structural, allocation, hash, and output budgets before work;
- server responses, event queues, and serialization are bounded by request and
  connection child resources.

Verification:

- Official Beacon API conformance, client compatibility, duplicate-key,
  structural-depth/node, malformed, allocation, output, and request-flood tests.

Exit criteria:

- External tooling can operate the beacon node through complete versioned server APIs.
- `v0.362.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.363.0 - Beacon Block Production Service

Status: planned; internal signed tag, publication at v0.365.0.

Goal: give the beacon node sole default ownership of unsigned block
construction while keeping the service embeddable behind an explicit trait.

Scope: bounded milestone. Depends on v0.362.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Reorg-safe parent and head selection;
- operation-pool selection and packing;
- RANDAO input handling without receiving private keys;
- Engine payload request and payload-status coordination;
- local execution-payload fallback;
- blob sidecar and data-column construction;
- deposit, withdrawal, request, slashing, attestation, and sync operation
  inclusion;
- fork-aware fee recipient, gas limit, graffiti, and deadline policy;
- unsigned local-block production API;
- blinded-block request and response types plus a fail-closed provider hook;
- no live Builder API or relay communication before `v0.381.0`.

Verification:

- Official block-production and operation-ordering vectors;
- local execution-client and PeerDAS integration;
- reorg, timeout, invalid-payload, pool-conflict, and deadline tests;
- checks proving no validator secret or signature enters this service;
- tests proving the blinded hook cannot contact a relay or fabricate a bid
  before a backend is admitted at `v0.381.0`.

Exit criteria:

- The beacon node can produce a complete unsigned local block for every
  claimed fork and exposes only a fail-closed blinded-production hook until
  `v0.381.0`, while signing authorization remains outside the service.
- `v0.363.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.364.0 - Validator API And Production Boundary

Status: planned; internal signed tag, publication at v0.365.0.

Goal: expose complete safety-aware validator APIs while preserving beacon-node
ownership of block construction and validator-client ownership of independent
checks, slashing authorization, signing, and publication.

Scope: bounded milestone. Depends on v0.363.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Duties, attestation data, aggregates, unsigned block and blinded-block
  requests, signed publication, sync contributions, proposer preparation, fee
  recipient, liveness, subscriptions, and optimistic/sync safety status;
- API evidence binding responses to head, fork, genesis, slot, and execution
  status;
- explicit separation from the `v0.363.0` production service.

Verification:

- Official Beacon Validator API compatibility;
- unsafe/optimistic-node refusal tests;
- stale-head and mismatched-context tests;
- ownership tests proving the validator client cannot request operation
  packing or direct Engine coordination.

Exit criteria:

- A validator client can obtain unsigned duty material and publish signed
  results through a complete safety-aware API without becoming the block
  production service.
- `v0.364.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 24: Slashing Protection And Validator Key Foundation

### v0.365.0 - Slashing Protection Model And Invariants

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Slashability decisions are a small first-party security kernel with explicit invariants.

Scope: bounded milestone. Depends on v0.364.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Double-proposal, double-vote, surround-vote, repeat-signing, low-watermark, chain/genesis isolation, and fail-closed decision types.

Verification:

- Exhaustive bounded properties, official slashing cases, Kani candidate proofs.

Exit criteria:

- Slashability decisions are a small first-party security kernel with explicit invariants.
- `v0.365.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.366.0 - Transactional Slashing Database

Status: planned; internal signed tag, publication at v0.370.0.

Goal: A signature cannot escape before its slashing record is durably committed.

Scope: bounded milestone. Depends on v0.365.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Record-before-signature-release transactions, durable writes, multiprocess locking, concurrency serialization, database-error refusal, backups, and recovery.

Verification:

- Process-kill, concurrent signer, lock loss, torn-write, and restore tests.

Exit criteria:

- A signature cannot escape before its slashing record is durably committed.
- `v0.366.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.367.0 - EIP-3076 Interchange And Safety Recovery

Status: planned; internal signed tag, publication at v0.370.0.

Goal: Validator histories move between clients without permitting previously slashable signatures.

Scope: bounded milestone. Depends on v0.366.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Version-5 import/export, conservative missing-root handling, merge/low-watermark rules, stopped-client requirement, chain binding, validation, and migration reports.

Verification:

- EIP-3076 schema/examples, cross-client interchange, gap/rollback attack tests.

Exit criteria:

- Validator histories move between clients without permitting previously slashable signatures.
- `v0.367.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.368.0 - Validator Key Foundation And Deposit Data

Status: planned; internal signed tag, publication at v0.370.0.

Goal: generate validator identities and deposit artifacts with strict
separation between signing keys and withdrawal authority.

Scope: bounded milestone. Depends on v0.367.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- EIP-2333 BLS key generation and child derivation;
- EIP-2334 validator derivation paths;
- cryptographically secure entropy requirements and deterministic test seams;
- validator signing-key and withdrawal-key role types;
- scheme-tagged opaque key IDs, BLS public keys/signatures, EIP-2335 keystores,
  and custody handles conforming to `v0.95.0`;
- withdrawal credentials for BLS and execution-address modes;
- offline withdrawal-key workflow;
- deposit message, deposit-data root, signature, and JSON artifact generation;
- independent deposit-data verification before export;
- explicit refusal to load withdrawal secrets into normal validator runtime.

Verification:

- Official EIP-2333 and EIP-2334 vectors;
- deposit CLI and launchpad-compatible fixture checks where officially
  applicable;
- wrong-path, weak-entropy, credential-substitution, and role-confusion tests;
- memory-sanitization and offline-workflow review.

Exit criteria:

- Validator signing keys, withdrawal authority, derivation paths, and deposit
  artifacts are first-party, verifiable, and cannot be silently substituted
  across roles.
- `v0.368.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.369.0 - Validator Signer And Local Keystores

Status: planned; internal signed tag, publication at v0.370.0.

Goal: isolate local signing behind final domain and slashing authorization,
using the key roles and derivation rules established at `v0.368.0`.

Scope: bounded milestone. Depends on v0.368.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Consensus-domain signing packages;
- implement only the BLS12-381 `ConsensusSigner` capability from `v0.95.0`;
- EIP-2335 keystore import/export and password policy;
- locked and sanitized key memory;
- final fork, genesis, domain, signing-root, and duty-context validation;
- mandatory transactional slashing check before signature release;
- signing audit log, refusal policy, and local key lifecycle;
- hard rejection of withdrawal keys in validator-signing slots;
- no transaction, EIP-712, personal-message, EIP-7702, secp256k1, or
  transport-identity request can reach this signer.

Verification:

- Official and independent signing/keystore vectors;
- memory-sanitization review;
- wrong-domain, wrong-genesis, withdrawal-key, slashing-DB failure, and
  audit-redaction tests;
- compile-fail execution/transport request and cross-scheme key/signature/
  keystore substitution tests.

Exit criteria:

- Local validator signing is isolated, domain-safe, coupled to durable
  slashing protection, and incapable of consuming withdrawal authority.
- `v0.369.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 25: Complete Validator Client Duties

### v0.370.0 - Validator Duty Scheduler And Safety State

Status: planned; public crates.io checkpoint after cumulative review.

Goal: No duty reaches signing unless timing, chain, quorum, and safety preconditions hold.

Scope: bounded milestone. Depends on v0.369.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Drift-aware slot clock built on `v0.89.0`, duty lookahead/cache, reorg
  refresh, multi-beacon-node quorum/failover, doppelganger detection, and
  optimistic/unsafe refusal; external time evidence cannot override slashing or
  duty-safety decisions.

Verification:

- Clock skew, reorg, conflicting-node, startup, and doppelganger simulations.

Exit criteria:

- No duty reaches signing unless timing, chain, quorum, and safety preconditions hold.
- `v0.370.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.371.0 - Proposer Duties Signing And Publication

Status: planned; internal signed tag, publication at v0.375.0.

Goal: let the validator client request, independently validate, authorize,
sign, and publish proposer duties without owning block construction.

Scope: bounded milestone. Depends on v0.370.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- RANDAO reveal signing;
- proposer preparation and configuration submission;
- unsigned local/blinded block requests from `v0.363.0`;
- independent slot, parent, fork, fee-recipient, gas-limit, execution-status,
  and data-availability context checks;
- transactional slashing authorization;
- beacon-block proposer signature production;
- construction and publication of blob or data-column sidecars carrying the
  corresponding `SignedBeaconBlockHeader` derived from that block signature,
  without a separate sidecar-signing operation;
- publication deadlines, retries, and duplicate prevention.

Verification:

- Official proposer behavior;
- malicious or stale beacon-node response tests;
- slashing-database failure and duplicate proposal tests;
- local and blinded publication deadline/failure tests;
- pinned Deneb and Fulu honest-validator vectors proving sidecars reuse the
  signed beacon-block header.

Exit criteria:

- The validator client can safely sign and publish complete proposer duties
  for claimed forks while parent selection, operation packing, Engine calls,
  and DA construction remain beacon-node responsibilities.
- `v0.371.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.372.0 - Attester And Aggregator Duties

Status: planned; internal signed tag, publication at v0.375.0.

Goal: Attestation and aggregation duties are complete and slash-safe.

Scope: bounded milestone. Depends on v0.371.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Committee assignments, attestation construction, timing, selection proofs,
  aggregation, publication, duplicate prevention, fork-aware signing domains,
  and mandatory transactional slashing authorization before every attestation
  signature.

Verification:

- Official validator vectors, timing/reorg/duplicate simulations.

Exit criteria:

- Attestation and aggregation duties are complete and slash-safe through the
  already admitted `v0.365.0` and `v0.366.0` kernel and database.
- `v0.372.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.373.0 - Sync Committee Duties

Status: planned; internal signed tag, publication at v0.375.0.

Goal: Sync-committee participation is complete and refuses unsafe chain views.

Scope: bounded milestone. Depends on v0.372.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Membership tracking, messages, selection proofs, contributions,
  aggregation/publication, subnet subscriptions, optimistic-node refusal, and
  durable duplicate-signing records before signature release.

Verification:

- Official sync-committee vectors and timing/fork-boundary tests.

Exit criteria:

- Sync-committee participation is complete and refuses unsafe chain views.
- `v0.373.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.374.0 - Validator Lifecycle Requests And Operations

Status: planned; internal signed tag, publication at v0.375.0.

Goal: Operators can manage validator lifecycle without bypassing signer or slashing policy.

Scope: bounded milestone. Depends on v0.373.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Voluntary exits, BLS-to-execution changes, consolidation/lifecycle requests,
  deposit-data import from `v0.368.0`, fee/graffiti config, key enable/disable,
  authorization checks, and audit records.

Verification:

- Official operation vectors, authorization/policy tests, local testnet workflows.

Exit criteria:

- Operators can manage validator lifecycle without bypassing signer or slashing policy.
- `v0.374.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 26: External And Distributed Validator Key Custody

### v0.375.0 - Transactional Keymanager Methods

Status: planned; public crates.io checkpoint after cumulative review.

Goal: make key import/delete operations safe before convenience administration.

Scope: implementation pass. Depends on v0.374.0. The retained
workstream contract at v0.376.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement official import/delete/list and remote registration with slashing-history, stopped/active and per-key builder policy.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- OpenAPI fixtures, partial import/delete failure, concurrent duties and secret-redacted logs.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Administrative success never leaves key and slashing state inconsistent.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.375.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.376.0 - Keymanager Operator API Completion

Status: planned; internal signed tag, publication at v0.380.0.

Goal: implement the operator-to-validator-client Keymanager trust direction
without conflating it with outbound remote signing or custody backends.

Scope: completion and integration pass. Depends on v0.375.0.
The implementation passes v0.375.0 through v0.375.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
authenticated Keymanager server/client integration, builder configuration and operational limits.

Deliverables:

- Official Keymanager REST server;
- optional typed administrative client;
- import, delete, list, status, and remote-key registration methods;
- authentication, TLS, rate limits, audit logs, and secret redaction;
- transactional import/delete semantics;
- slashing-history preconditions and stopped/active validator policy.

Verification:

- Official Keymanager API conformance;
- unauthorized, concurrent mutation, partial import, and active-key deletion
  tests;
- audit-redaction and rate-limit tests.

Exit criteria:

- Operators can manage validator-client key registrations through the official
  API without obtaining signing authority or bypassing slashing policy.
- `v0.376.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.377.0 - Remote Signer Protocol And Slashing Authority

Status: planned; internal signed tag, publication at v0.380.0.

Goal: define the validator-client-to-signing-service trust direction and make
the authoritative slashing database location explicit for every deployment.

Scope: bounded milestone. Depends on v0.376.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Runtime-neutral remote signer request/response protocol;
- complete signing context including chain, genesis, fork, domain, duty, and
  signing root;
- mutually authenticated transport, timeout, cancellation, and replay policy;
- deployment modes with either signer-authoritative or coordinated
  slashing protection;
- prohibition of multiple independent authoritative slashing databases;
- idempotency and audit evidence for repeated requests;
- fail-closed behavior when slashing authority cannot be proven.

Verification:

- Remote signer conformance mocks;
- replay, split-brain, stale-context, timeout, and partial-response tests;
- multiple-validator-client simulations against one signer;
- proof that no signature escapes before the authoritative record commits.

Exit criteria:

- Remote signing cannot create ambiguous slashing authority, duplicate
  authorization, or a path around final signer-domain validation.
- `v0.377.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.378.0 - HSM KMS And Hardware Custody Adapters

Status: planned; internal signed tag, publication at v0.380.0.

Goal: define the signer-to-custody-backend trust direction for non-exporting
validator keys.

Scope: bounded milestone. Depends on v0.377.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Capability-based HSM, KMS, and hardware signer traits;
- non-exporting key handles and attested key identity;
- backend health, timeout, cancellation, retry, and concurrency policy;
- final signing-context validation before backend invocation;
- backend-specific secret and scratch sanitization requirements;
- optional adapters only behind reviewed features.

Verification:

- HSM/KMS/hardware mocks and admitted-device integration tests;
- wrong-key-handle, stale-session, timeout, retry, and failover tests;
- feature/dependency audit;
- key-isolation and sanitization review.

Exit criteria:

- A custody backend can hold validator keys without becoming slashing policy,
  domain policy, or validator-client control logic.
- `v0.378.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.379.0 - Distributed Signing Intent Model

Status: planned; internal signed tag, publication at v0.380.0.

Goal: prove a single slashing authority before threshold backend integration.

Scope: implementation pass. Depends on v0.378.0. The retained
workstream contract at v0.380.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Model quorum/membership changes, partition handling and durable signing intent with replay-bound transcripts.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Model checking of split brain, equivocation, partial-signature reuse and lost commits.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- No threshold adapter is admitted while two conflicting intents can authorize signatures.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.379.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.380.0 - Threshold DVT And Distributed Slashing Coordination Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: support threshold and distributed validator signing without weakening
single-signature slashing invariants.

Scope: completion and integration pass. Depends on v0.379.0.
The implementation passes v0.379.0 through v0.379.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
one reviewed threshold/DVT adapter and protocol-specific conformance using the proven authority model.

Deliverables:

- Threshold/DVT signer adapter boundary;
- participant identity, quorum, transcript, and timeout policy;
- distributed slashing-intent coordination;
- one authoritative decision per duty across validator clients and shares;
- partial-signature replay and equivocation evidence;
- fail-closed partition and membership-change handling.

Verification:

- Threshold signer simulations;
- conflicting coordinator, partition, duplicate-share, stale-membership, and
  quorum-loss tests;
- property tests proving no aggregate can escape without one durable
  non-slashable authorization.

Exit criteria:

- Distributed signing preserves the same domain, duty, and record-before-
  release guarantees as the local signer.
- `v0.380.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 27: Builder And MEV Integration

### v0.381.0 - Builder API And Blinded Proposals

Status: planned; internal signed tag, publication at v0.385.0.

Goal: admit one reviewed Builder API backend owned by the beacon-node block
production service without giving the validator client direct relay access.

Scope: bounded milestone. Depends on v0.380.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Beacon-node-owned relay client integrated behind the `v0.363.0` production
  hook;
- validator registration and preference submission from validator client to
  beacon node;
- beacon-node bid requests, relay authentication, signature/value/header
  validation, blinded block construction, payload reveal, fork-versioned
  Builder API, and deadline policy;
- validator-client independent validation of slot, parent, proposer, fork,
  execution header, value policy, and signing context before signing;
- signed blinded-block publication through the beacon node;
- no direct validator-client bid or payload-reveal request path by default.

Verification:

- Official Builder API conformance;
- malformed bid/reveal and stale-parent tests;
- local relay integration;
- process-boundary tests proving relay credentials and communication remain in
  the beacon node;
- malicious beacon-node response tests at the validator client.

Exit criteria:

- A validator can use one reviewed relay through the beacon node without
  trusting the bid, reveal, or unsigned blinded block blindly and without the
  validator client directly contacting the relay.
- `v0.381.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.382.0 - Relay Multiplexing Local Fallback And PBS Evolution

Status: planned; internal signed tag, publication at v0.385.0.

Goal: External builders cannot prevent a safe local proposal when a viable local payload exists.

Scope: bounded milestone. Depends on v0.381.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Beacon-node-owned multiple-relay communication;
- validator-client relay preferences and minimum-value policy submitted to the
  beacon node;
- bid comparison/minimums;
- deadline-aware circuit breakers;
- withholding and invalid-reveal defenses;
- beacon-node local-builder fallback;
- validator-client independent blinded-block validation and signing;
- audit evidence and protocol-native PBS/ePBS adapter boundary;
- no default direct validator-client relay communication.

Verification:

- Relay outage/equivocation/withholding simulations;
- guaranteed beacon-node local fallback tests;
- validator refusal of malformed, stale, or policy-violating blinded blocks;
- process-boundary tests for relay credentials and traffic.

Exit criteria:

- External builders cannot prevent a safe local proposal when a viable local
  payload exists, and relay interaction remains a beacon-node production
  responsibility.
- `v0.382.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 28: Consensus Safety Operations And Executables

### v0.383.0 - Optional Slasher Service

Status: planned; internal signed tag, publication at v0.385.0.

Goal: detect slashable network messages and feed verified evidence into
operation pools without placing detection on the validator signing path.

Scope: bounded milestone. Depends on v0.382.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Optional proposer- and attester-slashing detector;
- bounded historical message indexes;
- double-proposal, double-vote, and surround-vote evidence construction;
- gossip and imported-block observation;
- duplicate suppression and evidence persistence;
- operation-pool submission and operator reporting;
- explicit separation from local signing slashing protection.

Verification:

- Official slashable-message cases;
- long-history, duplicate, reorg, restart, and adversarial-flood tests;
- evidence round trips through block-operation pools;
- resource-ceiling benchmarks.

Exit criteria:

- The node can optionally detect and publish valid slashing evidence without
  making network observation a prerequisite for safe local signing.
- `v0.383.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.384.0 - Consensus Connectivity And NAT Diagnostics

Status: planned; internal signed tag, publication at v0.385.0.

Goal: make peer reachability, NAT behavior, subnet participation, and eclipse
risk diagnosable without weakening network policy.

Scope: bounded milestone. Depends on v0.383.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- NAT and externally observed address diagnostics;
- UPnP/NAT-PMP or successor adapters only behind explicit reviewed features;
- inbound/outbound reachability checks;
- ENR and listening-address consistency checks;
- subnet and custody-connectivity diagnostics;
- peer diversity and eclipse-risk reports;
- operator-safe remediation guidance without automatic unsafe exposure.

Verification:

- Public/private/NATed network simulations;
- malformed discovery response and address-spoofing tests;
- feature/dependency review;
- diagnostics redaction and bounded-probe tests.

Exit criteria:

- Operators can distinguish local configuration, NAT, subnet, custody, and
  hostile-peer failures without disabling security controls.
- `v0.384.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.385.0 - Consensus Operations Monitoring And Analytics

Status: planned; public crates.io checkpoint after cumulative review.

Goal: operate and monitor beacon and validator services with stable schemas,
including validator performance and safety analytics.

Scope: bounded milestone. Depends on v0.384.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Versioned shared config and CLI schemas;
- structured logs and tracing;
- Prometheus metrics;
- health, readiness, and task diagnostics;
- authentication, TLS, and rate limits;
- safe shutdown and service supervision;
- validator inclusion distance, effectiveness, missed-duty, balance, reward,
  sync participation, proposal, and slashing-risk analytics;
- privacy/redaction policy for validator identifiers and endpoints;
- all evidence/log/trace rendering uses stable reason codes and bounded
  diagnostics while redacting peer addresses, credentials, transaction privacy
  data, and secret-adjacent fields required by `v0.93.0`.

Verification:

- Config compatibility;
- redaction, cardinality, maximum-diagnostic/output, and malformed-evidence
  rendering tests;
- overload, shutdown, restart, and observability tests;
- analytics differentials against beacon-state outcomes.

Exit criteria:

- Beacon and validator services can be operated, monitored, and performance-
  analyzed without hidden state, secret leakage, or unbounded metric labels.
- `v0.385.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.386.0 - Beacon Node Executable And Packaging

Status: planned; internal signed tag, publication at v0.390.0.

Goal: ship an explicit production beacon-node executable rather than only
orchestration crates.

Scope: bounded milestone. Depends on v0.385.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- `eth-beacon-node` binary;
- stable CLI/config schema and validation;
- documented data-directory layout and permissions;
- genesis, checkpoint, execution endpoint, network, pruning, and API startup
  workflows;
- signal handling, graceful shutdown, crash recovery, and stable exit codes;
- container image and system-service packaging;
- binary version, build, SBOM, and provenance reporting;
- upgrade and rollback commands.

Verification:

- Fresh-start, checkpoint-sync, restart, signal, crash, config-migration, and
  data-directory permission tests;
- container and system-service smoke tests on supported operating systems;
- binary upgrade and rollback drills.

Exit criteria:

- Operators can install, configure, run, stop, upgrade, roll back, and diagnose
  a production beacon-node binary through stable documented interfaces.
- `v0.386.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.387.0 - Validator Client Executable And Packaging

Status: planned; internal signed tag, publication at v0.390.0.

Goal: ship an explicit production validator-client executable with signer and
slashing safety enabled by construction.

Scope: bounded milestone. Depends on v0.386.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- `eth-validator-client` binary;
- stable CLI/config schema and validator data-directory layout;
- local, remote, HSM/KMS, and threshold signer selection;
- mandatory slashing-database configuration and chain/genesis binding;
- beacon-node quorum/failover configuration;
- signal handling, graceful duty drain, stable exit codes, and restart policy;
- container image and system-service packaging;
- key import/migration separated from routine startup;
- upgrade and rollback commands.

Verification:

- Startup refusal without valid slashing and signer configuration;
- signal, restart, failover, doppelganger, config-migration, and permission
  tests;
- container and system-service smoke tests;
- binary upgrade and rollback drills with no slashable signatures.

Exit criteria:

- Operators can run a production validator-client binary that cannot silently
  bypass signer, chain, slashing, or duty-safety policy.
- `v0.387.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.388.0 - Database Inspection Migration And Recovery Tools

Status: planned; internal signed tag, publication at v0.390.0.

Goal: Operators can diagnose and recover storage without ad hoc database mutation.

Scope: bounded milestone. Depends on v0.387.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Read-only inspection, consistency checks, migration commands, checkpoint/state import, backup/restore, pruning controls, repair plans, and dangerous-operation confirmations.

Verification:

- Corrupt/mixed-version database drills and operator workflow tests.

Exit criteria:

- Operators can diagnose and recover storage without ad hoc database mutation.
- `v0.388.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.389.0 - Deterministic Consensus Simulator

Status: planned; internal signed tag, publication at v0.390.0.

Goal: Consensus and validator regressions can be reproduced without an external testnet.

Scope: bounded milestone. Depends on v0.388.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- In-process beacon nodes, validators, execution engines, and reproducible
  clocks/networks with loss, duplication, reordering, partitions, latency,
  Byzantine peers, and clock skew;
- equivocation, invalid payloads, DA loss, builder failures, disk/restart
  events, and reproducible seeds.

Verification:

- Scenario snapshots, determinism checks, mutation testing, regression corpus.

Exit criteria:

- Consensus and validator regressions can be reproduced without an external testnet.
- `v0.389.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 29: Full Consensus Assurance And Product Baseline

### v0.390.0 - Production Acceptance Matrix And Quantitative Budgets

Status: planned; public crates.io checkpoint after cumulative review.

Goal: replace subjective production gates with a versioned, numeric acceptance
contract before interoperability, longevity, and performance claims run.

Scope: bounded milestone. Depends on v0.389.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- A committed production-acceptance policy naming required official Hive
  suites with no "`or equivalent`" substitution;
- a named matrix of at least three independent consensus clients and three
  independent execution clients, unless an explicit reviewed ecosystem
  availability exception is recorded;
- a long-testnet floor of at least 30 continuous days, 4 beacon nodes, and
  4,096 active validators;
- zero locally generated slashable signatures and zero client-attributable
  missed proposals;
- no more than 0.1 percent client-attributable missed attestation or sync
  duties over the measured stable period;
- a deterministic event-attribution taxonomy covering `eth` beacon node,
  `eth` validator client, execution client, builder/relay, host/network,
  external clock source, planned fault injection, and unresolved/ambiguous
  causes;
- predeclared fault-injection windows, expected effects, recovery deadlines,
  and whether the window counts toward the continuous stable-period floor;
- rules that attribute an external execution-client or relay failure to the
  external component only when `eth` detects, contains, and recovers according
  to policy; failure to trigger required local fallback remains
  client-attributable;
- rules that distinguish independently observed host/network or clock-source
  outages from internal timeout, scheduling, skew-detection, or recovery
  failures;
- conservative treatment of ambiguous failures as client-attributable until a
  maintainer and an independent release reviewer approve a different
  classification from immutable evidence;
- all acceptance evidence and event ledgers obey explicit entry, witness,
  observation, diagnostic, serialization, output, and retention limits; links
  reference separately reserved artifacts instead of embedding unbounded data;
- required restart, database recovery, execution disagreement, reorg,
  clock-skew, network partition, DA loss, and builder-withholding scenarios;
- numeric mainnet-scale CPU, RAM, stack, disk-growth, disk-I/O, bandwidth,
  API-latency, duty-latency, and startup/recovery budgets on a reproducible
  reference hardware profile;
- numeric `v0.104.0` evidence-overhead, reservation/arena/context/code-size,
  allocation, contention, and retained-memory ceilings for execution,
  consensus, gossip, batch, and validator workloads, including a policy for
  evidence-disabled internal baseline measurements;
- `v0.109.0` measurement policy making uninstrumented absolute production
  thresholds authoritative, restricting full-disable comparisons to valid
  paths, and requiring invalid semantic projections/minimal-evidence baselines,
  untimed setup/result work, and separate non-perturbing instrumentation;
- an exception process requiring written security review and a replacement
  gate, never silent threshold reduction.

Verification:

- Machine-readable acceptance-policy schema and validator;
- machine-readable event ledger, attribution records, evidence links, reviewer
  identities, and immutable classification history with schema-enforced
  evidence budgets and redaction;
- scenario coverage audit;
- hardware-profile reproducibility check;
- evidence-enabled/evidence-disabled threshold reports across the required
  workload matrix;
- invalid semantic-projection/minimal-evidence and uninstrumented-production/
  instrumented-conformance report pairs with timing-boundary attestations;
- dry-run reports that fail on every deliberately violated threshold;
- adversarial reclassification tests proving planned faults, external
  failures, fallback failures, and ambiguous causes cannot be relabeled to
  bypass a gate;
- oversized artifact, observation-flood, diagnostic-redaction, and retention-
  expiry tests.

Exit criteria:

- Every remaining interoperability, longevity, performance, and release gate
  has a numeric pass/fail condition and an identified evidence artifact.
- `v0.390.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.391.0 - Hive And Multi-Consensus-Client Interoperability

Status: planned; internal signed tag, publication at v0.395.0.

Goal: The beacon node interoperates with the broader consensus-client ecosystem.

Scope: bounded milestone. Depends on v0.390.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Run every required Ethereum Hive consensus suite named by `v0.390.0`;
- consensus P2P, API, state-transition, sync, builder, and validator scenarios;
- compatibility with the full named independent consensus-client matrix;
- explicit issue ownership and waiver prohibition for unexplained failures.

Verification:

- Published Hive and interop reports;
- zero unexplained failures in claimed scope;
- no substitution of private or self-authored tests for a required Hive suite.

Exit criteria:

- The beacon node interoperates with the broader consensus-client ecosystem.
- `v0.391.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.392.0 - Multi-Execution-Client Interoperability

Status: planned; internal signed tag, publication at v0.395.0.

Goal: Beacon correctness is not coupled to one execution-client implementation.

Scope: bounded milestone. Depends on v0.391.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Full Engine workflows against the complete execution-client matrix fixed at
  `v0.390.0`, including payload invalidation, failover, disagreement, reorg,
  blobs/data columns, and restart recovery.

Verification:

- Long-running mixed-client scenarios;
- required restart, partition, reorg, disagreement, and latency cases;
- published failure and recovery reports.

Exit criteria:

- Beacon correctness is not coupled to one execution-client implementation.
- `v0.392.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.393.0 - Long-Running Validator Testnet

Status: planned; internal signed tag, publication at v0.395.0.

Goal: The complete beacon-node and validator stack demonstrates stable operation under realistic faults.

Scope: bounded milestone. Depends on v0.392.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- At least the `v0.390.0` minimum 30-day, 4-node, 4,096-validator sustained
  testnet with proposals, attestations, sync duties, reorgs, inactivity,
  restarts, key movement, builders, DA faults, and execution-client diversity.

Verification:

- Published duration, validator-count, client-matrix, load, and fault report;
- slashing database audit;
- finality and participation evidence;
- zero locally generated slashable signatures;
- zero client-attributable missed proposals;
- at most 0.1 percent client-attributable missed attestation or sync duties.

Exit criteria:

- The complete beacon-node and validator stack demonstrates stable operation under realistic faults.
- `v0.393.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.394.0 - Consensus Client Performance Gate

Status: planned; internal signed tag, publication at v0.395.0.

Goal: Mainnet-scale consensus workloads meet documented CPU, memory, disk, network, and timing budgets.

Scope: bounded milestone. Depends on v0.393.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Enforce the numeric `v0.390.0` budgets for SSZ roots, BLS batches,
  transition/epoch processing, fork choice, pools, storage, networking, sync,
  DA, validator duties, slashing DB, APIs, startup, and recovery;
- enforce evidence hot-path budgets across valid consensus objects, gossip
  workers, BLS/PeerDAS batches, `FirstInvalid`, diagnostic modes, stack/context/
  arena/code size, allocation, contention, and retained memory.

Verification:

- Reproducible reference hardware profile;
- mainnet-scale load tests;
- evidence-enabled/evidence-disabled comparisons with valid-path
  allocation/lock/atomic/clone/sink instrumentation;
- `v0.109.0` uninstrumented production runs, invalid semantic-projection/
  minimal-evidence comparisons, untimed setup/result work, and separate
  instrumentation conformance runs;
- threshold validator;
- regression alarms that fail the release.

Exit criteria:

- Mainnet-scale consensus workloads meet documented CPU, memory, disk, network, and timing budgets.
- `v0.394.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.395.0 - Kani State Transition And Fork-Choice Proofs

Status: planned; public crates.io checkpoint after cumulative review.

Goal: Selected consensus-state and fork-choice safety invariants have machine-checked evidence.

Scope: bounded milestone. Depends on v0.394.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded proofs for transition rollback, balance/churn arithmetic, checkpoint monotonicity, latest-message handling, invalidation, and transactional fork-choice handlers.

Verification:

- Pinned Kani runs with documented assumptions and bounds.

Exit criteria:

- Selected consensus-state and fork-choice safety invariants have machine-checked evidence.
- `v0.395.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.396.0 - Kani Slashing And Duty-Safety Proofs

Status: planned; internal signed tag, publication at v0.400.0.

Goal: Selected validator and slashing invariants have machine-checked evidence.

Scope: bounded milestone. Depends on v0.395.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Bounded proofs for double proposal/vote, surround vote, record-before-release state machines, optimistic-node refusal, duty uniqueness, and signer-domain binding.

Verification:

- Pinned Kani runs and cross-checks against property suites.

Exit criteria:

- Selected validator and slashing invariants have machine-checked evidence.
- `v0.396.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.397.0 - SSZ BLS PeerDAS And Acceleration Audit

Status: planned; internal signed tag, publication at v0.400.0.

Goal: independently audit the cryptographic and authenticated-data
implementations introduced after the earlier core audit.

Scope: bounded milestone. Depends on v0.396.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent audit of SSZ encoding/decoding;
- tree mutation, cached roots, generalized indices, branches, and multiproofs;
- BLS key generation, signing, aggregation, aggregate verification, randomized
  batch verification, subgroup handling, and timing behavior;
- cell KZG proof creation and verification;
- erasure coding and reconstruction;
- trusted-setup handling;
- every optional acceleration and cryptographic backend boundary;
- side-channel, fault, partial-output, and resource-exhaustion review.

Verification:

- Published report and complete finding register;
- implementation-level vector and differential reruns;
- timing and resource retests;
- clean independent remediation retest.

Exit criteria:

- No unresolved critical or high finding remains in first-party SSZ, BLS,
  PeerDAS cryptography, erasure coding, trusted setup, or acceleration
  boundaries.
- `v0.397.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.398.0 - State Transition And Fork-Choice Audit

Status: planned; internal signed tag, publication at v0.400.0.

Goal: No unresolved critical/high transition or fork-choice finding remains.

Scope: bounded milestone. Depends on v0.397.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent audit of fork upgrades, per-slot/epoch transition, deposit and
  genesis services, fork choice, optimistic execution, operation pools,
  Engine evidence consumption, and persistence, using the implementation audit
  from `v0.397.0` as a prerequisite.

Verification:

- Published report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high transition or fork-choice finding remains.
- `v0.398.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.399.0 - Consensus Network Sync And DA Audit

Status: planned; internal signed tag, publication at v0.400.0.

Goal: No unresolved critical/high network, sync, or data-availability finding remains.

Scope: bounded milestone. Depends on v0.398.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent audit of discv5/libp2p/GossipSub, NAT/connectivity adapters,
  validation, scoring, ReqResp, sync, weak subjectivity, PeerDAS consumers,
  custody, availability, slasher ingestion, and DoS controls;
- integration review proving every PeerDAS consumer validates through the
  `v0.315.0` core audited at `v0.397.0`.

Verification:

- Published report, adversarial retest, clean pentest.

Exit criteria:

- No unresolved critical/high network, sync, or data-availability finding remains.
- `v0.399.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.400.0 - Validator Slashing Keymanager And Builder Audit

Status: planned; public crates.io checkpoint after cumulative review.

Goal: No unresolved critical/high signing, slashing, key-custody, or builder finding remains.

Scope: bounded milestone. Depends on v0.399.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent audit of duties, timing, doppelganger detection, key generation,
  withdrawal-key separation, deposit data, slashing DB, EIP-3076, keystores,
  Keymanager API, remote signer slashing authority, HSM/KMS/hardware adapters,
  threshold/DVT coordination, relays, and local fallback.

Verification:

- Published report, slashing-safety retest, clean pentest.

Exit criteria:

- No unresolved critical/high signing, slashing, key-custody, or builder finding remains.
- `v0.400.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.401.0 - Beacon Storage API And Operations Audit

Status: planned; internal signed tag, publication at v0.405.0.

Goal: No unresolved critical/high storage, API, or operational finding remains.

Scope: bounded milestone. Depends on v0.400.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Independent audit of hot/cold storage, snapshots, migrations, repair,
  Beacon/Validator APIs, configuration, authentication, metrics, validator
  analytics, supervision, executable startup/shutdown, data directories,
  containers/system services, and binary upgrade/rollback behavior.

Verification:

- Published report, recovery/authorization retest, clean pentest.

Exit criteria:

- No unresolved critical/high storage, API, or operational finding remains.
- `v0.401.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.402.0 - Complete Consensus Remediation

Status: planned; internal signed tag, publication at v0.405.0.

Goal: The entire beacon-node and validator finding register is closed or explicitly accepted.

Scope: bounded milestone. Depends on v0.401.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Resolve all consensus-client findings, rerun official vectors, Hive, long-running testnet, performance, formal, and platform gates, and document accepted low risks.

Verification:

- All audit retests clean, zero unexplained conformance failures, updated SBOM/provenance.

Exit criteria:

- The entire beacon-node and validator finding register is closed or explicitly accepted.
- `v0.402.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.403.0 - Foundation And Consensus API Stability Baseline

Status: planned; internal signed tag, publication at v0.405.0.

Goal: record a reviewed stability baseline without pretending later
execution-node and integrated-node APIs are already frozen.

Scope: bounded milestone. Depends on v0.402.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Classify every publishable crate as a future `1.0` product, independently
  versioned support crate, optional backend/adapter, or internal crate;
- stabilize the foundation, beacon, validator, slashing, and builder contracts
  completed through `v0.402.0`;
- publish the APIs that remain intentionally open for `v0.407.0..=v0.441.0`;
- preserve independent support-crate versions and strict public dependency
  compatibility.

Verification:

- Workspace semver, feature, re-export, and public-dependency review;
- generated crate stability matrix;
- tests proving no support crate is forced to `1.0.0`.

Exit criteria:

- The completed foundation and consensus surfaces have a reviewable stability
  baseline, and every remaining pre-1.0 API area is named.
- `v0.403.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.404.0 - RC-Aware Release Tooling Foundation

Status: planned; internal signed tag, publication at v0.405.0.

Goal: make release tooling structurally understand prerelease candidates
before the final candidate cycle.

Scope: bounded milestone. Depends on v0.403.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Separate package version, candidate tag, report path, release-note path, and
  gate identifier in release metadata;
- parse SemVer and repeated `rc.N` identifiers structurally;
- support RC-specific pentest and release-note paths;
- retain independent support-crate versions and dependency order.

Verification:

- Parser and path tests for `rc.1`, `rc.2`, malformed identifiers, and
  package/tag mismatches;
- gate-selection and metadata-validation tests;
- ordinary `0.x` release regression tests.

Exit criteria:

- Release tooling can represent repeated RCs without confusing a package
  version with a candidate tag.
- `v0.404.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.405.0 - Exact Archive Publication Prototype

Status: planned; public crates.io checkpoint after cumulative review.

Goal: prove that approved crate archives can be preserved and published
without repackaging.

Scope: bounded milestone. Depends on v0.404.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Audited exact-archive uploader prototype;
- package name/version, checksum, manifest, commit, SBOM, and provenance
  binding;
- credential input through a non-file secret boundary with mandatory
  redaction;
- interrupted-upload recovery and dependency-order state.

Verification:

- Offline archive verification;
- mock registry upload, interruption, duplicate, and wrong-checksum tests;
- credential-redaction review.

Exit criteria:

- Exact approved `.crate` archives can be identified and submitted without
  invoking Cargo packaging again.
- `v0.405.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.406.0 - Consensus Product Baseline Gate

Status: planned; internal signed tag, publication at v0.410.0.

Goal: close the consensus-client expansion with complete evidence while
keeping the newly identified execution-product work open.

Scope: bounded milestone. Depends on v0.405.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Rerun consensus conformance, Hive, mixed execution-client, long-testnet,
  performance, formal, audit, remediation, platform, and packaging evidence;
- publish the foundation/consensus support and stability baseline;
- prove every remaining execution-client and integrated-node gap is assigned
  to `v0.407.0..=v0.449.0`;
- make no production-candidate or final API-freeze claim.

Verification:

- Exact-candidate pentest and retest for this milestone;
- green CI and CodeQL;
- traceability audit from the completeness register to later versions.

Exit criteria:

- Foundation, beacon-node, and validator-client work has a closed baseline,
  and 1.0 remains blocked on the explicit later product milestones.
- `v0.406.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 30: Full-Stack Core Cryptography Revalidation

The first-party Keccak-256, secp256k1, ECDSA, ECDH, and required symmetric
transport/keystore primitives are implemented and initially audited at
`v0.74.0..=v0.87.0`. This late phase revalidates those foundations after all
execution, wallet, networking, consensus, validator, and storage consumers
exist; it does not postpone their initial implementation.

### v0.407.0 - Core Cryptography Consumer Inventory

Status: planned; internal signed tag, publication at v0.410.0.

Goal: prove every production cryptographic consumer uses an admitted provider,
domain, cache identity, and failure contract.

Scope: bounded milestone. Depends on v0.406.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Inventory all hashing, signing, recovery, ECDH, KDF, MAC, cipher, KZG, BLS,
  entropy, transport, keystore, validator, and acceleration consumers;
- trace each consumer to its first-party or explicitly audited provider and
  domain-separated request type;
- identify temporary, reference-only, duplicate, or bypass paths for removal.

Verification:

- Feature-power-set dependency and call-path analysis;
- compile-fail tests for direct backend access;
- zero unexplained production consumers outside the inventory.

Exit criteria:

- The complete product has no hidden cryptographic implementation or provider
  bypass.
- `v0.407.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.408.0 - Core Crypto Side-Channel And Acceleration Revalidation

Status: planned; internal signed tag, publication at v0.410.0.

Goal: repeat side-channel, fault, zeroization, and acceleration review against
the final compiler, platforms, and production call patterns.

Scope: bounded milestone. Depends on v0.407.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Revalidate fixed-work/constant-time secret paths and public gas-to-cycles
  bounds across supported targets;
- audit hardware/vector acceleration, fallback equivalence, feature dispatch,
  fault handling, and backend state clearing;
- update the secret-taint inventory for all wallet, validator, networking, and
  key-custody paths.

Verification:

- dudect/ctgrind-style reports, assembly review, fault injection, and
  accelerated/scalar differential tests on named platforms.

Exit criteria:

- Final production builds preserve the admitted cryptographic security
  properties across every supported backend and target.
- `v0.408.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.409.0 - Cryptographic Cache Transcript And Batch Audit

Status: planned; internal signed tag, publication at v0.410.0.

Goal: audit cross-system domain separation, transcript soundness, batching,
and cache identity after every consumer is integrated.

Scope: bounded milestone. Depends on v0.408.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Review KZG/BLS batch coefficients, transcript domains, entropy failures,
  failure isolation, queue latency, and reuse policy;
- review signature/proof/hash caches for complete chain, fork, root, message,
  setup, key, and validation-level identity;
- test cross-protocol substitution among execution, consensus, validator,
  networking, wallet, and account-abstraction domains.

Verification:

- Adversarial transcript/collision/cache/batch corpus;
- cross-domain compile-fail and runtime substitution tests;
- independent review with zero unexplained cache or transcript equivalence.

Exit criteria:

- No final cryptographic batch or cache can accept evidence from an incomplete
  or different security context.
- `v0.409.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.410.0 - Core Cryptography Production Readmission

Status: planned; public crates.io checkpoint after cumulative review.

Goal: close all full-stack cryptographic findings and readmit the exact final
production paths before historical and executable product completion.

Scope: bounded milestone. Depends on v0.409.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Resolve the `v0.407.0..=v0.409.0` inventory, side-channel, acceleration,
  transcript, batching, cache, dependency, and domain findings;
- rerun all KAT, differential, fuzz, timing, fault, platform, feature, and
  consumer integration evidence;
- publish the full-stack core-cryptography conformance and audit report.

Verification:

- Independent cryptography audit, remediation, and clean retest;
- dependency graph proving no unapproved required implementation remains;
- zero critical/high findings and zero unexplained vector failures.

Exit criteria:

- The early first-party cryptographic core remains correct and secure in the
  complete production system.
- `v0.410.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 31: Historical Proof-Of-Work Execution

### v0.411.0 - Ethash Hashimoto Cache And Dataset

Status: planned; internal signed tag, publication at v0.415.0.

Goal: verify historical Ethereum proof-of-work seals first party.

Scope: bounded milestone. Depends on v0.410.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Ethash epoch, seed, cache, dataset-item, Hashimoto-light/full, mix digest,
  nonce, target, and seal verification;
- bounded cache/dataset generation and persistence policy;
- explicit endian, overflow, cancellation, and resource behavior;
- optional historical seal-production interface for controlled private tests.

Verification:

- Official historical Ethash vectors and known mainnet headers;
- light/full equivalence, corruption, boundary, and fuzz tests;
- memory, disk, CPU, and denial-of-service budgets.

Exit criteria:

- Historical proof-of-work headers can be cryptographically verified without
  another client.
- `v0.411.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.412.0 - Historical PoW Difficulty Ommers And Rewards

Status: planned; internal signed tag, publication at v0.415.0.

Goal: implement every pre-Merge block-consensus rule needed to validate
Ethereum history.

Scope: bounded milestone. Depends on v0.411.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Fork-specific difficulty and difficulty-bomb rules;
- ommer ancestry, age, uniqueness, seal, inclusion, and hash validation;
- miner/ommer rewards, DAO irregular transition, block rewards, and historical
  system-state changes;
- terminal-total-difficulty and Paris transition handling.

Verification:

- Difficulty, blockchain, DAO, reward, ommer, and Merge transition fixtures;
- mainnet checkpoint differentials and adversarial ancestry tests.

Exit criteria:

- Pre-Merge blocks are validated and applied with their historical consensus
  rules, not only EVM gas rules.
- `v0.412.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.413.0 - Genesis-To-Merge Historical Execution Gate

Status: planned; internal signed tag, publication at v0.415.0.

Goal: prove that the first-party execution stack can validate canonical
Ethereum history from genesis through the Merge.

Scope: bounded milestone. Depends on v0.412.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Historical fork-by-fork manifest and checkpoint corpus;
- genesis-to-Paris import path with Ethash, ommers, rewards, state roots, and
  total difficulty;
- archive/pruned verification modes with explicit evidence;
- unexplained divergence and skip prohibition.

Verification:

- Pinned historical execution-spec-tests and blockchain tests;
- differential checkpoint roots against independent clients;
- interrupted import, restart, reorg, and corruption tests.

Exit criteria:

- Historical execution claims cover the complete pre-Merge chain through the
  first post-Merge block.
- `v0.413.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 32: Production Execution Client Product

### v0.414.0 - Production Database Backend Admission

Status: planned; internal signed tag, publication at v0.415.0.

Goal: provide at least one reviewed durable backend capable of running a
mainnet execution node.

Scope: bounded milestone. Depends on v0.413.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Optional production database adapter implementing the `v0.219.0` contract;
- atomic batches, snapshots, iterators, checksums, corruption detection,
  backup/restore, migration, and read-only modes;
- version and capability negotiation so alternative backends remain possible;
- no database dependency in the default `no_std` facade graph.

Verification:

- Backend conformance, process-kill, torn-write, corruption, and migration
  suites;
- mainnet-scale key/value and compaction benchmarks;
- dependency and security review.

Exit criteria:

- Storage traits have a production backend with measured durability and
  recovery behavior.
- `v0.414.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.415.0 - Execution Stage Pipeline Unwind And State Healing

Status: planned; public crates.io checkpoint after cumulative review.

Goal: turn sync and storage components into a restartable production import
pipeline.

Scope: bounded milestone. Depends on v0.414.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Stages for headers, bodies, sender recovery, execution, hashes, trie roots,
  account/storage history, transaction lookup, pruning, and finish state;
- durable stage checkpoints, bounded commits, dependency ordering, unwind, and
  bad-block quarantine;
- snap pivot changes, state healing, missing-trie recovery, and static/history
  file production;
- online follow mode after initial sync.

Verification:

- Stage-by-stage execute/unwind properties;
- crash/restart, pivot movement, malicious peer, missing-node, and deep-reorg
  simulations;
- differential stage checkpoints against independent clients.

Exit criteria:

- Sync can progress, unwind, heal, restart, and enter live follow mode without
  ad hoc orchestration.
- `v0.415.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.416.0 - Local Execution Payload Builder

Status: planned; internal signed tag, publication at v0.420.0.

Goal: construct complete fork-valid execution payloads locally for the Engine
API.

Scope: bounded milestone. Depends on v0.415.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Reorg-safe parent/state selection and deterministic payload identifiers;
- txpool selection/replacement ordering for all transaction families;
- gas, blob, request, withdrawal, fee-recipient, parent-beacon-root, and
  deadline handling;
- incremental execution, receipts, roots, value calculation, cancellation,
  cache, and invalidation;
- embeddable builder trait separated from transport.

Verification:

- Engine payload fixtures and cross-client payload differentials;
- adversarial txpool, deadline, cancellation, reorg, and invalidation tests;
- value, root, and block-limit checks.

Exit criteria:

- The first-party execution client can build every locally supported payload
  required by a consensus client.
- `v0.416.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.417.0 - Authenticated Engine API Server

Status: planned; internal signed tag, publication at v0.420.0.

Goal: expose the complete execution-client side of the Engine API.

Scope: bounded milestone. Depends on v0.416.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Versioned `newPayload`, `forkchoiceUpdated`, `getPayload`, payload-body,
  blob, capability, and transition-configuration methods from pinned specs;
- JWT authentication, secret generation/loading, clock-skew policy, method
  allowlist, isolated listener, and redacted audit records;
- strict sequencing, payload status, `latestValidHash`, idempotency, timeout,
  cancellation, and restart behavior;
- integration with `v0.415.0` and `v0.416.0`.

Verification:

- Official Engine API fixtures and Hive engine suites;
- malformed JWT, replay, stale capability, sequence, invalidation, and restart
  tests;
- independent consensus-client smoke tests.

Exit criteria:

- A consensus client can drive the first-party execution client through the
  complete authenticated Engine API.
- `v0.417.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.418.0 - Execution HTTP RPC Request Server

Status: planned; internal signed tag, publication at v0.420.0.

Goal: implement ordinary execution request handling behind a secure listener.

Scope: implementation pass. Depends on v0.417.0. The retained
workstream contract at v0.420.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement pinned request-response eth methods, canonical state/proof/simulation/fee queries and HTTP admission; event methods are assigned to the following filter workstream.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Execution-api fixtures, vhost/CORS, batch/response work limits and Engine-listener isolation.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Request-response service is interoperable without claiming subscription support.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.418.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.419.0 - Execution WS And IPC Request Servers

Status: planned; internal signed tag, publication at v0.420.0.

Goal: add transport parity without duplicating method authority.

Scope: implementation pass. Depends on v0.418.0. The retained
workstream contract at v0.420.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement WS and native IPC request handling using the same dispatcher, peer/origin/ACL policy and bounded cancellation.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Cross-transport results, identity substitution, disconnects and backpressure tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Transport choice cannot bypass method or resource policy.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.419.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.420.0 - Public Execution JSON-RPC Server Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: serve the complete pinned standard execution API with production
security controls.

Scope: completion and integration pass. Depends on v0.419.0.
The implementation passes v0.418.0 through v0.419.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
request-server transport/security integration; filter and subscription service completion is explicitly in the following workstream.

Deliverables:

- Inbound HTTP, WebSocket, and IPC JSON-RPC servers;
- complete pinned `eth` method handlers over canonical state, chain, txpool,
  simulation, fee, proof, filter, and subscription services;
- namespace allowlists, CORS/origin/vhost policy, batch/concurrency/response
  limits, timeouts, gas/fee caps, and connection backpressure;
- explicit separation from the authenticated Engine listener.

Verification:

- Official execution-apis schema/tests and Hive `rpc-compat`;
- HTTP/WS/IPC interoperability and malicious-client fuzzing;
- permission, batch-bomb, subscription-flood, and resource tests.

Exit criteria:

- External users and tooling can use the node through a standard bounded
  execution JSON-RPC server.
- `v0.420.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.421.0 - Execution Filters And Subscription Server

Status: planned; internal signed tag, publication at v0.425.0.

Goal: implement reorg-correct event serving independently of diagnostics.

Scope: implementation pass. Depends on v0.420.0. The retained
workstream contract at v0.423.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement log/block/pending filters, retention/index binding and WS lifecycle with bounded queues.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Reorg/removal, pruning, reconnect, expired filter and slow-consumer tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Subscribers see explicit gaps and cannot retain unbounded server state.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.421.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.422.0 - Execution GraphQL And Diagnostic Server

Status: planned; internal signed tag, publication at v0.425.0.

Goal: admit optional query and diagnostic namespaces separately from public eth RPC.

Scope: implementation pass. Depends on v0.421.0. The retained
workstream contract at v0.423.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Implement pinned GraphQL and bounded net/web3/txpool/debug/trace/admin handlers behind explicit policies.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Schema/interoperability fixtures, deep query cost, namespace isolation and redaction tests.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Expensive or privileged methods require explicit bounded admission.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.422.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.423.0 - Filters Subscriptions GraphQL And Client APIs Completion

Status: planned; internal signed tag, publication at v0.425.0.

Goal: complete the indexed query and operational API surfaces expected from a
general-purpose execution client.

Scope: completion and integration pass. Depends on v0.422.0.
The implementation passes v0.421.0 through v0.422.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
filters, indexes, GraphQL and diagnostics namespace integration.

Deliverables:

- Reorg-correct log/block/pending filters and subscription lifecycle;
- bloom/log, transaction lookup, account/storage history, and trace indexes
  with pruning-aware errors;
- optional official GraphQL schema/server;
- bounded `net`, `web3`, `txpool`, `debug`, `trace`, and reviewed
  administrative methods;
- no unsafe legacy account-unlock or `personal` API by default.

Verification:

- execution-apis GraphQL validation, RPC compatibility, reorg, pruning, and
  subscription fixtures;
- index reconstruction and query differential tests;
- authorization and information-leak review.

Exit criteria:

- Standard queries, filters, subscriptions, GraphQL, and operator APIs are
  complete without weakening default node security.
- `v0.423.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.424.0 - Execution Discovery DNS And Peer Operations

Status: planned; internal signed tag, publication at v0.425.0.

Goal: turn first-party DevP2P protocols into an operable public-network peer
service.

Scope: bounded milestone. Depends on v0.423.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- EIP-1459 DNS trees, bootnodes, static/trusted peers, node-key persistence,
  listen/advertise addresses, NAT mapping, and discovery controls;
- current pinned `eth`/`snap` capability negotiation, fork ID, available block
  range, serving limits, and protocol downgrade policy;
- inbound/outbound quotas, diversity, required-block policy, peer events, and
  operator inspection;
- public-network and private-network profiles.

Verification:

- Cross-client discovery, RLPx, `eth`, and `snap` matrices;
- DNS tampering, eclipse, NAT, stale fork ID, range-lie, and flood tests;
- restart and node-identity persistence tests.

Exit criteria:

- The execution client can discover, connect, serve, and synchronize on public
  Ethereum networks with explicit peer policy.
- `v0.424.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.425.0 - Execution Node Executable And Packaging

Status: planned; public crates.io checkpoint after cumulative review.

Goal: ship a standalone production execution-node binary and reusable node
builder.

Scope: bounded milestone. Depends on v0.424.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- `eth-execution-node` binary and embeddable node builder;
- stable CLI/config schema, network presets, data-directory layout, JWT,
  database, pruning/archive, P2P, RPC, metrics, and payload settings;
- startup validation, file permissions, signals, exit codes, graceful
  shutdown, restart, service templates, and platform packages;
- default-secure listener and namespace policy.

Verification:

- Fresh mainnet/testnet/custom-chain startup;
- signal, permission, config migration, upgrade/rollback, and package tests;
- platform smoke matrix.

Exit criteria:

- Operators and downstream builders can run or embed a complete first-party
  execution client.
- `v0.425.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.426.0 - Execution Database Chain And Snapshot Tools

Status: planned; internal signed tag, publication at v0.430.0.

Goal: provide supported recovery and data-management tools instead of
requiring direct database mutation.

Scope: bounded milestone. Depends on v0.425.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Database inspect/verify/compact/migrate/repair commands;
- chain import/export, block/receipt/state dump, snapshot download/verify/
  import/export, stage inspection/unwind, and bad-block management;
- history/archive and EIP-4444-era data tooling with provenance;
- dangerous-operation confirmations and offline locking.

Verification:

- Multi-version migration, corrupt snapshot, interrupted import, wrong-chain,
  and rollback drills;
- read-only and destructive-command permission tests.

Exit criteria:

- Operators can inspect, move, recover, and verify execution data through
  supported tooling.
- `v0.426.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.427.0 - Execution Operations Security And Resource Controls

Status: planned; internal signed tag, publication at v0.430.0.

Goal: make the execution-node product observable and fail closed under
resource or configuration pressure.

Scope: bounded milestone. Depends on v0.426.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Structured logs/traces, Prometheus metrics, health/readiness, peer/sync/
  stage/txpool/payload diagnostics, and bounded labels;
- disk-space shutdown policy, cache and worker budgets, RPC/P2P rate limits,
  file-descriptor limits, secret redaction, and config validation;
- backup, incident, degraded-mode, and restart evidence;
- explicit unsafe-development flags separated from production profiles.

Verification:

- Disk-full, memory pressure, descriptor exhaustion, slow disk, network flood,
  RPC abuse, and secret-leak tests;
- monitoring schema and cardinality checks.

Exit criteria:

- The execution node is operable under defined resource budgets without
  silent data loss or unsafe fallback.
- `v0.427.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 33: Complete Execution Client Assurance

### v0.428.0 - Execution Hive And RPC Compatibility

Status: planned; internal signed tag, publication at v0.430.0.

Goal: prove the execution client against mandatory ecosystem conformance
suites rather than only internal fixtures.

Scope: bounded milestone. Depends on v0.427.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Run required Hive blockchain, engine, RPC compatibility, DevP2P, sync, and
  transaction suites;
- publish exact suite revisions, pass/fail/skip results, logs, and ownership;
- prohibit unexplained skips and self-authored substitutions.

Verification:

- Reproducible Hive runner and immutable report;
- zero unexplained failures for claimed scope.

Exit criteria:

- The first-party execution client passes the required public compatibility
  suites.
- `v0.428.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.429.0 - Execution Engine Multi-Consensus-Client Interoperability

Status: planned; internal signed tag, publication at v0.430.0.

Goal: prove that the first-party Engine server is not coupled to the
first-party beacon node.

Scope: bounded milestone. Depends on v0.428.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Full Engine workflows driven by the named independent consensus-client
  matrix;
- capabilities, payload building, invalidation, blobs/data columns, requests,
  restart, and fork transition scenarios;
- incompatibility ownership and remediation records.

Verification:

- Mixed-client devnets and long-running Engine sessions;
- malformed, delayed, duplicate, and reordered method tests.

Exit criteria:

- Independent consensus clients can safely drive the first-party execution
  client.
- `v0.429.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.430.0 - Mainnet And Testnet Sync Follow Gate

Status: planned; public crates.io checkpoint after cumulative review.

Goal: demonstrate reliable public-network synchronization and continuous
canonical following.

Scope: bounded milestone. Depends on v0.429.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Fresh full/snap sync on mainnet and supported public testnets;
- genesis/history, checkpoint/snapshot, pruning/archive, restart, peer churn,
  and reorg profiles;
- sustained live follow with root/head comparison against independent clients;
- published hardware, duration, bandwidth, disk, and failure evidence.

Verification:

- Reproducible public-network runbooks and immutable reports;
- canonical root/head sampling with zero unexplained divergence;
- interrupted sync and database recovery drills.

Exit criteria:

- The execution node can synchronize and remain canonical on supported public
  networks.
- `v0.430.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.431.0 - Execution Client Performance Gate

Status: planned; internal signed tag, publication at v0.435.0.

Goal: enforce mainnet-scale CPU, memory, disk, network, and API budgets for the
complete execution node.

Scope: bounded milestone. Depends on v0.430.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Numeric budgets for import, live execution, payload building, txpool, sync,
  state healing, database growth, pruning, RPC, tracing, and startup;
- reproducible reference hardware and regression thresholds;
- adversarial worst-case transaction, block, proof, peer, and request loads.

Verification:

- Automated benchmark and soak reports;
- release failure on budget regression without reviewed exception.

Exit criteria:

- The complete execution client meets documented production resource budgets.
- `v0.431.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.432.0 - Complete Execution Client Audit

Status: planned; internal signed tag, publication at v0.435.0.

Goal: independently review the runnable execution client as one security
boundary.

Scope: bounded milestone. Depends on v0.431.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Audit core cryptography, historical/current execution, storage backend,
  sync, txpool, DevP2P, payload builder, Engine server, RPC servers, tooling,
  executable, and operations controls;
- review first-party versus optional dependency ownership;
- publish findings, severity, evidence, and remediation assignments.

Verification:

- Independent report covering every execution product component;
- no omitted production feature or unreviewed critical trust boundary.

Exit criteria:

- Every execution-client finding is recorded with an owner and remediation
  version.
- `v0.432.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.433.0 - Complete Execution Client Remediation

Status: planned; internal signed tag, publication at v0.435.0.

Goal: close the execution-client audit and conformance register.

Scope: bounded milestone. Depends on v0.432.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Resolve all execution audit, Hive, interoperability, public-sync,
  performance, documentation, compatibility, and platform findings;
- rerun the complete execution evidence set;
- record accepted low risks with expiry and owner.

Verification:

- Clean independent retest;
- zero critical/high findings and zero unexplained conformance failures;
- updated SBOM, provenance, support matrix, and runbooks.

Exit criteria:

- The standalone execution client is a production candidate before integrated
  node work begins.
- `v0.433.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 34: Integrated Ethereum Node Product

### v0.434.0 - First-Party Integrated Node Orchestration

Status: planned; internal signed tag, publication at v0.435.0.

Goal: compose the first-party execution and beacon clients into one operable
Ethereum node without collapsing their security boundaries.

Scope: bounded milestone. Depends on v0.433.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- `eth-node` launcher and embeddable supervisor for execution plus beacon
  services;
- shared network selection, data-root policy, JWT provisioning, startup
  ordering, health propagation, shutdown, restart, and version compatibility;
- independent process and in-process deployment modes;
- validator client remains separately isolated by default.

Verification:

- Fresh sync, restart, partial-service failure, JWT rotation, and version
  mismatch tests;
- proof that Engine and public/admin API listeners remain separated.

Exit criteria:

- Operators can run a complete first-party Ethereum full node through one
  supported orchestration surface.
- `v0.434.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.435.0 - Private Devnet And Custom Network Tooling

Status: planned; public crates.io checkpoint after cumulative review.

Goal: create reproducible execution/consensus networks for testing and
downstream client development.

Scope: bounded milestone. Depends on v0.434.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Coupled execution genesis, beacon genesis, fork schedule, deposit contract,
  validator keys, bootnodes, JWT, and chain configuration generation;
- deterministic ephemeral devnets and persistent private networks;
- prefunded accounts, validator deposits, configurable fork activation, and
  safe test-only block timing;
- artifact manifests and cleanup.

Verification:

- Multi-node first-party devnets across historical/current fork boundaries;
- wrong-genesis, mismatched-fork, duplicate-key, and restart tests.

Exit criteria:

- Developers can reproduce a complete Ethereum network without hand-assembling
  incompatible EL/CL configuration.
- `v0.435.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.436.0 - Mixed First-Party And Independent Client Matrix

Status: planned; internal signed tag, publication at v0.440.0.

Goal: prove every first-party role can be replaced independently.

Scope: bounded milestone. Depends on v0.435.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Matrices for first-party EL with independent CLs, first-party CL with
  independent ELs, independent validators with first-party beacon nodes, and
  first-party validators with independent beacon nodes;
- Engine, P2P, Beacon API, Validator API, Builder API, Keymanager, and signer
  compatibility;
- fork-transition, restart, invalid-data, and degraded-service scenarios.

Verification:

- Published mixed-client testnet and Hive reports;
- zero unexplained role-coupling failures.

Exit criteria:

- Downstream users can adopt individual `eth` client components without
  requiring the entire first-party stack.
- `v0.436.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.437.0 - Long-Running Integrated Ethereum Testnet

Status: planned; internal signed tag, publication at v0.440.0.

Goal: demonstrate sustained operation of the complete first-party node and
validator stack.

Scope: bounded milestone. Depends on v0.436.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- At least 30 continuous days with multiple first-party and independent EL/CL
  nodes, at least 4,096 validators, transactions, contracts, blobs/data
  columns, builders, pruning, sync, and validator lifecycle operations;
- planned reorg, partition, clock, disk, restart, execution invalidation,
  builder withholding, DA loss, and key-movement faults;
- deterministic attribution through the `v0.390.0` taxonomy.

Verification:

- Immutable duration, participation, finality, head/root, resource, and fault
  reports;
- zero slashable signatures and zero unexplained consensus divergence.

Exit criteria:

- The integrated stack remains correct and recoverable under realistic
  sustained operation.
- `v0.437.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.438.0 - Integrated Node Performance And Failure Recovery

Status: planned; internal signed tag, publication at v0.440.0.

Goal: enforce whole-node resource and recovery budgets across execution,
consensus, and validator boundaries.

Scope: bounded milestone. Depends on v0.437.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Combined CPU, RAM, disk, I/O, bandwidth, startup, sync, API, payload, and
  duty-latency budgets;
- cascading-failure, backpressure, disk-full, database corruption, Engine
  outage, network partition, and clock fault scenarios;
- backup/restore and rolling component upgrade evidence.

Verification:

- Reproducible reference-hardware tests;
- recovery deadline and attribution checks;
- no hidden resource double-counting between services.

Exit criteria:

- The complete node meets numeric resource and recovery targets under
  cross-layer failures.
- `v0.438.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.439.0 - Operator Installation Upgrade And Incident Guides

Status: planned; internal signed tag, publication at v0.440.0.

Goal: make production deployment and recovery possible without source-code
archaeology.

Scope: bounded milestone. Depends on v0.438.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Install, configuration, firewall, JWT, backup, monitoring, pruning/archive,
  validator separation, builder, upgrade, rollback, migration, and incident
  guides;
- systemd/container/manual deployment profiles;
- diagnostics decision trees and support bundles with secret redaction;
- downstream embedding guides for execution, beacon, validator, and full-node
  builders.

Verification:

- Fresh-operator exercises on every supported desktop/server platform;
- disaster-recovery drills and documentation link/doctest checks.

Exit criteria:

- A new operator or client builder can deploy, upgrade, diagnose, and recover
  the supported products from maintained documentation.
- `v0.439.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.440.0 - Complete Full-Stack Security Audit

Status: planned; public crates.io checkpoint after cumulative review.

Goal: independently review the integrated Ethereum stack and every cross-layer
trust transition.

Scope: bounded milestone. Depends on v0.439.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Audit execution/beacon orchestration, Engine/JWT, shared configuration,
  databases, networking, APIs, validator isolation, builder fallback,
  monitoring, packaging, upgrades, and recovery;
- review mixed-version and partial-compromise behavior;
- publish complete findings and remediation assignments.

Verification:

- Independent report covering all production binaries and embedding modes;
- no unreviewed cross-layer trust or privilege path.

Exit criteria:

- Every full-stack finding is recorded before final production admission.
- `v0.440.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.441.0 - Complete Full-Stack Remediation

Status: planned; internal signed tag, publication at v0.445.0.

Goal: close all integrated-stack findings before final acceptance and API
freeze.

Scope: bounded milestone. Depends on v0.440.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Resolve full-stack audit, long-testnet, mixed-client, performance, recovery,
  packaging, documentation, and platform findings;
- rerun all affected component and integrated evidence;
- publish accepted low-risk register with owner and expiry.

Verification:

- Clean independent retest;
- zero critical/high findings and zero unexplained cross-layer failures;
- final pre-freeze SBOM and provenance.

Exit criteria:

- No known implementation or operational blocker remains before final
  production admission.
- `v0.441.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 35: Final Production Admission

### v0.442.0 - Independent Execution Client Embedding Gate

Status: planned; internal signed tag, publication at v0.445.0.

Goal: prove downstream users can build an execution client without repository internals.

Scope: implementation pass. Depends on v0.441.0. The retained
workstream contract at v0.445.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Build an out-of-workspace client from approved archives with caller-selected runtime, database, node configuration and payload service.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- No path patches/private imports, two backend/runtime adapter fixtures, restart/reorg and mixed-CL Engine scenarios.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- The public crate interfaces are sufficient to construct and operate a downstream execution client.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.442.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.443.0 - Independent Consensus And Validator Embedding Gate

Status: planned; internal signed tag, publication at v0.445.0.

Goal: prove beacon and validator construction are supported public-library use cases.

Scope: implementation pass. Depends on v0.442.0. The retained
workstream contract at v0.445.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Build separate downstream beacon and validator consumers with swappable Engine, network and signer adapters plus isolated slashing storage.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- Official API interoperability, duty refusal/record-before-release and process/in-process combinations from packaged crates.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Downstream clients do not need privileged repository access or a monolithic launcher.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.443.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.444.0 - SDK Light Client And Stateless Consumer Gate

Status: planned; internal signed tag, publication at v0.445.0.

Goal: prove non-node applications remain practical after full-client expansion.

Scope: implementation pass. Depends on v0.443.0. The retained
workstream contract at v0.445.0 applies from this first implementation;
its integration gate is not a prerequisite implementation. No later
feature is implicitly enabled by this pass.

Deliverables:

- Build packaged no_std verifier, alloc SDK, wallet/contract and stateless/light-client consumers with explicit trust and feature choices.
- Document public/private ownership, supported inputs/forks, failure
  behavior, feature/resource bounds and runnable examples for this slice.

Verification:

- MSRV/stable, feature isolation, mobile/WASM capability fixtures and independent proof/transaction workflows.
- Record executable commands and immutable fixtures/results; run the
  full local gate and exact-commit pentest, fix findings and retest.

Exit criteria:

- Full-client functionality does not force networking, storage or signing dependencies on library consumers.
- Evidence covers each listed behavior; no placeholder counts as
  implementation and unresolved failures block the tag.
- `v0.444.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.445.0 - Final Production Acceptance Matrix Completion

Status: planned; public crates.io checkpoint after cumulative review.

Goal: apply one quantitative acceptance contract to every production product
and embedding mode.

Scope: completion and integration pass. Depends on v0.444.0.
The implementation passes v0.442.0 through v0.444.0 own
the extracted implementations. The retained bullets below remain the
complete workstream acceptance contract, not permission to reimplement
those pieces. Remaining implementation in this pass is limited to:
quantitative final acceptance over completed external-consumer and product evidence.

Deliverables:

- Consolidate execution, beacon, validator, integrated node, SDK, provider,
  wallet, contract, light-client, stateless, and tooling gates;
- require current official conformance/Hive suites and named independent
  client matrices;
- define final CPU, memory, disk, network, latency, longevity, recovery,
  security, compatibility, platform, and documentation thresholds;
- preserve deterministic failure attribution and reviewed exceptions.

Verification:

- Machine-readable acceptance policy and evidence index;
- deliberately failing threshold and attribution tests;
- audit proving every 1.0 support claim has a gate.

Exit criteria:

- Every production claim has a numeric or otherwise objective release-blocking
  criterion.
- `v0.445.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.446.0 - Complete Public API And Crate Stability Freeze

Status: planned; internal signed tag, publication only after RC/1.0 admission.

Goal: freeze all public contracts only after every planned product exists.

Scope: bounded milestone. Depends on v0.445.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Freeze core, SDK, execution, provider, wallet, contract, storage, networking,
  light-client, beacon, validator, builder, binary, embedding, and operations
  APIs;
- finalize crate classifications and independent support-crate versions;
- require strict compatibility for public support-crate dependencies;
- publish migration, deprecation, feature, platform, and stability matrices.

Verification:

- Whole-workspace semver, feature-powerset, re-export, and public-dependency
  review;
- API snapshots and downstream compile matrix;
- proof that only approved products are promoted to `1.0.0`.

Exit criteria:

- No foundational or product API invention remains before 1.0.
- `v0.446.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.447.0 - Final Release Evidence Dry Run

Status: planned; internal signed tag, publication only after RC/1.0 admission.

Goal: rehearse the exact final release and operator upgrade path from a clean
environment.

Scope: bounded milestone. Depends on v0.446.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Ordered crate publication, exact archives, checksums, signatures, SBOM,
  provenance, platform packages, database/config migrations, and rollback;
- RC-specific reports, notes, gates, candidate/package separation, and
  repeated `rc.N`;
- executable and container artifact reproduction;
- exact approved archive preservation.

Verification:

- Full clean-room dry run;
- changed-candidate, wrong-version, wrong-checksum, rollback, and repeated-RC
  tests;
- offline archive and artifact verification.

Exit criteria:

- The complete 1.0 evidence and upgrade process has been exercised without
  changing an approved artifact.
- `v0.447.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.448.0 - Version-Only 1.0 Promotion Rehearsal

Status: planned; internal signed tag, publication only after RC/1.0 admission.

Goal: prove the final package-version promotion is isolated, reviewable, and
reproducible.

Scope: bounded milestone. Depends on v0.447.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Rehearse manifests, workspace dependencies, lockfile, version matrix, notes,
  SBOM, checksums, provenance, packages, binaries, and images for `1.0.0`;
- keep `release.version = "1.0.0"` separate from
  `candidate_tag = "v1.0.0-rc.1"`;
- promote only products approved at `v0.446.0`;
- preserve independent support-crate versions;
- use the audited exact-archive uploader from `v0.405.0`.

Verification:

- Semantic package/artifact diff;
- complete release-tool integration suite;
- proof that any implementation or unrelated metadata change rejects the
  promotion.

Exit criteria:

- The project can produce the exact `1.0.0` candidate through a constrained
  version-only promotion.
- `v0.448.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.449.0 - Production Candidate Admission Gate

Status: planned; internal signed tag, publication only after RC/1.0 admission.

Goal: freeze the final `0.x` implementation and authorize one exact
version-promotion commit.

Scope: bounded milestone. Depends on v0.448.0.
Only the deliverables below are admitted. Subsequent product/fork claims
require their own milestones; inherited security rules apply to this slice.

Deliverables:

- Run every final acceptance, conformance, Hive, mixed-client, public-sync,
  long-testnet, performance, formal, audit, platform, documentation,
  packaging, and release gate;
- publish final support, stability, migration, artifact, and checksum
  manifests;
- authorize only the rehearsed `v0.448.0` version-promotion operation;
- invalidate admission on any implementation or unrelated metadata change.

Verification:

- Exact implementation-candidate pentest and clean retest;
- green CI and CodeQL;
- reproducible packages, binaries, images, and exact archive checks;
- final `v0.445.0` acceptance-policy pass.

Exit criteria:

- The only permitted next change is the audited version-only promotion commit
  tagged as `v1.0.0-rc.1`.
- `v0.449.0 implementation stop reached. Run pentest for this exact
  commit.`

## v1.0.0-rc.1 - Exact Production Candidate

Status: planned release candidate.

Goal: create the actual `1.0.0`-versioned artifact once, test that exact commit,
and use it unchanged for the final stable tag.

Scope: exact-artifact promotion. Depends on v0.449.0.
No implementation changes are admitted. Reopen a numbered remediation pass
and repeat candidate admission if the approved implementation must change.

Deliverables:

- Apply only the version-promotion changes rehearsed at `v0.448.0`;
- set `eth` and only the deliberately stabilized public products to their
  approved `1.0.0` versions;
- preserve every other support crate's independently reviewed version;
- record package version `1.0.0` separately from candidate tag identifier
  `v1.0.0-rc.1`;
- regenerate lockfiles, crate-version matrices, SBOM, checksums, provenance,
  package archives, and release metadata;
- write RC-specific release notes and pentest evidence paths;
- run the RC-specific release gate;
- preserve the approved `.crate` archives in the release evidence store with
  checksums bound to the candidate commit;
- tag the exact promoted commit as `v1.0.0-rc.1`;
- publish no stable crate until final admission.

Verification:

- Full release gate on the promoted commit;
- exact-candidate pentest and clean retest;
- green CI and CodeQL on the promoted commit;
- reproducible package and checksum verification;
- exact archive offline verification and uploader dry run;
- independent support-crate version-policy verification;
- semantic package diff proving only approved version metadata differs from
  `v0.449.0`;
- repeat as `v1.0.0-rc.N` from a newly reviewed commit if any change is needed.

Exit criteria:

- The exact commit is approved for the final `v1.0.0` tag with no further
  source, manifest, lockfile, documentation, SBOM, or packaging changes.
- `v1.0.0-rc.1 implementation stop reached. Run pentest for this exact
  commit.`

## v1.0.0 - Complete Production Ethereum Toolkit

Status: planned.

Goal: publish the first serious production-ready release only after the complete
roadmap above has reached its exit criteria for every capability claimed by
the support matrix.

Scope: publication of the unchanged approved v1.0.0-rc.N candidate.
No new implementation or repackaging is permitted at the stable tag.

Deliverables:

- complete owned SDK, first-party execution, provider, wallet, contract,
  storage, canonical-chain, full beacon-node, validator-client,
  consensus-light-client, execution and consensus networking, sync,
  slashing-protection, builder, and stateless-support surfaces described above;
- first-party Keccak-256, secp256k1, ECDSA/recovery/ECDH, historical Ethash,
  pre-Merge consensus validation, and genesis-to-head execution support;
- a production execution-node binary with local payload building,
  authenticated Engine API server, public JSON-RPC/GraphQL, DevP2P/snap sync,
  database backend, recovery tools, and operational controls;
- historical and current fork support backed by pinned official conformance
  evidence;
- a production beacon node that coordinates with independent execution clients
  through the Engine API, plus a validator client that refuses unsafe duties;
- an integrated full-node launcher that preserves EL/CL/validator security
  boundaries while supporting independent component replacement;
- transactional slashing protection, EIP-3076 interchange, Keymanager API,
  local/remote/HSM signer boundaries, and safe builder fallback;
- explicit unsupported/future-fork behavior with no silent fallback;
- stable API, feature, MSRV, platform, and migration policy;
- signed release manifest, checksums, SBOM, provenance, audit references, and
  dependency/feature compatibility matrix;
- `eth` and every deliberately stabilized product at `1.0.0`, with support
  crates retaining their independently approved versions and compatibility
  bindings.

Verification:

- `scripts/checks.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/generate-sbom.sh --check`
- all official conformance, interoperability, fuzz, Kani, Miri, sanitizer,
  platform, performance, Hive, public-sync, long-running execution, validator,
  integrated-node, and live-node gates
  assigned above;
- `scripts/validate-release-readiness.sh v1.0.0`
- verify that `v1.0.0` points to the exact already approved
  `v1.0.0-rc.N` commit;
- publish the exact approved `.crate` archives through the audited archive
  uploader without invoking a repackaging path;
- do not rebuild, regenerate, or modify candidate artifacts;
- verify crates.io checksums against the approved candidate archive manifest;
- pentest and clean retest evidence for the exact candidate commit.

Exit criteria:

- no unresolved critical or high finding;
- no unexplained conformance skip for any claimed feature or fork;
- every public capability is implemented, tested, documented, and represented
  accurately in the support matrix;
- the final `v1.0.0` tag points to the unchanged approved
  `v1.0.0-rc.N` commit.
- `v1.0.0 implementation stop reached. Run pentest for this exact commit.`
