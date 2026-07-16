# eth Release Plan To 1.0

Status: planning document

This plan is intentionally granular. `eth` is security-sensitive Ethereum
protocol software, so each milestone must be small enough to review, test,
pentest, and stop cleanly before tagging.

The list below is not a maximum. Add patch releases or split a milestone before
implementation if the work no longer fits in one safe review pass.

Tags use:

```text
v0.N.0      milestone release
v0.N.P      patch/fix release for milestone N
v1.0.0-rc.N exact 1.0-versioned production candidate
v1.0.0      first serious production-ready Ethereum crate
```

## Release Principles

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

Every untagged release must remain a standalone implementation handoff with
these explicit sections:

- `Status`: whether the release is planned, in implementation, awaiting
  pentest, or ready to tag;
- `Goal`: the single outcome the release exists to achieve;
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
  local release gate before tagging or publishing;
- `sbom/eth.spdx.json` exists, is non-empty, and passes semantic drift
  comparison against a freshly generated document;
- the tag does not already exist locally;
- `scripts/validate-release-readiness.sh vX.Y.Z` passes before the tag is
  created;
- GitHub's release workflow is metadata-only and manually dispatched. Do not
  rely on a tag-push workflow for readiness: after a tag is pushed, the
  readiness script intentionally fails closed because the tag already exists.

`scripts/check_latest_tools.sh` is an advisory networked current-version check.
Run it before updating pinned tools and before release when network access is
available, but do not make tag readiness depend on live upstream state.

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
2. Local gates pass: `scripts/checks.sh`, `cargo deny check`, and `cargo audit`.
3. The maintainer runs pentest and writes temporary findings to root
   `PENTEST.md`.
4. Findings are reviewed and fixed.
5. Documentation, tests, and release notes are updated for the fixes.
6. `PENTEST.md` is removed after findings are handled, before the remediation
   commit is finalized.
7. Local gates are run again.
8. GitHub CI and CodeQL default setup are checked after the fix commit.
9. A permanent report is written at `security/pentest/vX.Y.Z.md` only when the
   exact implementation commit has passed with `Status: PASS`.
10. Commit only the permanent report as the release report commit.
11. GitHub CI and CodeQL default setup are checked on the release report commit.
12. `scripts/validate-release-readiness.sh vX.Y.Z` passes locally through the
    versioned release gate before the tag is created.
13. Tagging and pushing tags happen only when explicitly requested.

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
| Public RLP derives had only an evaluation/prototype milestone. | Added `v0.25.0 - Public RLP Derives`. |
| Full EIP-712 `encodeType`/`encodeData`/`hashStruct` support was missing from the roadmap. | Added `v0.26.0 - EIP-712 Typed-Data Encoder`. |
| EIP-712 JSON-RPC typed-data parsing was deferred from the no-JSON typed encoder without a visible patch milestone. | Added `v0.26.1 - EIP-712 JSON Typed-Data Parser Boundary`. |
| A first-party optional software Keccak backend was deferred without a versioned admission point. | Added `v0.27.0 - Optional Keccak Backend Admission`. |
| Formal verification evidence was not scheduled. | Added `v0.177.0` through `v0.179.0` for Kani, Miri, sanitizer, and bounded invariant evidence as extra assurance, not replacements for fuzzing, conformance tests, pentest, or audit. |
| ABI encoding, Engine API, SSZ, and DevP2P/RLPx were marked deferred. | Added explicit ABI/contract releases `v0.120.0..=v0.129.0`, consensus/Engine releases `v0.141.0..=v0.153.0`, and networking releases `v0.154.0..=v0.164.0`. |
| ENS and common ERC/application standards were not scheduled. | Added `v0.127.0..=v0.129.0` for common standards, ENS, permit, interface helpers, and contract-tooling fuzz/DX gates. |
| Node-level sync, txpool, mining/validator boundaries, and observability were not scheduled. | Added storage/client releases `v0.130.0..=v0.140.0`, networking/sync releases `v0.154.0..=v0.164.0`, and operational-runtime release `v0.139.0`. |
| REVM dependency admission failed the existing dependency policy. | Added `v0.37.1 - REVM Dependency Recheck` before execution work may continue. |
| Native audited EVM execution was not explicitly versioned; REVM could look like the long-term core. | Added the first-party engine and precompile sequence at `v0.40.0..=v0.52.9`, then complete execution, state transition, conformance, tracing, and simulation at `v0.69.0..=v0.91.0`. |
| Default verification previously depended directly on `k256` and used direct `sha3` test wrappers, which conflicted with the long-term first-party-core goal. | Added `v0.37.2` and `v0.37.3` to audit core dependencies, move cryptographic implementation crates behind explicit boundaries/features, and document any accepted cryptographic backend plan. |
| `subtle`, `alloy-rlp`, dev `serde_json`, optional `serde`/`serde_json`, and optional `sanitization` need explicit long-term dependency classifications before execution grows. | Added `v0.37.4` and `v0.37.5` so constant-time helpers, reference oracles, JSON parser support, and sanitization bridges remain deliberate dependency choices. |
| `v0.45.0` deliberately admits cryptographic precompiles as fail-closed descriptors without concrete execution backends. | Added `v0.46.0` through `v0.52.0` for SHA-256, RIPEMD-160, ECRECOVER, ModExp, BN254, BLAKE2F, KZG/BLS backend planning, conformance vectors, fuzzing, dependency review, and pentest gates before state-test claims depend on them. |
| Native opcodes alone do not make full Ethereum execution support; genesis, full block validity, trie-root construction, state transition integration, blob/KZG validation, EOF, and full execution fixtures were not versioned before RPC work. | Added `v0.69.0..=v0.88.0` for full execution, KZG, EOF, current-fork maintenance, fixtures, differential evidence, and performance gates. |
| The native EVM state-access pass intentionally fails closed for pre-London forks until historical gas/opcode rules are implemented. | Added `v0.43.1 - Native EVM Historical Fork Matrix` and `v0.43.2 - Native EVM Pre-Berlin State Gas Schedules` before calls/create build on state access. |
| Rich protocol values were borrowed views only, leaving no owned SDK model. | Added `v0.53.0..=v0.59.0` for general integer/byte primitives, owned transaction/block/state models, and lossless ref/owned/validated conversions. |
| Protocol typestates did not carry transaction payloads or evidence. | Added `v0.62.0 - Payload-Bound Transaction Typestates`. |
| Protocol and native EVM crates exposed disconnected address, word, gas, state, and result domains. | Added `v0.64.0` and `v0.65.0` for shared execution domains and native-core integration before state transition. |
| Fork selection relied on fragmented enums and ordinal chronology. | Added `v0.63.0 - Fork Rules And Chain Specification 2.0` with identity, activation, capability, and parameter separation. |
| Provider transports and end-to-end transaction workflows were not concretely planned. | Added `v0.92.0..=v0.108.0` for typed RPC methods, HTTP/WS/IPC/EIP-1193 transports, provider layers, transaction builders/fillers, simulation, signing, broadcasting, watching, replacement, and live-node tests. |
| Wallet, key-management, contract-signature, multisig, and account-abstraction ecosystems were missing. | Added `v0.109.0..=v0.119.0` for local/remote/hardware signers, keystores, HD wallets, ERC-1271, Safe, ERC-4337, paymasters, session keys, and EIP-7702 delegated workflows. |
| Database, canonical-chain, fork-choice, crash consistency, pruning, history expiry, and runtime supervision were not planned concretely. | Added `v0.130.0..=v0.140.0` for persistent stores, atomic batches, migrations, snapshots, pruning/archive modes, canonical reorgs, head tracking, invalidation, supervision, and performance gates. |
| Consensus light-client work lacked bootstrap, weak subjectivity, aggregate signatures, committee rotation, scoring, persistence, and execution-proof binding. | Added `v0.146.0..=v0.151.0` for a complete light-client security model and official end-to-end vectors. |
| Peer management, request scheduling, bans, bounded multi-peer sync, and historical-data acquisition were absent. | Added `v0.158.0..=v0.164.0` for peer services, request schedulers, txpool, sync, Portal/history acquisition, and builder/validator boundaries. |
| EVM tracing, state overrides, call traces, state diffs, and debug/trace models were missing. | Added `v0.89.0..=v0.91.0` for inspector hooks, trace models, deterministic simulation, and RPC trace interoperability. |
| Witnesses, stateless execution, commitment-scheme agility, Verkle/binary trees, and state/history evolution were not versioned. | Added `v0.165.0..=v0.174.0` for proof-format abstraction, witnesses, stateless execution, future commitments, state-expiry policy, zk-proof boundaries, and fork-maintenance automation. |
| SDK compatibility and documentation drift were not release-blocking. | Added `v0.66.0`, `v0.180.0`, and `v0.181.0` for feature truthfulness, generated dependency snippets, semver/feature/serde compatibility gates, and task-oriented documentation. |
| Consensus types, Engine boundaries, and a light client did not amount to a full beacon node. | Added `v0.189.0..=v0.234.0` for consensus architecture, complete transition and fork choice, storage, networking, sync, Engine coordination, PeerDAS, historical deposits/genesis, beacon orchestration, block production, and server/validator APIs. |
| PeerDAS state, storage, networking, and sync consumers were scheduled before the cell/KZG/reconstruction core. | Moved the first-party PeerDAS core to `v0.193.0`; all DA consumers now follow it, and `v0.265.0` audits the implementation and acceleration boundaries. |
| Historical deposit-contract tracking, deposit trees, eth1 voting, and genesis construction were missing. | Added `v0.228.0` and `v0.229.0` before beacon-node orchestration. |
| Block-production ownership was split ambiguously between beacon and validator clients. | Added beacon-owned, embeddable unsigned production at `v0.233.0`; `v0.234.0` exposes it, and `v0.241.0` limits the validator client to independent checks, slashing authorization, signing, and publication. |
| Live validator duties preceded slashing protection and the signer. | Reordered `v0.235.0..=v0.244.0` so the slashing kernel, transactional database, EIP-3076, key foundation, and signer all precede duty scheduling and every signature-producing duty. |
| Validator key generation and deposit artifacts lacked EIP-2333/EIP-2334 and withdrawal-key separation. | Added `v0.238.0` for key derivation, strict key roles, offline withdrawal credentials, and deposit-data generation/verification. |
| Keymanager, remote signing, and HSM/hardware custody were conflated. | Split operator Keymanager control, outbound remote signing/slashing authority, and signer-to-HSM/KMS custody into `v0.245.0..=v0.247.0`; added threshold/DVT coordination at `v0.248.0`. |
| Builder relay integration and safe local-builder fallback were only a boundary decision. | Added `v0.249.0` and `v0.250.0` for Builder API workflows, relay multiplexing, bid/reveal validation, local fallback, withholding defenses, and protocol-native PBS evolution. |
| Optional network slashing detection, distributed signing, validator analytics, and connectivity diagnostics were absent. | Added `v0.248.0`, `v0.251.0`, `v0.252.0`, and `v0.253.0` with explicit trust and resource boundaries. |
| Production beacon-node and validator-client executables, packaging, data directories, signals, exit codes, upgrades, and rollback were not explicit. | Added separate binary and packaging milestones at `v0.254.0` and `v0.255.0`. |
| A Lighthouse/Prysm-class claim lacked deterministic simulation, mandatory Hive suites, broad client matrices, and quantitative long-testnet/performance gates. | Added `v0.257.0..=v0.262.0`, including the numeric acceptance contract at `v0.258.0`. |
| Later SSZ, BLS, PeerDAS, erasure-coding, and acceleration implementations were not covered by an implementation-level core audit. | Added `v0.265.0` and expanded the integration audits at `v0.266.0..=v0.269.0`. |
| The final unchanged-candidate claim ignored manifest, lockfile, SBOM, and checksum changes required by a `1.0.0` version promotion. | Added `v0.273.0`, `v0.274.0`, and an explicit `v1.0.0-rc.1` exact-candidate flow; the stable tag must point to the unchanged approved RC commit. |
| The final API freeze and production release candidate occurred before full consensus-client abstractions existed. | Reclassified `v0.182.0..=v0.188.0` as foundation stabilization and moved complete remediation, freeze, rehearsal, promotion, and candidate admission to `v0.270.0..=v0.274.0` plus `v1.0.0-rc.1`. |

## Phase 0: Repository And Release Discipline

### v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.97.0` pinned.
- Rust `1.90.0` through `1.97.0` compatibility policy.
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

### v0.2.0 - Release Readiness Gate

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

## Phase 1: Primitive And Error Foundation

### v0.3.0 - Domain Newtypes

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

### v0.4.0 - Stable Error Model

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

### v0.5.0 - Decode Budget Model

Goal: make resource limits mandatory for untrusted bytes.

Deliverables:

- byte, list, nesting, allocation, proof-node, and item-count limits;
- checked arithmetic helpers for lengths and offsets;
- adversarial tests for budget rejection.

Verification:

- `cargo test -p eth-valkyoth-codec`

Exit criteria:

- No decoder entry point can be designed without an explicit budget parameter.

## Phase 2: RLP Codec In Small Passes

### v0.6.0 - RLP Scalar Decoder

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

### v0.7.0 - RLP List Decoder

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

### v0.8.0 - Canonical RLP Integers

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

### v0.9.0 - RLP Encoding Round Trips

Goal: add canonical encoding for admitted RLP values.

Deliverables:

- encoding helpers;
- decode-then-encode canonicality tests;
- property tests or table-driven round trips.

Verification:

- `cargo test -p eth-valkyoth-codec`

Exit criteria:

- Canonical values round-trip without accepting noncanonical forms.

### v0.9.1 - Canonical Integer Source Of Truth

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

### v0.9.2 - Primitive RLP Bridge

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

### v0.9.3 - Keccak Boundary Decision

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

### v0.10.0 - RLP Fuzz Harness

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

## Phase 3: Transaction Envelopes

### v0.11.0 - Transaction Envelope Shell

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

### v0.12.0 - Legacy Transaction Decode

Goal: decode legacy Ethereum transactions without sender recovery.

Deliverables:

- field model;
- gas, value, nonce, input, and signature field bounds;
- malformed field tests.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Legacy transactions can be decoded into an unvalidated state only.

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

### v0.52.1 - BLS12-381 Canonical Field And Point Encodings

Status: implementation complete; awaiting independent pentest and retest.

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
  and clears partial encode-data output.

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

### v0.52.2 - BLS12-381 G1 Arithmetic And Addition

Status: planned.

Goal: implement dependency-free G1 field arithmetic and the `0x0b` addition
precompile with official positive, infinity, invalid-field, and invalid-curve
vectors.

Deliverables:

- fixed-width Fp add, subtract, multiply, square, inversion, and square-root
  operations with checked domain conversion;
- complete G1 affine/projective conversion, negation, doubling, and addition
  formulas covering infinity, equal points, and inverse points;
- curve-membership validation separated from subgroup validation;
- charged `0x0b` execution over exactly two G1 points with canonical 128-byte
  output and no subgroup rejection, as required by EIP-2537;
- output-unchanged and no-arithmetic evidence for wrong-kind, wrong-length,
  malformed-point, and out-of-gas failures.

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
- `v0.52.2 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.3 - BLS12-381 Fp2, G2, And Addition

Status: planned.

Goal: implement dependency-free Fp2/G2 arithmetic and the `0x0d` addition
precompile, then establish the extension-tower foundation required by pairing.

Deliverables:

- Fp2 add, subtract, multiply, square, conjugate, inverse, and square-root
  operations using the EIP-2537 non-residue and coefficient order;
- complete G2 affine/projective conversion, negation, doubling, and addition;
- G2 curve-membership validation that remains separate from subgroup checks;
- charged `0x0d` execution over exactly two G2 points with canonical 256-byte
  output and no subgroup rejection;
- explicit extension-tower conventions reused by later pairing releases.

Verification:

- official EIP-2537 G2 addition vectors and generator constants;
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
- `v0.52.3 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.4 - BLS12-381 Subgroup Validation

Status: planned.

Goal: admit bounded first-party G1/G2 subgroup checks for MSM and pairing
inputs without incorrectly adding subgroup rejection to the addition APIs.

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
- `v0.52.4 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.5 - BLS12-381 Multiscalar Multiplication

Status: planned.

Goal: implement `0x0c` and `0x0e` with bounded Pippenger-style execution,
official discount gas, item limits, vectors, differential tests, and CPU/gas
evidence.

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
- `v0.52.5 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.6 - BLS12-381 Map-To-Curve

Status: planned.

Goal: implement the EIP-2537 Fp-to-G1 and Fp2-to-G2 mappings at `0x10` and
`0x11` from the pinned mapping specification and official vectors.

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
- `v0.52.6 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.7 - BLS12-381 Pairing Foundation

Status: planned.

Goal: add the first-party Fp6/Fp12 tower, line functions, Miller loop, and
bounded final-exponentiation foundation while pairing remains fail closed.

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
- `v0.52.7 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.8 - BLS12-381 Pairing Execution

Status: planned.

Goal: admit non-empty `0x0f` pairing execution with canonical zero/one output,
subgroup enforcement, official vectors, differential checks, and gas/CPU
evidence.

Deliverables:

- charged non-empty pairing execution over complete 384-byte tuples;
- mandatory G1/G2 curve and subgroup validation for every tuple;
- canonical 32-byte false/true output words and no alternate success values;
- explicit precompile-error outcome contract so the future CALL dispatcher can
  burn all supplied call gas as required by EIP-2537;
- output-unchanged behavior on charge failure and malformed input;
- bounded batch accumulation reusing the `v0.52.7` arithmetic exactly once per
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
- `v0.52.8 implementation stop reached. Run pentest for this exact
  commit.`

### v0.52.9 - Prague Advanced-Precompile Admission

Status: planned.

Goal: run the complete official EIP-2537 fixture set, fuzz and pentest every
advanced precompile path, and admit only the Prague claims backed by evidence.

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
- `v0.52.9 implementation stop reached. Run pentest for this exact
  commit.`

## Roadmap Expansion From The 2026 Gap Analysis

The releases below replace the earlier narrow integration roadmap. They assign
every gap identified by the July 2026 completeness reviews to a version instead
of leaving work as an unversioned deferral. The roadmap may extend beyond
`v0.274.0` when new official Ethereum work or a newly discovered completeness
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

Roadmap source review date: 2026-07-16. Active fork names and requirements must
come from pinned official sources, not memory:

- <https://ethereum.org/roadmap/>
- <https://ethereum.org/roadmap/statelessness/>
- <https://github.com/ethereum/execution-specs>
- <https://github.com/ethereum/consensus-specs>
- <https://github.com/ethereum/execution-apis>
- <https://ethereum.github.io/beacon-APIs/>
- <https://ethereum.github.io/keymanager-APIs/>
- <https://ethereum.github.io/builder-specs/>
- <https://github.com/ethereum/hive>
- <https://eips.ethereum.org/EIPS/eip-3076>
- <https://eips.ethereum.org/EIPS/eip-3540>

## Phase 9: Owned SDK And Shared Domain Foundation

### v0.53.0 - General Integer Primitives

Status: planned.

Goal: deliver the General Integer Primitives release with this required outcome: Ethereum-sized integer work no longer depends on transaction-specific `Wei` helpers or external core types.

Deliverables:

- First-party `U256` and `I256`, checked arithmetic, endian conversion, parsing, formatting, and explicit overflow policy.

Verification:

- Arithmetic KATs, boundary/property tests, differential checks, fuzzing.

Exit criteria:

- Ethereum-sized integer work no longer depends on transaction-specific `Wei` helpers or external core types.
- `v0.53.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.54.0 - Bytes And Hash Domains

Status: planned.

Goal: deliver the Bytes And Hash Domains release with this required outcome: Raw byte arrays and generic `B256` are not the only public representation for semantically distinct domains.

Deliverables:

- Owned and borrowed `Bytes`, fixed-byte families, and distinct transaction, block, receipt, state, storage, code, and signing hash newtypes.

Verification:

- Conversion, domain-mismatch compile tests, allocation-limit tests, fuzzing.

Exit criteria:

- Raw byte arrays and generic `B256` are not the only public representation for semantically distinct domains.
- `v0.54.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.55.0 - Ethereum Text And Serde Interoperability

Status: planned.

Goal: deliver the Ethereum Text And Serde Interoperability release with this required outcome: Common Ethereum wire and display forms round-trip canonically without weakening the default graph.

Deliverables:

- Quantity/data hex codecs, EIP-55 and EIP-1191 address checksums, bounded optional serde, JSON-RPC quantity rules, and stable text errors.

Verification:

- Official checksum vectors, serde snapshots, malformed/oversized corpus, differential checks.

Exit criteria:

- Common Ethereum wire and display forms round-trip canonically without weakening the default graph.
- `v0.55.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.56.0 - Owned Transaction Models

Status: planned.

Goal: deliver the Owned Transaction Models release with this required outcome: Applications can retain and mutate complete transactions without keeping input buffers alive.

Deliverables:

- Owned legacy, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 transactions, requests, signatures, access lists, authorization lists, and blob sidecars.

Verification:

- Round trips against borrowed forms and official transaction fixtures.

Exit criteria:

- Applications can retain and mutate complete transactions without keeping input buffers alive.
- `v0.56.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.57.0 - Owned Block And Receipt Models

Status: planned.

Goal: deliver the Owned Block And Receipt Models release with this required outcome: Full execution payload data has stable owned SDK models.

Deliverables:

- Owned headers, blocks, bodies, receipts, logs, withdrawals, ommers, execution requests, and fork-specific optional fields.

Verification:

- Cross-fork fixture round trips, root-input serialization tests, serde snapshots.

Exit criteria:

- Full execution payload data has stable owned SDK models.
- `v0.57.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.58.0 - Owned State And Execution Models

Status: planned.

Goal: deliver the Owned State And Execution Models release with this required outcome: State and execution APIs no longer require disconnected adapter-only models.

Deliverables:

- Owned accounts, code, storage slots, state diffs, execution environments, results, logs, refunds, access summaries, and witness references.

Verification:

- State conversion/property tests, deterministic serialization, allocation-limit tests.

Exit criteria:

- State and execution APIs no longer require disconnected adapter-only models.
- `v0.58.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.59.0 - Lossless Model Conversion Matrix

Status: planned.

Goal: deliver the Lossless Model Conversion Matrix release with this required outcome: Every supported representation change is explicit, testable, and documented as lossless or intentionally lossy.

Deliverables:

- Checked conversions among borrowed, owned, canonical, validated, RPC, signer, and execution representations with preserved evidence.

Verification:

- Conversion matrix tests, lossy-conversion rejection, compile-fail typestate tests.

Exit criteria:

- Every supported representation change is explicit, testable, and documented as lossless or intentionally lossy.
- `v0.59.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.60.0 - Bounded Allocation Convenience

Status: planned.

Goal: deliver the Bounded Allocation Convenience release with this required outcome: Ergonomic allocation support does not weaken bounded resource or atomic-output guarantees.

Deliverables:

- Caller-owned transactional output buffers, scratch arenas, reusable workspaces, bounded collections, and all-or-nothing writer APIs.

Verification:

- OOM/limit simulation, output-unchanged tests, reuse tests, Miri candidates.

Exit criteria:

- Ergonomic allocation support does not weaken bounded resource or atomic-output guarantees.
- `v0.60.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.61.0 - Decode Policies And Error Context

Status: planned.

Goal: deliver the Decode Policies And Error Context release with this required outcome: Integrators can select reviewed policies and diagnose failures without parsing strings.

Deliverables:

- Named deployment policy builders plus structured field/index/offset/source error context without secret leakage.

Verification:

- Error snapshot tests, redaction tests, nested malformed fixtures, compatibility checks.

Exit criteria:

- Integrators can select reviewed policies and diagnose failures without parsing strings.
- `v0.61.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.62.0 - Payload-Bound Typestates

Status: planned.

Goal: deliver the Payload-Bound Typestates release with this required outcome: Validation state cannot become detached from the exact payload it proves.

Deliverables:

- Transaction/block payloads travel with canonicality, fork, signature, proof, and execution evidence; constructors remain proof-gated.

Verification:

- Compile-fail transition tests, evidence preservation tests, forged-state rejection.

Exit criteria:

- Validation state cannot become detached from the exact payload it proves.
- `v0.62.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.63.0 - Chain Specification And Fork Rules 2.0

Status: planned.

Goal: deliver the Chain Specification And Fork Rules 2.0 release with this required outcome: Consensus behavior never depends on enum ordinal ordering or a hardcoded mainnet chronology.

Deliverables:

- Separate fork identity, activation schedule, rule capabilities, parameters, system hooks, and complete historical/custom-chain configuration.

Verification:

- Historical mainnet vectors, custom-chain schedules, monotonicity/property tests.

Exit criteria:

- Consensus behavior never depends on enum ordinal ordering or a hardcoded mainnet chronology.
- `v0.63.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.64.0 - Shared Protocol And Execution Domains

Status: planned.

Goal: deliver the Shared Protocol And Execution Domains release with this required outcome: Equivalent protocol and EVM concepts no longer drift behind parallel types.

Deliverables:

- One address, word, gas, account, state, log, access, execution-status, and error vocabulary shared by protocol and native EVM crates.

Verification:

- API conversion audit, compile checks, no-copy path tests, semver baseline.

Exit criteria:

- Equivalent protocol and EVM concepts no longer drift behind parallel types.
- `v0.64.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.65.0 - Native EVM Core Integration

Status: planned.

Goal: deliver the Native EVM Core Integration release with this required outcome: The optional execution facade is a real first-party path, not a disconnected descriptor layer.

Deliverables:

- `eth-valkyoth-evm` consumes the first-party core and shared domains; one validated owned transaction executes through the public boundary.

Verification:

- End-to-end fixture, fail-closed unsupported paths, reference differential test.

Exit criteria:

- The optional execution facade is a real first-party path, not a disconnected descriptor layer.
- `v0.65.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.66.0 - Facade Prelude And Feature Truth

Status: planned.

Goal: deliver the Facade Prelude And Feature Truth release with this required outcome: Public discovery is simple and no feature name implies functionality it does not provide.

Deliverables:

- Curated prelude, task-oriented modules, truthful feature names, generated feature/dependency tables, and default-graph assertions.

Verification:

- Feature powerset sampling, README snippet generation check, docs tests.

Exit criteria:

- Public discovery is simple and no feature name implies functionality it does not provide.
- `v0.66.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.67.0 - Ecosystem Conversion Adapters

Status: planned.

Goal: deliver the Ecosystem Conversion Adapters release with this required outcome: Interoperability is available without making third-party core models authoritative.

Deliverables:

- Optional reviewed conversions for Alloy, Reth, and other admitted ecosystem types behind compatibility features.

Verification:

- Version matrix, conversion fixtures, default-graph exclusion checks.

Exit criteria:

- Interoperability is available without making third-party core models authoritative.
- `v0.67.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.68.0 - Owned SDK Hardening

Status: planned.

Goal: deliver the Owned SDK Hardening release with this required outcome: The owned SDK foundation is stable enough for execution, providers, wallets, and storage to build upon.

Deliverables:

- Fuzz all owned parsers/conversions, lock serde and display snapshots, establish semver baselines, and audit allocation behavior.

Verification:

- Full SDK fuzz suite, cargo-semver-checks baseline, docs/package checks, pentest.

Exit criteria:

- The owned SDK foundation is stable enough for execution, providers, wallets, and storage to build upon.
- `v0.68.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 10: Complete First-Party Execution

### v0.69.0 - Official State-Test Admission

Status: planned.

Goal: deliver the Official State-Test Admission release with this required outcome: Native execution claims are backed by official state-test evidence.

Deliverables:

- Harness official execution state tests by fork with explicit supported and skipped scopes.

Verification:

- Pinned fixtures, unsupported-reason report, cross-client differential samples.

Exit criteria:

- Native execution claims are backed by official state-test evidence.
- `v0.69.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.70.0 - Native EVM Audit Hardening

Status: planned.

Goal: deliver the Native EVM Audit Hardening release with this required outcome: Deeper state transition work rests on an independently reviewed engine.

Deliverables:

- Broaden bytecode, stack, memory, gas, journal, call-frame, and precompile fuzzing; close audit findings.

Verification:

- Fuzz corpus, load/DoS tests, stack report, clean pentest/retest.

Exit criteria:

- Deeper state transition work rests on an independently reviewed engine.
- `v0.70.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.71.0 - Genesis And Chain Configuration

Status: planned.

Goal: deliver the Genesis And Chain Configuration release with this required outcome: A chain can be initialized from explicit configuration without external core logic.

Deliverables:

- Parse and validate genesis/config data, allocs, fork schedules, terminal conditions, and initial state/header roots.

Verification:

- Mainnet/testnet/custom genesis fixtures and negative cases.

Exit criteria:

- A chain can be initialized from explicit configuration without external core logic.
- `v0.71.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.72.0 - Semantic Transaction Validity

Status: planned.

Goal: deliver the Semantic Transaction Validity release with this required outcome: Decoded transactions can be proven consensus-valid for a stated chain context.

Deliverables:

- Complete intrinsic gas, nonce, balance, fee, chain, authorization, blob, initcode, sender, and fork checks for every transaction type.

Verification:

- Official transaction tests, cross-type property tests, client differential checks.

Exit criteria:

- Decoded transactions can be proven consensus-valid for a stated chain context.
- `v0.72.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.73.0 - Header And Block Validity

Status: planned.

Goal: deliver the Header And Block Validity release with this required outcome: Headers and block envelopes can be validated against parent and chain state.

Deliverables:

- Parent linkage, gas/base fee, difficulty/TTD, timestamps, ommers, withdrawals, blob gas, requests, roots, and fork-field validation.

Verification:

- Blockchain/header fixtures across all claimed forks.

Exit criteria:

- Headers and block envelopes can be validated against parent and chain state.
- `v0.73.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.74.0 - State Transition And Journaling

Status: planned.

Goal: deliver the State Transition And Journaling release with this required outcome: A complete block transition can be computed first party.

Deliverables:

- Execute ordered transactions, commit/revert journaled state, apply rewards/system operations, and emit deterministic outcomes.

Verification:

- State-transition fixtures, nested revert tests, crash-free bounded execution.

Exit criteria:

- A complete block transition can be computed first party.
- `v0.74.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.75.0 - Receipts Logs Bloom And Withdrawals

Status: planned.

Goal: deliver the Receipts Logs Bloom And Withdrawals release with this required outcome: Post-execution outputs match consensus serialization and accounting rules.

Deliverables:

- Receipt construction, cumulative gas, status/root rules, logs bloom, withdrawal application, and execution request outputs.

Verification:

- Receipt/blockchain fixtures and root comparison.

Exit criteria:

- Post-execution outputs match consensus serialization and accounting rules.
- `v0.75.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.76.0 - Trie Construction And Root Computation

Status: planned.

Goal: deliver the Trie Construction And Root Computation release with this required outcome: The crate computes all execution-layer Merkle Patricia roots it validates.

Deliverables:

- First-party account, storage, transaction, and receipt trie builders with canonical node encoding and root calculation.

Verification:

- TrieTests, mutation/property tests, proof round trips, fuzzing.

Exit criteria:

- The crate computes all execution-layer Merkle Patricia roots it validates.
- `v0.76.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.77.0 - KZG Trusted Setup Boundary

Status: planned.

Goal: deliver the KZG Trusted Setup Boundary release with this required outcome: No blob proof runs against implicit or unverified setup material.

Deliverables:

- Versioned setup format, checksum/provenance, bounded loading, validation, and backend-independent setup handles.

Verification:

- Official setup checks, corruption/truncation tests, reproducibility report.

Exit criteria:

- No blob proof runs against implicit or unverified setup material.
- `v0.77.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.78.0 - KZG Field And Polynomial Core

Status: planned.

Goal: deliver the KZG Field And Polynomial Core release with this required outcome: KZG arithmetic foundations are first party and independently verified.

Deliverables:

- First-party BLS scalar field, polynomial evaluation, roots of unity, FFT/IFFT, and bounded workspace policy.

Verification:

- Algebra properties, independent vectors, constant-bound and performance tests.

Exit criteria:

- KZG arithmetic foundations are first party and independently verified.
- `v0.78.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.79.0 - KZG Commitments And Proofs

Status: planned.

Goal: deliver the KZG Commitments And Proofs release with this required outcome: Blob commitments and proofs are cryptographically executable, not descriptors.

Deliverables:

- Blob commitments, proof creation/verification, batch verification, and versioned-hash derivation.

Verification:

- Official EIP-4844 fixtures, differential vectors, malformed/batch fuzzing.

Exit criteria:

- Blob commitments and proofs are cryptographically executable, not descriptors.
- `v0.79.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.80.0 - Point-Evaluation Precompile Execution

Status: planned.

Goal: deliver the Point-Evaluation Precompile Execution release with this required outcome: The precompile is consensus-compatible for all claimed forks.

Deliverables:

- Admit the EIP-4844 point-evaluation precompile through verified KZG setup and exact gas/output/error behavior.

Verification:

- Official precompile vectors, gas ordering, fail-closed setup tests.

Exit criteria:

- The precompile is consensus-compatible for all claimed forks.
- `v0.80.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.81.0 - Blob Transaction And Block Integration

Status: planned.

Goal: deliver the Blob Transaction And Block Integration release with this required outcome: EIP-4844 validity is complete from transaction through block transition.

Deliverables:

- Enforce hash version/count, sidecar consistency, blob gas/base fee, commitments, proofs, and block limits.

Verification:

- Transaction/block fixtures and adversarial sidecar tests.

Exit criteria:

- EIP-4844 validity is complete from transaction through block transition.
- `v0.81.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.82.0 - EOF Format And Static Validation

Status: planned.

Goal: deliver the EOF Format And Static Validation release with this required outcome: EOF bytecode is admitted only after complete static validation.

Deliverables:

- Implement versioned EOF containers, section/type rules, validation stack, and fork gating.

Verification:

- Official EOF validation suite and malformed-structure fuzzing.

Exit criteria:

- EOF bytecode is admitted only after complete static validation.
- `v0.82.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.83.0 - EOF Control Flow And Execution

Status: planned.

Goal: deliver the EOF Control Flow And Execution release with this required outcome: Valid EOF containers execute with fork-correct semantics.

Deliverables:

- Implement EOF instructions, validated jumps/calls, stack contracts, data access, gas, and execution semantics.

Verification:

- Official execution vectors and differential tests.

Exit criteria:

- Valid EOF containers execute with fork-correct semantics.
- `v0.83.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.84.0 - EOF Creation And State Transition

Status: planned.

Goal: deliver the EOF Creation And State Transition release with this required outcome: EOF is complete at transaction and block level for claimed forks.

Deliverables:

- Integrate EOF deployment, init containers, code validation, creation rules, receipts, and state changes.

Verification:

- Blockchain/state fixtures covering deployment and rejection.

Exit criteria:

- EOF is complete at transaction and block level for claimed forks.
- `v0.84.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.85.0 - Current Fork Manifest Admission

Status: planned.

Goal: deliver the Current Fork Manifest Admission release with this required outcome: Every current fork claim maps to pinned rules and fixtures rather than a hand-maintained name list.

Deliverables:

- Generate a reviewed rule manifest from pinned execution/consensus specs for Prague/Pectra, Osaka/Fusaka/Fulu, and then-active Glamsterdam, Hegotá, Gloas, or successor work as applicable.

Verification:

- Source-lock drift check and feature-by-feature conformance matrix.

Exit criteria:

- Every current fork claim maps to pinned rules and fixtures rather than a hand-maintained name list.
- `v0.85.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.86.0 - Current Fork Execution Changes

Status: planned.

Goal: deliver the Current Fork Execution Changes release with this required outcome: No current execution-fork rule remains a descriptor or silent unsupported path.

Deliverables:

- Implement all opcodes, precompiles, system contracts, gas changes, request types, and state-transition changes in the admitted current manifest.

Verification:

- Official tests per EIP/fork, client differential suite, pentest.

Exit criteria:

- No current execution-fork rule remains a descriptor or silent unsupported path.
- `v0.86.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.87.0 - Complete Execution Fixture Gate

Status: planned.

Goal: deliver the Complete Execution Fixture Gate release with this required outcome: All claimed historical and current execution behavior has fixture evidence.

Deliverables:

- Run TransactionTests, BlockchainTests, GenesisTests, TrieTests, DifficultyTests, EOF tests, state tests, and current successor suites.

Verification:

- Generated pass/fail/skip report with zero unexplained skips.

Exit criteria:

- All claimed historical and current execution behavior has fixture evidence.
- `v0.87.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.88.0 - Execution Differential And Performance Gate

Status: planned.

Goal: deliver the Execution Differential And Performance Gate release with this required outcome: The first-party engine is correct and operationally bounded enough for higher layers.

Deliverables:

- Differentially compare blocks/state roots with independent clients and establish CPU, memory, stack, and gas benchmarks.

Verification:

- Reproducible differential corpus and regression thresholds.

Exit criteria:

- The first-party engine is correct and operationally bounded enough for higher layers.
- `v0.88.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.89.0 - Inspector And Hook Framework

Status: planned.

Goal: deliver the Inspector And Hook Framework release with this required outcome: Tooling can observe execution without changing consensus results.

Deliverables:

- Bounded opcode/call/state/log inspectors, cancellation, filtering, and no-op zero-cost path.

Verification:

- Hook-order tests, cancellation tests, overhead benchmarks, fuzzing.

Exit criteria:

- Tooling can observe execution without changing consensus results.
- `v0.89.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.90.0 - Trace And State-Diff Models

Status: planned.

Goal: deliver the Trace And State-Diff Models release with this required outcome: Execution evidence is usable by debuggers and analysis tools.

Deliverables:

- Call traces, opcode traces, state/access diffs, gas profiles, revert data, and client-compatible trace projections.

Verification:

- Cross-client trace fixtures, redaction and size-limit tests.

Exit criteria:

- Execution evidence is usable by debuggers and analysis tools.
- `v0.90.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.91.0 - Deterministic Simulation And Overrides

Status: planned.

Goal: deliver the Deterministic Simulation And Overrides release with this required outcome: Transactions and bundles can be simulated safely before signing or broadcast.

Deliverables:

- `eth_call`-style execution, block/state overrides, bundles, access-list generation, trace/debug RPC models, and deterministic reports.

Verification:

- Override fixtures, repeatability tests, bounded workload tests.

Exit criteria:

- Transactions and bundles can be simulated safely before signing or broadcast.
- `v0.91.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 11: Providers And Transaction Lifecycle

### v0.92.0 - Typed RPC Method Surface

Status: planned.

Goal: deliver the Typed RPC Method Surface release with this required outcome: Callers no longer assemble core RPC methods from untyped JSON values.

Deliverables:

- Typed request/response models for execution, debug, trace, txpool, Engine, and supported extension namespaces with explicit trust labels.

Verification:

- Official execution-apis fixtures, serde snapshots, unknown-field policy tests.

Exit criteria:

- Callers no longer assemble core RPC methods from untyped JSON values.
- `v0.92.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.93.0 - Runtime-Neutral Transport Traits

Status: planned.

Goal: deliver the Runtime-Neutral Transport Traits release with this required outcome: Provider logic is independent of HTTP stack and async runtime choice.

Deliverables:

- Bounded request, response, batch, subscription, timeout, cancellation, and transport-error contracts without selecting a runtime.

Verification:

- Mock transport conformance suite and object-safety/no_std checks.

Exit criteria:

- Provider logic is independent of HTTP stack and async runtime choice.
- `v0.93.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.94.0 - HTTP Provider

Status: planned.

Goal: deliver the HTTP Provider release with this required outcome: A production HTTP provider exists without entering the default graph.

Deliverables:

- Optional reviewed HTTP/TLS adapters, authentication redaction, payload limits, timeout policy, and endpoint allowlists.

Verification:

- Malicious server fixtures, TLS/config matrix, cancellation/load tests.

Exit criteria:

- A production HTTP provider exists without entering the default graph.
- `v0.94.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.95.0 - WebSocket Provider

Status: planned.

Goal: deliver the WebSocket Provider release with this required outcome: Long-lived subscriptions fail explicitly and cannot grow memory without bound.

Deliverables:

- Subscription lifecycle, bounded queues, reconnect/resubscribe policy, missed-event signaling, and backpressure.

Verification:

- Disconnect/reorder/flood tests and local-node integration.

Exit criteria:

- Long-lived subscriptions fail explicitly and cannot grow memory without bound.
- `v0.95.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.96.0 - IPC Custom And EIP-1193 Transports

Status: planned.

Goal: deliver the IPC Custom And EIP-1193 Transports release with this required outcome: Desktop, mobile, browser, and embedded integrators can supply an appropriate transport.

Deliverables:

- Unix/Windows IPC, caller-supplied transport, browser EIP-1193, and WASM adapter boundaries.

Verification:

- Platform matrix, browser mock tests, framing/flood tests.

Exit criteria:

- Desktop, mobile, browser, and embedded integrators can supply an appropriate transport.
- `v0.96.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.97.0 - RPC IDs Batching And Cancellation

Status: planned.

Goal: deliver the RPC IDs Batching And Cancellation release with this required outcome: Concurrent and batched calls cannot be confused or left unbounded.

Deliverables:

- Collision-safe IDs, batch correlation, partial failures, cancellation races, concurrency caps, and response size limits.

Verification:

- Reordering/duplication/flood fuzzing and race tests.

Exit criteria:

- Concurrent and batched calls cannot be confused or left unbounded.
- `v0.97.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.98.0 - Method Validation And Block Consistency

Status: planned.

Goal: deliver the Method Validation And Block Consistency release with this required outcome: Typed RPC data is structurally and contextually checked before promotion.

Deliverables:

- Method-specific response validation, chain identity checks, EIP-1898 block references, and consistent multi-call block context.

Verification:

- Malicious/inconsistent provider fixtures and quorum disagreement cases.

Exit criteria:

- Typed RPC data is structurally and contextually checked before promotion.
- `v0.98.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.99.0 - Provider Middleware

Status: planned.

Goal: deliver the Provider Middleware release with this required outcome: Operational policy is composable without hidden retries or data leakage.

Deliverables:

- Bounded retries, rate limits, circuit breakers, caches, metrics, redaction, request classification, and policy composition.

Verification:

- Failure-injection and retry-amplification tests, cache correctness tests.

Exit criteria:

- Operational policy is composable without hidden retries or data leakage.
- `v0.99.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.100.0 - Quorum Verified And Traced Providers

Status: planned.

Goal: deliver the Quorum Verified And Traced Providers release with this required outcome: Trust policy changes the return type and evidence, not only a boolean setting.

Deliverables:

- Multi-provider quorum, proof-backed reads, finalized/safe policies, disagreement evidence, and tracing metadata.

Verification:

- Byzantine provider simulations and proof/quorum fixtures.

Exit criteria:

- Trust policy changes the return type and evidence, not only a boolean setting.
- `v0.100.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.101.0 - Transaction Request Builders

Status: planned.

Goal: deliver the Transaction Request Builders release with this required outcome: Invalid field combinations are rejected before RPC or signing.

Deliverables:

- Fork-aware builders for every transaction family with explicit unset/derived/user-supplied field states.

Verification:

- Compile-fail builder tests and cross-type round trips.

Exit criteria:

- Invalid field combinations are rejected before RPC or signing.
- `v0.101.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.102.0 - Transaction Fillers

Status: planned.

Goal: deliver the Transaction Fillers release with this required outcome: Automatic filling is observable, bounded, and never silently overwrites user intent.

Deliverables:

- Chain ID, nonce, gas, fees, access list, blob fields, and authorization fillers with source/evidence records.

Verification:

- Concurrent nonce tests, hostile RPC fixtures, deterministic fill snapshots.

Exit criteria:

- Automatic filling is observable, bounded, and never silently overwrites user intent.
- `v0.102.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.103.0 - Blob Sidecars And Fee Markets

Status: planned.

Goal: deliver the Blob Sidecars And Fee Markets release with this required outcome: Blob transactions can be prepared end to end with first-party validation.

Deliverables:

- Sidecar construction, fee-history interpretation, blob base-fee calculation, replacement policy, and KZG workflow integration.

Verification:

- Local-node blob tests and fee boundary vectors.

Exit criteria:

- Blob transactions can be prepared end to end with first-party validation.
- `v0.103.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.104.0 - Build Simulate Sign Broadcast Workflow

Status: planned.

Goal: deliver the Build Simulate Sign Broadcast Workflow release with this required outcome: The common transaction lifecycle is available without bypassing validation evidence.

Deliverables:

- High-level typestate workflow from request through simulation, policy approval, signing, raw validation, and broadcast.

Verification:

- End-to-end local-node tests and fault injection at every transition.

Exit criteria:

- The common transaction lifecycle is available without bypassing validation evidence.
- `v0.104.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.105.0 - Pending Transaction Watcher

Status: planned.

Goal: deliver the Pending Transaction Watcher release with this required outcome: A broadcast transaction reaches a final, replaced, dropped, or timed-out terminal state explicitly.

Deliverables:

- Receipt watching, configurable confirmations, safe/finalized heads, reorg detection, timeout, cancellation, and evidence.

Verification:

- Reorg/restart/disconnect simulations.

Exit criteria:

- A broadcast transaction reaches a final, replaced, dropped, or timed-out terminal state explicitly.
- `v0.105.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.106.0 - Replacement Cancellation And Drop Recovery

Status: planned.

Goal: deliver the Replacement Cancellation And Drop Recovery release with this required outcome: Stuck transactions can be managed without unsafe nonce assumptions.

Deliverables:

- Fee-bump rules, cancellation transaction construction, competing hashes, dropped transaction detection, and nonce reconciliation.

Verification:

- Local-node replacement/reorg tests and adversarial provider cases.

Exit criteria:

- Stuck transactions can be managed without unsafe nonce assumptions.
- `v0.106.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.107.0 - Offline Signing Packages

Status: planned.

Goal: deliver the Offline Signing Packages release with this required outcome: Air-gapped and remote signers can participate without trusting provider serialization.

Deliverables:

- Deterministic signing packages, human-review summaries, policy hooks, QR/file-safe encoding, and signed-result verification.

Verification:

- Golden packages, tamper tests, secret-redaction review.

Exit criteria:

- Air-gapped and remote signers can participate without trusting provider serialization.
- `v0.107.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.108.0 - Live Node Integration Matrix

Status: planned.

Goal: deliver the Live Node Integration Matrix release with this required outcome: Provider and lifecycle claims pass against real nodes, not only mocks.

Deliverables:

- Self-managed Podman execution clients covering HTTP, WS, subscriptions, reorgs, blobs, traces, and transaction lifecycles.

Verification:

- Repeatable bring-up/tear-down scripts and CI/manual matrix.

Exit criteria:

- Provider and lifecycle claims pass against real nodes, not only mocks.
- `v0.108.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 12: Signers Wallets And Account Abstraction

### v0.109.0 - Signer Interface 2.0

Status: planned.

Goal: deliver the Signer Interface 2.0 release with this required outcome: Every signing request states exactly what domain and policy is being authorized.

Deliverables:

- Async/runtime-neutral signer contracts for transactions, messages, typed data, authorizations, chain policy, and signer capability discovery.

Verification:

- Mock signer suite, wrong-domain/chain tests, redaction audit.

Exit criteria:

- Every signing request states exactly what domain and policy is being authorized.
- `v0.109.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.110.0 - Local Secret Signer

Status: planned.

Goal: deliver the Local Secret Signer release with this required outcome: Local signing is usable but remains opt-in and security-reviewed.

Deliverables:

- Optional local secp256k1 signer, locked/sanitized secret ownership, deterministic signatures, and explicit export prohibition.

Verification:

- KATs, low-s/recovery checks, memory-sanitization evidence, pentest.

Exit criteria:

- Local signing is usable but remains opt-in and security-reviewed.
- `v0.110.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.111.0 - Encrypted Keystore

Status: planned.

Goal: deliver the Encrypted Keystore release with this required outcome: Keystore handling is compatible and cannot silently admit unsafe cost settings.

Deliverables:

- Web3 Secret Storage compatible import/export, parameter validation, bounded KDF work policy, password handling, and migration.

Verification:

- Official/independent vectors, malformed/KDF DoS tests, interoperability checks.

Exit criteria:

- Keystore handling is compatible and cannot silently admit unsafe cost settings.
- `v0.111.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.112.0 - BIP-39 Mnemonics

Status: planned.

Goal: deliver the BIP-39 Mnemonics release with this required outcome: Mnemonic workflows are standards-compatible and explicitly secret-bearing.

Deliverables:

- Entropy, checksum, language policy, seed derivation, passphrase handling, and sanitization boundaries.

Verification:

- Official vectors, normalization tests, memory handling review.

Exit criteria:

- Mnemonic workflows are standards-compatible and explicitly secret-bearing.
- `v0.112.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.113.0 - BIP-32 And BIP-44 Derivation

Status: planned.

Goal: deliver the BIP-32 And BIP-44 Derivation release with this required outcome: HD Ethereum accounts can be derived without external wallet-core logic.

Deliverables:

- Hardened/non-hardened derivation, Ethereum paths, extended key handling, watch-only support, and path policy.

Verification:

- Official/independent vectors, invalid-child/path tests.

Exit criteria:

- HD Ethereum accounts can be derived without external wallet-core logic.
- `v0.113.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.114.0 - Remote Hardware HSM And KMS Signers

Status: planned.

Goal: deliver the Remote Hardware HSM And KMS Signers release with this required outcome: External key custody integrates through one auditable signer boundary.

Deliverables:

- Capability-based adapters for hardware wallets, HSMs, KMS, and remote signing services with attestation metadata.

Verification:

- Mock protocol matrices, cancellation/timeouts, wrong-key/chain tests.

Exit criteria:

- External key custody integrates through one auditable signer boundary.
- `v0.114.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.115.0 - Signing Policy ERC-1271 And Multisig

Status: planned.

Goal: deliver the Signing Policy ERC-1271 And Multisig release with this required outcome: Contract and policy authorization are first-class, not forced into EOA assumptions.

Deliverables:

- Policy engine, spend/domain allowlists, ERC-1271 verification, threshold signature collections, and audit records.

Verification:

- Contract-wallet fixtures, policy bypass tests, malformed signature corpus.

Exit criteria:

- Contract and policy authorization are first-class, not forced into EOA assumptions.
- `v0.115.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.116.0 - Safe Workflows

Status: planned.

Goal: deliver the Safe Workflows release with this required outcome: Common multisig transactions can be built, reviewed, signed, and followed end to end.

Deliverables:

- Safe transaction hashing, nonce/module/guard modeling, signature packing, service adapters, and execution tracking.

Verification:

- Safe reference vectors and local-contract integration.

Exit criteria:

- Common multisig transactions can be built, reviewed, signed, and followed end to end.
- `v0.116.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.117.0 - ERC-4337 Core

Status: planned.

Goal: deliver the ERC-4337 Core release with this required outcome: User operations have complete typed and cryptographic foundations.

Deliverables:

- UserOperation versions, hashing, validation data, gas fields, aggregation boundaries, and EntryPoint models.

Verification:

- Official account-abstraction vectors and EntryPoint integration.

Exit criteria:

- User operations have complete typed and cryptographic foundations.
- `v0.117.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.118.0 - Bundler EntryPoint And Paymasters

Status: planned.

Goal: deliver the Bundler EntryPoint And Paymasters release with this required outcome: ERC-4337 works end to end with explicit third-party trust boundaries.

Deliverables:

- Bundler RPC, simulation, submission/watch flows, paymaster data/policy, aggregator handling, and reputation/error models.

Verification:

- Local bundler/EntryPoint tests and hostile paymaster fixtures.

Exit criteria:

- ERC-4337 works end to end with explicit third-party trust boundaries.
- `v0.118.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.119.0 - Session Keys And Delegated Accounts

Status: planned.

Goal: deliver the Session Keys And Delegated Accounts release with this required outcome: Delegated authorization is usable without weakening base signature and policy guarantees.

Deliverables:

- Session-key policies, scoped permissions, revocation, EIP-7702 delegated-account workflows, and wallet/account-abstraction composition.

Verification:

- Expiry/revocation/domain tests and local-node workflows.

Exit criteria:

- Delegated authorization is usable without weakening base signature and policy guarantees.
- `v0.119.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 13: ABI Contracts And Application Standards

### v0.120.0 - ABI Type System

Status: planned.

Goal: deliver the ABI Type System release with this required outcome: All standard ABI type shapes are represented without untyped strings.

Deliverables:

- Canonical Solidity ABI types, tuples, arrays, functions, events, errors, selectors, and bounded dynamic-size policy.

Verification:

- Solidity differential vectors and type parser fuzzing.

Exit criteria:

- All standard ABI type shapes are represented without untyped strings.
- `v0.120.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.121.0 - ABI Encode Decode

Status: planned.

Goal: deliver the ABI Encode Decode release with this required outcome: ABI values encode/decode canonically under explicit resource limits.

Deliverables:

- First-party head/tail encoding and strict decoding with offset, overlap, padding, depth, count, and allocation checks.

Verification:

- Official/reference vectors, malformed-offset fuzzing, round trips.

Exit criteria:

- ABI values encode/decode canonically under explicit resource limits.
- `v0.121.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.122.0 - Artifact And Metadata Ingestion

Status: planned.

Goal: deliver the Artifact And Metadata Ingestion release with this required outcome: Common build artifacts enter the SDK through validated owned models.

Deliverables:

- Bounded JSON ingestion for ABI, bytecode, deployed bytecode, link references, compiler metadata, and source maps.

Verification:

- Foundry/Hardhat/Solc artifact corpus and hostile JSON tests.

Exit criteria:

- Common build artifacts enter the SDK through validated owned models.
- `v0.122.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.123.0 - Contract Macros And Code Generation

Status: planned.

Goal: deliver the Contract Macros And Code Generation release with this required outcome: Users can obtain typed bindings without hand-written field glue.

Deliverables:

- Audited procedural/codegen path for typed calls, returns, events, errors, and contract interfaces.

Verification:

- Compile tests, generated-code snapshots, semver and size checks.

Exit criteria:

- Users can obtain typed bindings without hand-written field glue.
- `v0.123.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.124.0 - Deployment And Linking

Status: planned.

Goal: deliver the Deployment And Linking release with this required outcome: Contracts and libraries can be deployed through the validated transaction lifecycle.

Deliverables:

- Constructor encoding, library linking, CREATE/CREATE2 address prediction, deployment simulation, broadcast, and receipt verification.

Verification:

- Local-node deployments and link-reference negative tests.

Exit criteria:

- Contracts and libraries can be deployed through the validated transaction lifecycle.
- `v0.124.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.125.0 - Events Filters And Reorg Streams

Status: planned.

Goal: deliver the Events Filters And Reorg Streams release with this required outcome: Event consumers can resume and handle reorganizations correctly.

Deliverables:

- Typed event decoding, indexed topics, filter builders, log pagination, subscriptions, removed-log/reorg semantics, and checkpoints.

Verification:

- Local-node reorg/filter tests and malformed log fixtures.

Exit criteria:

- Event consumers can resume and handle reorganizations correctly.
- `v0.125.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.126.0 - Errors Multicall And Overrides

Status: planned.

Goal: deliver the Errors Multicall And Overrides release with this required outcome: Common read/simulation workflows are typed and diagnostically complete.

Deliverables:

- Custom error registry, revert decoding, Multicall workflows, batched calls, state/block overrides, and per-call evidence.

Verification:

- Contract fixture suite, partial-failure and ambiguity tests.

Exit criteria:

- Common read/simulation workflows are typed and diagnostically complete.
- `v0.126.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.127.0 - Token And NFT Standards

Status: planned.

Goal: deliver the Token And NFT Standards release with this required outcome: Common asset interactions are available without assuming compliant return behavior.

Deliverables:

- Typed ERC-20, ERC-721, ERC-1155, metadata, approval, safe-transfer, and interface-detection helpers.

Verification:

- Reference contract integration and nonconforming-token fixtures.

Exit criteria:

- Common asset interactions are available without assuming compliant return behavior.
- `v0.127.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.128.0 - ENS Permit And Signature Standards

Status: planned.

Goal: deliver the ENS Permit And Signature Standards release with this required outcome: Naming and permit workflows are first-class and domain-safe.

Deliverables:

- ENS resolution, namehash, reverse records, EIP-2612/permit variants, EIP-1271, EIP-6492, and supported signature wrappers.

Verification:

- Mainnet-fork/local fixtures and cross-standard domain tests.

Exit criteria:

- Naming and permit workflows are first-class and domain-safe.
- `v0.128.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.129.0 - Contract Tooling Hardening

Status: planned.

Goal: deliver the Contract Tooling Hardening release with this required outcome: Contract tooling is stable enough for production SDK use.

Deliverables:

- ABI/artifact/codegen fuzzing, binding ergonomics review, compatibility matrix, examples, and independent pentest.

Verification:

- Full contract suite, package/docs checks, clean retest.

Exit criteria:

- Contract tooling is stable enough for production SDK use.
- `v0.129.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 14: Storage Canonical Chain And Client Runtime

### v0.130.0 - Database Traits And Schema

Status: planned.

Goal: deliver the Database Traits And Schema release with this required outcome: Higher layers depend on a first-party storage contract, not one database API.

Deliverables:

- Transactional key-value traits, column/schema identifiers, versioning, iterators, snapshots, durability capabilities, and explicit error contracts.

Verification:

- In-memory conformance backend, crash/error injection, no_std trait checks.

Exit criteria:

- Higher layers depend on a first-party storage contract, not one database API.
- `v0.130.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.131.0 - Chain Content Stores

Status: planned.

Goal: deliver the Chain Content Stores release with this required outcome: Canonical chain content can be retained and queried consistently.

Deliverables:

- Headers, bodies, transactions, receipts, total difficulty, execution requests, blobs, and hash/number indexes.

Verification:

- Round trips, corruption detection, incomplete-write tests.

Exit criteria:

- Canonical chain content can be retained and queried consistently.
- `v0.131.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.132.0 - State Trie Flat State And Indexes

Status: planned.

Goal: deliver the State Trie Flat State And Indexes release with this required outcome: Persisted state representations have explicit consistency invariants.

Deliverables:

- Account/storage trie nodes, code, flat state, changesets, history indexes, and root/version association.

Verification:

- Root reconstruction, index consistency, corruption fixtures.

Exit criteria:

- Persisted state representations have explicit consistency invariants.
- `v0.132.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.133.0 - Atomic Batches And Crash Consistency

Status: planned.

Goal: deliver the Atomic Batches And Crash Consistency release with this required outcome: A committed block is either fully durable or detectably absent.

Deliverables:

- Multi-column atomic commits, write-ahead/recovery contract, idempotent replay, and partial-transition detection.

Verification:

- Process-kill and torn-write simulations across admitted backends.

Exit criteria:

- A committed block is either fully durable or detectably absent.
- `v0.133.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.134.0 - Migrations Snapshots And Cache Policy

Status: planned.

Goal: deliver the Migrations Snapshots And Cache Policy release with this required outcome: Storage upgrades and restores are reproducible and fail closed.

Deliverables:

- Forward migrations, rollback limits, snapshot import/export, cache sizing/eviction, and schema compatibility reports.

Verification:

- Upgrade/downgrade fixtures, snapshot checksums, cache pressure tests.

Exit criteria:

- Storage upgrades and restores are reproducible and fail closed.
- `v0.134.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.135.0 - Pruning Archive And History Expiry

Status: planned.

Goal: deliver the Pruning Archive And History Expiry release with this required outcome: Operators know exactly which historical guarantees each mode provides.

Deliverables:

- Configurable archive/pruned modes, retention proofs, ancient/history separation, expiry scheduling, and explicit unavailable-data errors.

Verification:

- Long-chain simulation, prune/reorg interactions, historical query tests.

Exit criteria:

- Operators know exactly which historical guarantees each mode provides.
- `v0.135.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.136.0 - Canonical Import And Reorg

Status: planned.

Goal: deliver the Canonical Import And Reorg release with this required outcome: Canonical chain changes preserve state and index consistency.

Deliverables:

- Block import pipeline, validation stages, total-difficulty/fork-choice inputs, canonical indexes, unwind, and re-execution.

Verification:

- Competing-chain and deep-reorg simulations, crash recovery.

Exit criteria:

- Canonical chain changes preserve state and index consistency.
- `v0.136.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.137.0 - Heads Fork Choice And Orphans

Status: planned.

Goal: deliver the Heads Fork Choice And Orphans release with this required outcome: Head state is explicit and cannot advance through invalid ancestry.

Deliverables:

- Unsafe/safe/finalized heads, orphan queues, ancestry checks, invalid ancestors, checkpoint constraints, and chain events.

Verification:

- Engine/fork-choice sequences and orphan/finality property tests.

Exit criteria:

- Head state is explicit and cannot advance through invalid ancestry.
- `v0.137.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.138.0 - Payload Orchestration And Invalidation

Status: planned.

Goal: deliver the Payload Orchestration And Invalidation release with this required outcome: Payload work terminates consistently under reorgs and invalid blocks.

Deliverables:

- Payload build/import state machines, optimistic execution, invalidation propagation, cancellation, and cache cleanup.

Verification:

- Engine API sequence fixtures, concurrent invalidation tests.

Exit criteria:

- Payload work terminates consistently under reorgs and invalid blocks.
- `v0.138.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.139.0 - Operational Client Runtime

Status: planned.

Goal: deliver the Operational Client Runtime release with this required outcome: Node-adjacent services have a coherent lifecycle and observable failure model.

Deliverables:

- Supervised bounded tasks for import, providers, peers, sync, pruning, metrics, shutdown, and restart without imposing one async runtime on core crates.

Verification:

- Failure injection, graceful shutdown/restart, resource-cap tests.

Exit criteria:

- Node-adjacent services have a coherent lifecycle and observable failure model.
- `v0.139.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.140.0 - Storage And Client Performance Gate

Status: planned.

Goal: deliver the Storage And Client Performance Gate release with this required outcome: Storage/client foundations meet documented correctness and operational budgets.

Deliverables:

- Benchmark import, state access, roots, reorgs, snapshots, pruning, memory, disk amplification, and startup recovery.

Verification:

- Reproducible hardware profile and regression thresholds.

Exit criteria:

- Storage/client foundations meet documented correctness and operational budgets.
- `v0.140.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 15: Consensus Engine And Light Client

### v0.141.0 - SSZ Foundational Codec And Merkleization

Status: planned.

Goal: establish the immutable SSZ wire and Merkle foundation required by
light-client and protocol-type work without claiming the later mutable,
cached, full-client surface.

Deliverables:

- First-party basic and composite SSZ type rules;
- bounded canonical encode/decode;
- offset validation;
- generalized indices;
- baseline Merkleization, branches, and hash-tree roots;
- explicit exclusions for incremental mutation, cached trees, and
  multiproofs assigned to `v0.191.0`.

Verification:

- Official consensus-spec vectors;
- malformed-offset fuzzing;
- baseline root differentials;
- cross-check that later full-client APIs cannot be inferred from this
  foundational release.

Exit criteria:

- Immutable consensus objects can be encoded, decoded, rooted, and proven
  without external SSZ core logic, while mutable production operations remain
  explicitly assigned to `v0.191.0`.
- `v0.141.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.142.0 - Beacon Types And Fork Domains

Status: planned.

Goal: deliver the Beacon Types And Fork Domains release with this required outcome: Consensus data has complete owned/borrowed/fork-aware models.

Deliverables:

- Fork-versioned beacon blocks, states, execution payloads, withdrawals, blobs/data columns, requests, domains, and signing roots.

Verification:

- Official consensus fixtures across all claimed forks.

Exit criteria:

- Consensus data has complete owned/borrowed/fork-aware models.
- `v0.142.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.143.0 - Engine API Types And Validation

Status: planned.

Goal: deliver the Engine API Types And Validation release with this required outcome: Engine messages are fully typed and version/fork checked.

Deliverables:

- All pinned Engine API versions, payload attributes, execution status, capabilities, transition configuration, and strict validation.

Verification:

- execution-apis fixtures and client interoperability snapshots.

Exit criteria:

- Engine messages are fully typed and version/fork checked.
- `v0.143.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.144.0 - Engine Transport And Protocol Boundary

Status: planned.

Goal: define and test the reusable Engine API protocol and authenticated
transport boundary without claiming beacon-node coordination policy.

Deliverables:

- Runtime-neutral Engine client/server traits;
- authenticated transport adapter;
- request/response sequencing primitives;
- idempotency, cancellation, timeout, and error mapping;
- explicit statement that beacon fork-choice and payload orchestration belong
  to the Beacon Engine Coordinator at `v0.226.0`.

Verification:

- Protocol sequence tests;
- JWT and redaction review;
- transport conformance tests independent of beacon-node policy.

Exit criteria:

- Engine messages can travel through an authenticated, runtime-neutral
  boundary in either embedding direction without assigning beacon-node
  coordination ownership to this layer.
- `v0.144.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.145.0 - Beacon API Provider Client

Status: planned.

Goal: provide a typed outbound Beacon API client/provider boundary without
claiming the later beacon-node server implementation.

Deliverables:

- Typed outbound Beacon REST methods;
- event-stream client handling;
- pagination and version negotiation;
- finality, blob, and data-column responses;
- bounded transport policy;
- explicit server-side ownership assigned to `v0.232.0`.

Verification:

- Beacon API client fixtures;
- local independent consensus-client integration;
- compile and documentation checks separating provider and server roles.

Exit criteria:

- Consensus data can be acquired through a production typed provider
  boundary, while serving the Beacon API remains a distinct beacon-node
  responsibility.
- `v0.145.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.146.0 - Light Client Bootstrap And Weak Subjectivity

Status: planned.

Goal: deliver the Light Client Bootstrap And Weak Subjectivity release with this required outcome: A light client starts only from explicit, valid trust roots.

Deliverables:

- Trusted checkpoint/bootstrap validation, fork/genesis binding, weak-subjectivity periods, stale-checkpoint rejection, and clock policy.

Verification:

- Official bootstrap vectors and stale/adversarial checkpoint tests.

Exit criteria:

- A light client starts only from explicit, valid trust roots.
- `v0.146.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.147.0 - BLS Sync Committee Verification

Status: planned.

Goal: deliver the BLS Sync Committee Verification release with this required outcome: Sync committee attestations are cryptographically verified first party or through an audited explicit backend.

Deliverables:

- Aggregate BLS signatures, participant bits, signing domains, committee membership, and cryptographic backend admission.

Verification:

- Official BLS/light-client vectors, malformed/subgroup fuzzing.

Exit criteria:

- Sync committee attestations are cryptographically verified first party or through an audited explicit backend.
- `v0.147.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.148.0 - Committee Rotation And Persistence

Status: planned.

Goal: deliver the Committee Rotation And Persistence release with this required outcome: Trust state survives rotation and restart without accepting stale committees.

Deliverables:

- Period transitions, next-committee proofs, durable store, rollback/recovery, fork upgrades, and checkpoint export.

Verification:

- Multi-period vectors, crash/restart tests, conflicting-update cases.

Exit criteria:

- Trust state survives rotation and restart without accepting stale committees.
- `v0.148.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.149.0 - Finality Optimistic Scoring And Misbehavior

Status: planned.

Goal: deliver the Finality Optimistic Scoring And Misbehavior release with this required outcome: Update selection and finality are deterministic under conflicting inputs.

Deliverables:

- Update ranking, optimistic/finalized headers, participation thresholds, duplicate/conflict handling, and misbehavior evidence.

Verification:

- Official update-processing vectors and Byzantine peer simulations.

Exit criteria:

- Update selection and finality are deterministic under conflicting inputs.
- `v0.149.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.150.0 - Execution Proof Binding

Status: planned.

Goal: deliver the Execution Proof Binding release with this required outcome: Verified RPC/state evidence can anchor to a light-client trust root.

Deliverables:

- Bind finalized beacon execution payload roots to execution headers/state/receipts through SSZ and MPT proofs.

Verification:

- End-to-end consensus-to-execution proof fixtures.

Exit criteria:

- Verified RPC/state evidence can anchor to a light-client trust root.
- `v0.150.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.151.0 - Checkpoint Recovery And Multi-Source Acquisition

Status: planned.

Goal: deliver the Checkpoint Recovery And Multi-Source Acquisition release with this required outcome: Light-client operation can recover without silently replacing its trust root.

Deliverables:

- Multiple bootstrap/update sources, quorum/evidence, stale-source isolation, checkpoint rotation, and recovery procedures.

Verification:

- Offline/recovery and malicious-source simulations.

Exit criteria:

- Light-client operation can recover without silently replacing its trust root.
- `v0.151.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.152.0 - Complete Light-Client Conformance

Status: planned.

Goal: deliver the Complete Light-Client Conformance release with this required outcome: Complete light-client claims are fixture-backed and operationally documented.

Deliverables:

- Run all official light-client suites for historical/current forks and publish support/skip evidence.

Verification:

- Generated conformance report with zero unexplained skips.

Exit criteria:

- Complete light-client claims are fixture-backed and operationally documented.
- `v0.152.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.153.0 - PeerDAS Threat Model And Admission Plan

Status: planned.

Goal: define PeerDAS trust, cryptographic, custody, sampling, networking, and
resource requirements before implementation begins.

Deliverables:

- PeerDAS threat model;
- data-column and sampling boundaries;
- custody policy;
- cryptographic and trusted-setup requirements;
- CPU, memory, bandwidth, and retention ceilings;
- versioned implementation assignments beginning at `v0.193.0`;
- fail-closed rules that prevent this planning release from implying an
  executable PeerDAS implementation.

Verification:

- Review against pinned PeerDAS/current-fork specifications;
- abuse-case and dependency review;
- traceability check proving every admitted requirement has a later release.

Exit criteria:

- PeerDAS implementation cannot begin with ambiguous trust, cryptographic,
  custody, networking, or resource boundaries, and no consumer can claim
  support before the `v0.193.0` core exists.
- `v0.153.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 16: Networking Txpool And Synchronization

### v0.154.0 - Networking Threat And Dependency Gate

Status: planned.

Goal: deliver the Networking Threat And Dependency Gate release with this required outcome: No live peer code lands before trust and resource boundaries are approved.

Deliverables:

- Protocol threat model, crypto/transport dependency review, identity/key policy, resource ceilings, and wire-spec locks.

Verification:

- cargo-deny/audit, protocol corpus plan, architecture review.

Exit criteria:

- No live peer code lands before trust and resource boundaries are approved.
- `v0.154.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.155.0 - Discovery And RLPx

Status: planned.

Goal: deliver the Discovery And RLPx release with this required outcome: Peers can be discovered and authenticated through bounded first-party protocol logic.

Deliverables:

- Discovery v4/v5 as admitted, ENR, node records, handshakes, framing, capabilities, encryption/MAC, and replay protections.

Verification:

- Official/reference vectors, packet/frame fuzzing, interoperability tests.

Exit criteria:

- Peers can be discovered and authenticated through bounded first-party protocol logic.
- `v0.155.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.156.0 - Eth Protocol Messages

Status: planned.

Goal: deliver the Eth Protocol Messages release with this required outcome: Execution chain data can be exchanged through typed wire messages.

Deliverables:

- Status negotiation and all admitted `eth` protocol request/response/announcement messages with fork capability checks.

Verification:

- Cross-client devp2p fixtures and malformed message fuzzing.

Exit criteria:

- Execution chain data can be exchanged through typed wire messages.
- `v0.156.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.157.0 - Snap Protocol

Status: planned.

Goal: deliver the Snap Protocol release with this required outcome: Snapshot data is validated before storage or state promotion.

Deliverables:

- Account ranges, storage ranges, bytecodes, trie nodes, proofs, continuation rules, and response limits.

Verification:

- Cross-client fixtures, proof verification, response-bomb tests.

Exit criteria:

- Snapshot data is validated before storage or state promotion.
- `v0.157.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.158.0 - Peer Service

Status: planned.

Goal: deliver the Peer Service release with this required outcome: Peer selection and isolation are explicit and bounded.

Deliverables:

- Peer lifecycle, capability scoring, quotas, diversity, bans, disconnect reasons, persistence, and metrics.

Verification:

- Churn/eclipse/flood simulations and restart tests.

Exit criteria:

- Peer selection and isolation are explicit and bounded.
- `v0.158.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.159.0 - Request Scheduler And Backpressure

Status: planned.

Goal: deliver the Request Scheduler And Backpressure release with this required outcome: Network work cannot create unbounded queues or retry amplification.

Deliverables:

- Correlated requests, deadlines, retries, per-peer/global budgets, cancellation, fair scheduling, and invalid-response penalties.

Verification:

- Loss/reorder/timeout/load simulations and race tests.

Exit criteria:

- Network work cannot create unbounded queues or retry amplification.
- `v0.159.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.160.0 - Transaction Pool

Status: planned.

Goal: deliver the Transaction Pool release with this required outcome: Pending transaction policy is deterministic and resource bounded.

Deliverables:

- Stateless/stateful admission, replacement, nonce gaps, blob/set-code policy, eviction, reorg reinjection, persistence boundary, and propagation.

Verification:

- Client differential cases, adversarial pool loads, reorg tests.

Exit criteria:

- Pending transaction policy is deterministic and resource bounded.
- `v0.160.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.161.0 - Sync Orchestration

Status: planned.

Goal: deliver the Sync Orchestration release with this required outcome: Sync progresses or fails with explicit recoverable state.

Deliverables:

- Header/body/receipt/state stages, checkpoints, progress persistence, invalidation, restart, and strategy selection.

Verification:

- Interrupted sync and competing-chain simulations.

Exit criteria:

- Sync progresses or fails with explicit recoverable state.
- `v0.161.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.162.0 - Multi-Peer Full And Snap Sync

Status: planned.

Goal: deliver the Multi-Peer Full And Snap Sync release with this required outcome: A node can reach verified canonical state without trusting one peer.

Deliverables:

- Peer assignment, proof-backed ranges, healing, pivot changes, finalization, and canonical import integration.

Verification:

- Multi-client local network, malicious peer, restart, and reorg tests.

Exit criteria:

- A node can reach verified canonical state without trusting one peer.
- `v0.162.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.163.0 - Portal And Historical Data Acquisition

Status: planned.

Goal: deliver the Portal And Historical Data Acquisition release with this required outcome: Expired historical data has an explicit verified acquisition path.

Deliverables:

- Portal/history network boundary, content keys, proofs, provider fallback, history-expiry aware retrieval, and provenance.

Verification:

- Portal/reference fixtures and unavailable/corrupt source tests.

Exit criteria:

- Expired historical data has an explicit verified acquisition path.
- `v0.163.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.164.0 - Builder Validator And Network Hardening

Status: planned.

Goal: deliver the Builder Validator And Network Hardening release with this required outcome: Networking, sync, and node-adjacent boundaries are production candidates.

Deliverables:

- Payload-builder/validator boundaries, gossip/pool interactions, load/DoS suite, interoperability matrix, and pentest.

Verification:

- Cross-client network matrix and clean retest.

Exit criteria:

- Networking, sync, and node-adjacent boundaries are production candidates.
- `v0.164.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 17: Statelessness Commitment Evolution And Future Forks

### v0.165.0 - Proof Format Abstraction

Status: planned.

Goal: deliver the Proof Format Abstraction release with this required outcome: MPT is no longer hardwired into every proof consumer.

Deliverables:

- Commitment/proof traits, domain-separated roots/keys, batch proofs, capability negotiation, and migration-safe evidence types.

Verification:

- Backend conformance suite and domain-substitution compile tests.

Exit criteria:

- MPT is no longer hardwired into every proof consumer.
- `v0.165.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.166.0 - Execution Witness Model

Status: planned.

Goal: deliver the Execution Witness Model release with this required outcome: State dependencies of execution can be represented explicitly.

Deliverables:

- Owned/borrowed witness data for accounts, storage, code, block context, accesses, writes, and missing-node diagnostics.

Verification:

- Canonical encoding, limit tests, mutation/property tests.

Exit criteria:

- State dependencies of execution can be represented explicitly.
- `v0.166.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.167.0 - MPT Witness Construction And Verification

Status: planned.

Goal: deliver the MPT Witness Construction And Verification release with this required outcome: MPT-backed execution inputs can be proven complete.

Deliverables:

- Build minimal MPT witnesses, verify completeness/correctness, deduplicate nodes, and bind them to roots and transactions/blocks.

Verification:

- State-test derived witnesses, omission/substitution fuzzing.

Exit criteria:

- MPT-backed execution inputs can be proven complete.
- `v0.167.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.168.0 - Stateless Execution

Status: planned.

Goal: deliver the Stateless Execution release with this required outcome: Claimed execution can run without a full local state database.

Deliverables:

- Execute transactions/blocks from witnesses, reject missing/extraneous invalid evidence, and emit post-state commitments/deltas.

Verification:

- Stateful-versus-stateless differential fixtures.

Exit criteria:

- Claimed execution can run without a full local state database.
- `v0.168.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.169.0 - Verkle Or Successor Commitment Boundary

Status: planned.

Goal: deliver the Verkle Or Successor Commitment Boundary release with this required outcome: Future state commitments fit the shared proof model without pretending unfinished cryptography is implemented.

Deliverables:

- First-party format/rule model and audited cryptographic backend boundary for the officially selected successor commitment scheme.

Verification:

- Pinned official vectors and backend-admission review.

Exit criteria:

- Future state commitments fit the shared proof model without pretending unfinished cryptography is implemented.
- `v0.169.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.170.0 - Successor Commitment Backend

Status: planned.

Goal: deliver the Successor Commitment Backend release with this required outcome: The selected successor proof scheme is cryptographically executable.

Deliverables:

- Implement/admit polynomial/vector commitment arithmetic, key mapping, proof creation/verification, and canonical serialization required by the selected fork.

Verification:

- Official and independent vectors, differential checks, performance/pentest.

Exit criteria:

- The selected successor proof scheme is cryptographically executable.
- `v0.170.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.171.0 - Successor Witness And State Integration

Status: planned.

Goal: deliver the Successor Witness And State Integration release with this required outcome: Historical MPT and successor states coexist with explicit fork rules.

Deliverables:

- Trie/state migration, witness generation, stateless execution, sync/storage, and root validation for the successor scheme.

Verification:

- Transition and mixed-era fixtures, crash/reorg tests.

Exit criteria:

- Historical MPT and successor states coexist with explicit fork rules.
- `v0.171.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.172.0 - State Expiry And Address Evolution

Status: planned.

Goal: deliver the State Expiry And Address Evolution release with this required outcome: State-lifecycle evolution is implemented when specified, not left as an architectural surprise.

Deliverables:

- Implement officially adopted state-expiry, address-extension, resurrection, access-list, or migration rules with archive/provider policy.

Verification:

- Official fork fixtures and long-horizon state simulations.

Exit criteria:

- State-lifecycle evolution is implemented when specified, not left as an architectural surprise.
- `v0.172.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.173.0 - ZK Execution Proof Boundary

Status: planned.

Goal: deliver the ZK Execution Proof Boundary release with this required outcome: ZK proof systems can integrate without becoming an implicit consensus dependency.

Deliverables:

- Versioned proof/public-input types, verifier trait, fork/circuit binding, recursion/batch policy, and explicit trust/error evidence.

Verification:

- Mock and admitted verifier conformance, malformed proof corpus.

Exit criteria:

- ZK proof systems can integrate without becoming an implicit consensus dependency.
- `v0.173.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.174.0 - Future Fork Automation

Status: planned.

Goal: deliver the Future Fork Automation release with this required outcome: New hard forks cannot silently outrun the support matrix.

Deliverables:

- Monitor official specs/EIPs/fixtures, generate drift reports and candidate manifests, and require a named maintenance release for every adopted change.

Verification:

- Scheduled checker tests and simulated upstream fork changes.

Exit criteria:

- New hard forks cannot silently outrun the support matrix.
- `v0.174.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 18: Foundation Assurance Before Full Consensus Client

### v0.175.0 - Platform And Target Matrix

Status: planned.

Goal: deliver the Platform And Target Matrix release with this required outcome: Every promised platform has repeatable evidence or an explicit limitation.

Deliverables:

- Linux, Windows, BSD, macOS, Android, iOS, WASM where applicable, big/little-endian review, and Aesynx-readiness constraints.

Verification:

- Cross-target builds/tests and documented unsupported combinations.

Exit criteria:

- Every promised platform has repeatable evidence or an explicit limitation.
- `v0.175.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.176.0 - Whole-System Performance Program

Status: planned.

Goal: deliver the Whole-System Performance Program release with this required outcome: Performance and DoS budgets are release-blocking rather than anecdotal.

Deliverables:

- Benchmarks and budgets for codec, crypto, EVM, proofs, providers, storage, sync, ABI, wallets, and end-to-end workflows.

Verification:

- Reproducible benchmark runner and regression thresholds.

Exit criteria:

- Performance and DoS budgets are release-blocking rather than anecdotal.
- `v0.176.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.177.0 - Kani Codec Primitive And Typestate Proofs

Status: planned.

Goal: deliver the Kani Codec Primitive And Typestate Proofs release with this required outcome: Selected foundational invariants have machine-checked evidence in addition to tests.

Deliverables:

- Bounded proofs for arithmetic, canonical decoding, budget accounting, writers, conversions, and impossible typestate transitions.

Verification:

- Pinned Kani toolchain and reproducible proof report.

Exit criteria:

- Selected foundational invariants have machine-checked evidence in addition to tests.
- `v0.177.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.178.0 - Kani EVM Trie And State Proofs

Status: planned.

Goal: deliver the Kani EVM Trie And State Proofs release with this required outcome: Selected consensus-critical execution invariants have machine-checked evidence.

Deliverables:

- Bounded proofs for stack/gas/memory arithmetic, journal rollback, trie paths, proof verification, and state-transition invariants.

Verification:

- Reproducible proof harnesses with documented bounds/assumptions.

Exit criteria:

- Selected consensus-critical execution invariants have machine-checked evidence.
- `v0.178.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.179.0 - Miri Sanitizers And Undefined-Behavior Gate

Status: planned.

Goal: deliver the Miri Sanitizers And Undefined-Behavior Gate release with this required outcome: Dynamic memory/UB evidence complements the first-party unsafe-code ban.

Deliverables:

- Miri for applicable crates/tests, address/thread/memory sanitizers where supported, stack-use analysis, and unsafe-dependency review.

Verification:

- Reproducible tool reports and zero unexplained failures.

Exit criteria:

- Dynamic memory/UB evidence complements the first-party unsafe-code ban.
- `v0.179.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.180.0 - Compatibility And Semver Gate

Status: planned.

Goal: deliver the Compatibility And Semver Gate release with this required outcome: Accidental breaking or stale publication metadata blocks release.

Deliverables:

- cargo-semver-checks, feature powerset, minimal/default/all-feature graphs, README dependency versions, serde/text snapshots, and MSRV/stable checks.

Verification:

- Automated compatibility report for every published crate.

Exit criteria:

- Accidental breaking or stale publication metadata blocks release.
- `v0.180.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.181.0 - Task-Oriented Documentation

Status: planned.

Goal: deliver the Task-Oriented Documentation release with this required outcome: Public functionality is discoverable without reading internal source.

Deliverables:

- Complete guides for decoding, verification, execution, providers, wallets, contracts, storage, light clients, sync, stateless operation, and migration.

Verification:

- Doctests, link checks, fresh-user task exercises.

Exit criteria:

- Public functionality is discoverable without reading internal source.
- `v0.181.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.182.0 - Core SDK API Stability Baseline

Status: planned.

Goal: deliver the Core SDK API Stability Baseline release with this required outcome: Later consensus-client work builds on deliberate foundation contracts without pretending the complete 1.0 API is frozen.

Deliverables:

- Stabilize naming, ownership, errors, features, deprecation, compatibility, and migration policy for the core, SDK, execution, provider, wallet, contract, storage, light-client, and networking foundations.

Verification:

- Public API review and semver baseline for admitted foundation crates.

Exit criteria:

- Later consensus-client work builds on deliberate foundation contracts without pretending the complete 1.0 API is frozen.
- `v0.182.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.183.0 - Core Cryptography And Codec Audit

Status: planned.

Goal: deliver the Core Cryptography And Codec Audit release with this required outcome: No unresolved critical/high core finding remains.

Deliverables:

- Independent review of primitives, hashing, signatures, BLS/KZG, RLP/SSZ/ABI, proofs, and sanitization boundaries.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high core finding remains.
- `v0.183.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.184.0 - Execution Storage And Light-Client Audit

Status: planned.

Goal: deliver the Execution Storage And Light-Client Audit release with this required outcome: No unresolved critical/high finding remains in the execution/client foundation or light-client scope.

Deliverables:

- Independent review of EVM, execution state transition, tries, storage, execution fork choice, Engine boundaries, light-client paths, and stateless execution.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high finding remains in the execution/client foundation or light-client scope.
- `v0.184.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.185.0 - Provider Wallet And Contract Audit

Status: planned.

Goal: deliver the Provider Wallet And Contract Audit release with this required outcome: No unresolved critical/high SDK or key-management finding remains.

Deliverables:

- Independent review of transports, trust layers, transaction lifecycle, signers, keystores, account abstraction, ABI, and codegen.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high SDK or key-management finding remains.
- `v0.185.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.186.0 - Execution Networking Sync And Runtime Audit

Status: planned.

Goal: deliver the Execution Networking Sync And Runtime Audit release with this required outcome: No unresolved critical/high finding remains in the execution-network or runtime foundation.

Deliverables:

- Independent review of RLPx/discovery, execution peer management, txpool, execution sync, runtime supervision, pruning, and operational DoS controls.

Verification:

- Published scope/report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high finding remains in the execution-network or runtime foundation.
- `v0.186.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.187.0 - Foundation Remediation Release

Status: planned.

Goal: deliver the Foundation Remediation Release release with this required outcome: The SDK, execution, storage, light-client, and execution-network foundation is ready to host the full consensus client.

Deliverables:

- Resolve residual foundation audit, conformance, compatibility, documentation, and performance findings; record accepted low risks.

Verification:

- Full gate, all foundation retests, zero unexplained skips, updated SBOM/provenance.

Exit criteria:

- The SDK, execution, storage, light-client, and execution-network foundation is ready to host the full consensus client.
- `v0.187.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.188.0 - Full-Stack Foundation Integration Baseline

Status: planned.

Goal: deliver the Full-Stack Foundation Integration Baseline release with this required outcome: Full beacon-node and validator work starts from a reviewed integrated foundation rather than an assumed 1.0 candidate.

Deliverables:

- Exercise all foundation layers together, lock component interoperability contracts, rehearse publication order, and publish the pre-consensus-client evidence set.

Verification:

- Exact-candidate pentest, reproducible packages, local execution-node matrix, green CI.

Exit criteria:

- Full beacon-node and validator work starts from a reviewed integrated foundation rather than an assumed 1.0 candidate.
- `v0.188.0 implementation stop reached. Run pentest for this exact
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

### v0.189.0 - Consensus Client Architecture And Threat Model

Status: planned.

Goal: deliver the Consensus Client Architecture And Threat Model release with this required outcome: No consensus-client implementation begins with ambiguous ownership, trust, or persistence boundaries.

Deliverables:

- Define beacon-node, validator, signer, slashing, builder, Engine, storage, network, sync, and data-availability trust boundaries; allocate focused crates and resource ceilings.

Verification:

- Architecture review, dependency classification, abuse-case register, crate-cycle check.

Exit criteria:

- No consensus-client implementation begins with ambiguous ownership, trust, or persistence boundaries.
- `v0.189.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.190.0 - Consensus Configuration And Fork Registry

Status: planned.

Goal: deliver the Consensus Configuration And Fork Registry release with this required outcome: Consensus behavior is source-generated and fork-modular rather than spread through optional-field conditionals.

Deliverables:

- Network presets, chain configurations, genesis data, fork schedules, fork digests, domain constants, typed stable-fork variants, and generated experimental-fork modules.

Verification:

- Official preset/config fixtures, fork-digest vectors, source-lock drift tests.

Exit criteria:

- Consensus behavior is source-generated and fork-modular rather than spread through optional-field conditionals.
- `v0.190.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.191.0 - Complete SSZ Client Surface

Status: planned.

Goal: extend the immutable `v0.141.0` SSZ foundation into the complete mutable,
cached, proof-capable surface required by production beacon state and
networking.

Deliverables:

- Complete production containers, lists, vectors, bitlists, and bitvectors;
- incremental hash-tree roots;
- mutable generalized-index operations;
- branches and multiproofs;
- cached trees and cache-invalidation rules;
- bounded transactional mutation APIs;
- compatibility with the canonical encoding and baseline roots from
  `v0.141.0`.

Verification:

- Official SSZ vectors;
- incremental-versus-full-root differential tests;
- cache invalidation and mutation rollback tests;
- malformed-offset and proof fuzzing;
- compatibility tests against `v0.141.0` encodings and roots.

Exit criteria:

- Beacon state and network objects can use first-party SSZ without missing
  production mutation, caching, container, or proof operations, and without
  redefining the foundational codec.
- `v0.191.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.192.0 - BLS Signing Aggregation And Batch Verification

Status: planned.

Goal: deliver the BLS Signing Aggregation And Batch Verification release with this required outcome: Consensus and validator paths have a complete first-party BLS surface, not verification-only light-client hooks.

Deliverables:

- First-party BLS signing, verification, aggregation, aggregate verification, randomized batch verification, proof of possession where required, secret/public key domains, and optional acceleration only behind audited adapters.

Verification:

- Official BLS vectors, independent differential tests, subgroup/fault/batch fuzzing, timing review.

Exit criteria:

- Consensus and validator paths have a complete first-party BLS surface, not verification-only light-client hooks.
- `v0.192.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.193.0 - PeerDAS Cell And Reconstruction Core

Status: planned.

Goal: implement the first-party cryptographic and erasure-coding core before
any state-transition, storage, networking, synchronization, or validator path
consumes PeerDAS data.

Deliverables:

- Blob-to-cell conversion;
- cell KZG proof creation and verification;
- data-column construction;
- erasure coding and bounded reconstruction;
- batch verification;
- bounded reusable workspaces;
- explicit acceleration/backend boundaries;
- canonical failure and partial-output behavior.

Verification:

- Official EIP-7594 and pinned current-fork vectors;
- independent differential checks;
- malformed proof, cell, and reconstruction fuzzing;
- corruption and insufficient-column tests;
- CPU, memory, and workspace ceilings;
- default-graph and backend-admission checks.

Exit criteria:

- Data columns can be created, verified, and reconstructed first party before
  any downstream milestone treats PeerDAS evidence as actionable.
- `v0.193.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.194.0 - Committees Shuffling Domains And Signing Roots

Status: planned.

Goal: deliver the Committees Shuffling Domains And Signing Roots release with this required outcome: Every duty and signature domain is derived from pinned consensus rules.

Deliverables:

- Validator shuffling, proposer/committee selection, subnet assignments, fork-aware domains, signing roots, RANDAO, aggregator selection, and cached epoch context.

Verification:

- Official shuffling/committee vectors, property tests, cross-fork domain checks.

Exit criteria:

- Every duty and signature domain is derived from pinned consensus rules.
- `v0.194.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 20: Complete Beacon State Transition

### v0.195.0 - Beacon Transition Shell And Per-Slot Processing

Status: planned.

Goal: deliver the Beacon Transition Shell And Per-Slot Processing release with this required outcome: Per-slot processing is complete and failed transitions cannot partially mutate caller-visible state.

Deliverables:

- Mutable/transactional BeaconState, slot processing, historical roots/summaries, state/block roots, cache policy, and rollback-safe transition errors.

Verification:

- Official slot-transition vectors, output-unchanged failure tests, state-root differential checks.

Exit criteria:

- Per-slot processing is complete and failed transitions cannot partially mutate caller-visible state.
- `v0.195.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.196.0 - Epoch Registry And Balance Processing

Status: planned.

Goal: deliver the Epoch Registry And Balance Processing release with this required outcome: Epoch-wide validator and balance bookkeeping matches the specification.

Deliverables:

- Registry updates, effective balances, justification/finalization inputs, slashings vectors, inactivity scores, participation rotation, and epoch caches.

Verification:

- Official epoch-component vectors and large-validator-set resource tests.

Exit criteria:

- Epoch-wide validator and balance bookkeeping matches the specification.
- `v0.196.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.197.0 - Activation Exit Churn Withdrawal And Consolidation

Status: planned.

Goal: deliver the Activation Exit Churn Withdrawal And Consolidation release with this required outcome: The full validator lifecycle is state-transition complete.

Deliverables:

- Eligibility, activation queue, exits, churn limits, withdrawals, credential changes, consolidation requests, and fork-specific lifecycle rules.

Verification:

- Official lifecycle vectors, queue/churn properties, historical/current fork tests.

Exit criteria:

- The full validator lifecycle is state-transition complete.
- `v0.197.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.198.0 - Rewards Penalties Participation And Inactivity

Status: planned.

Goal: deliver the Rewards Penalties Participation And Inactivity release with this required outcome: Balance outcomes match official vectors across normal and non-finalizing periods.

Deliverables:

- Base rewards, attestation deltas, proposer rewards, sync rewards, inactivity leaks, participation flags, and fork-specific accounting.

Verification:

- Official reward/penalty vectors, arithmetic proofs/properties, inactivity simulations.

Exit criteria:

- Balance outcomes match official vectors across normal and non-finalizing periods.
- `v0.198.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.199.0 - Deposits Slashings And Credential Operations

Status: planned.

Goal: deliver the Deposits Slashings And Credential Operations release with this required outcome: Every consensus operation that changes validator state is implemented and checked.

Deliverables:

- Deposit processing and proofs, proposer/attester slashings, voluntary exits, BLS-to-execution changes, pending requests, and duplicate/conflict policy.

Verification:

- Official operation vectors, malformed proof/signature tests, slashing edge cases.

Exit criteria:

- Every consensus operation that changes validator state is implemented and checked.
- `v0.199.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.200.0 - Attestations Sync Committees And Block Operations

Status: planned.

Goal: deliver the Attestations Sync Committees And Block Operations release with this required outcome: Beacon blocks can process all stable-fork consensus operations.

Deliverables:

- Attestation validation/processing, indexed attestations, sync aggregates, block header/body operations, operation ordering, and bounded block-operation limits.

Verification:

- Official operation/block vectors and malformed aggregate fuzzing.

Exit criteria:

- Beacon blocks can process all stable-fork consensus operations.
- `v0.200.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.201.0 - Execution Payload And Request Processing

Status: planned.

Goal: deliver the Execution Payload And Request Processing release with this required outcome: Consensus transition is correctly bound to execution validity and current request types.

Deliverables:

- Merge transition rules, execution payload/header processing, withdrawals, deposit receipts, execution requests, consolidations, payload status binding, and Engine evidence.

Verification:

- Official Bellatrix-through-current vectors and invalid execution-status cases.

Exit criteria:

- Consensus transition is correctly bound to execution validity and current request types.
- `v0.201.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.202.0 - Data Availability State Transition

Status: planned.

Goal: deliver the Data Availability State Transition release with this required outcome: Consensus transition does not accept data-dependent blocks without the required availability evidence.

Deliverables:

- Blob/data-column commitments, custody requirements, availability status, block acceptance dependencies, retention metadata, and stable-fork DA rules.

Verification:

- Official Deneb/Fulu/current vectors, missing/invalid sidecar tests, custody calculations.

Exit criteria:

- Consensus transition does not accept data-dependent blocks without the required availability evidence.
- `v0.202.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.203.0 - Explicit Consensus Fork Upgrades

Status: planned.

Goal: deliver the Explicit Consensus Fork Upgrades release with this required outcome: Every supported fork transition is explicit, tested, and free of implicit optional-field reinterpretation.

Deliverables:

- State upgrade functions between every supported stable fork, migration invariants, typed pre/post states, and experimental-fork admission policy.

Verification:

- Official fork-upgrade vectors and round-trip migration audits.

Exit criteria:

- Every supported fork transition is explicit, tested, and free of implicit optional-field reinterpretation.
- `v0.203.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.204.0 - Complete State-Transition Vector Gate

Status: planned.

Goal: deliver the Complete State-Transition Vector Gate release with this required outcome: The complete beacon state transition is fixture-backed for every claimed stable fork.

Deliverables:

- Run all official operation, epoch, transition, fork-upgrade, sanity, finality, random, and current stable-fork suites with generated coverage reports.

Verification:

- Zero unexplained skips for claimed forks and cross-client state-root samples.

Exit criteria:

- The complete beacon state transition is fixture-backed for every claimed stable fork.
- `v0.204.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 21: Production Consensus Fork Choice And Beacon Chain

### v0.205.0 - Transactional Fork-Choice Store

Status: planned.

Goal: deliver the Transactional Fork-Choice Store release with this required outcome: Fork-choice updates are transactional as required by the specification.

Deliverables:

- Store contract, atomic handlers, tick/block/attestation/slashing inputs, checkpoint states, latest messages, invalid-input rollback, and bounded caches.

Verification:

- Official handler tests, mutation-failure injection, property tests proving invalid calls preserve store state.

Exit criteria:

- Fork-choice updates are transactional as required by the specification.
- `v0.205.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.206.0 - LMD-GHOST And Latest Messages

Status: planned.

Goal: deliver the LMD-GHOST And Latest Messages release with this required outcome: Head computation matches LMD-GHOST under competing branches and votes.

Deliverables:

- Ancestry, latest-message tracking, vote weights, filtered trees, head selection, equivocation handling, and efficient incremental updates.

Verification:

- Official fork-choice vectors, randomized tree differential tests, scale benchmarks.

Exit criteria:

- Head computation matches LMD-GHOST under competing branches and votes.
- `v0.206.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.207.0 - Casper FFG Proposer Boost And Reorg Policy

Status: planned.

Goal: deliver the Casper FFG Proposer Boost And Reorg Policy release with this required outcome: Finality and proposer policies match pinned stable-fork rules.

Deliverables:

- Justified/finalized and unrealized checkpoints, proposer boost, proposer reorgs, weak-head/strong-parent rules, and configurable safety policy.

Verification:

- Official vectors, reorg-threshold properties, delayed-attestation simulations.

Exit criteria:

- Finality and proposer policies match pinned stable-fork rules.
- `v0.207.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.208.0 - Optimistic Execution And Invalidation

Status: planned.

Goal: deliver the Optimistic Execution And Invalidation release with this required outcome: Execution-invalid ancestry cannot remain canonical or authorize validator duties.

Deliverables:

- Optimistic statuses, `latestValidHash`, invalid subtree removal, weight removal, poisoning defenses, Engine failure states, and validator-safety signals.

Verification:

- Official optimistic-sync cases, execution invalidation/reorg simulations, multi-engine fault injection.

Exit criteria:

- Execution-invalid ancestry cannot remain canonical or authorize validator duties.
- `v0.208.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.209.0 - Fork-Choice Persistence And Recovery

Status: planned.

Goal: deliver the Fork-Choice Persistence And Recovery release with this required outcome: Restarted fork choice returns the same safe/finalized/head state or fails closed.

Deliverables:

- Durable fork-choice snapshots/logs, atomic import coupling, restart reconstruction, checkpoint recovery, corruption detection, and deterministic replay.

Verification:

- Crash/restart/torn-write simulations and replay equivalence tests.

Exit criteria:

- Restarted fork choice returns the same safe/finalized/head state or fails closed.
- `v0.209.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.210.0 - Beacon Operation Pools

Status: planned.

Goal: deliver the Beacon Operation Pools release with this required outcome: Block production has complete, bounded, reorg-aware operation sources.

Deliverables:

- Attestation aggregation, sync contributions, slashings, exits, credential changes, execution/consolidation requests, duplicate suppression, expiry, and block packing.

Verification:

- Pool property/fuzz tests, reorg handling, bounded-memory and packing tests.

Exit criteria:

- Block production has complete, bounded, reorg-aware operation sources.
- `v0.210.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.211.0 - Hot And Finalized Beacon Storage

Status: planned.

Goal: deliver the Hot And Finalized Beacon Storage release with this required outcome: Beacon blocks and states survive restart and finalization atomically.

Deliverables:

- Hot blocks/states/sidecars, finalized/cold storage, canonical indexes, finalized migration, root/slot lookup, and atomic fork-choice coupling.

Verification:

- Long-chain/reorg/finality/crash simulations and corruption fixtures.

Exit criteria:

- Beacon blocks and states survive restart and finalization atomically.
- `v0.211.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.212.0 - State Snapshots And Reconstruction

Status: planned.

Goal: deliver the State Snapshots And Reconstruction release with this required outcome: Required historical states can be reconstructed within documented resource bounds.

Deliverables:

- Incremental snapshots, state diffs, epoch-boundary checkpoints, historical-root/summary access, replay reconstruction, and cache eviction.

Verification:

- Random-state reconstruction, snapshot corruption, memory/disk benchmark tests.

Exit criteria:

- Required historical states can be reconstructed within documented resource bounds.
- `v0.212.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.213.0 - Sidecar Custody Pruning And Retention

Status: planned.

Goal: deliver the Sidecar Custody Pruning And Retention release with this required outcome: Data availability obligations persist correctly across restarts and pruning.

Deliverables:

- Blob/data-column storage, custody-group history, backfill markers, hot/cold retention, pruning, archive policy, and availability provenance.

Verification:

- Fulu retention/custody simulations, prune/reorg/restart tests.

Exit criteria:

- Data availability obligations persist correctly across restarts and pruning.
- `v0.213.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.214.0 - Beacon Database Migration And Repair

Status: planned.

Goal: deliver the Beacon Database Migration And Repair release with this required outcome: Beacon storage upgrades and repairs are reproducible and fail closed.

Deliverables:

- Versioned schemas, online/offline migrations, checkpoint/genesis imports, consistency scanning, repair plans, backup/restore, and operator-safe tooling contracts.

Verification:

- Multi-version migration fixtures, corruption recovery drills, checksum verification.

Exit criteria:

- Beacon storage upgrades and repairs are reproducible and fail closed.
- `v0.214.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 22: Consensus Networking And Synchronization

Consensus networking is separate from execution-layer DevP2P/RLPx. It uses the
transport and protocols required by the pinned consensus P2P specification and
must remain behind explicit optional features.

### v0.215.0 - Consensus Networking Threat And Dependency Gate

Status: planned.

Goal: deliver the Consensus Networking Threat And Dependency Gate release with this required outcome: No live consensus networking lands before its dependencies and abuse controls are approved.

Deliverables:

- libp2p/discv5/crypto/runtime dependency review, identity/key separation, protocol limits, eclipse/amplification model, and network resource budgets.

Verification:

- Dependency audit, threat-model review, transport prototype load tests.

Exit criteria:

- No live consensus networking lands before its dependencies and abuse controls are approved.
- `v0.215.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.216.0 - Discv5 ENR And Secure Transport

Status: planned.

Goal: deliver the Discv5 ENR And Secure Transport release with this required outcome: Consensus peers can be discovered and authenticated with current fork/custody metadata.

Deliverables:

- Discovery v5, ENR fork/custody fields, identity persistence, secure multiplexed transport, fork compatibility, and address policy.

Verification:

- Official/reference vectors, cross-client discovery/handshake tests, packet fuzzing.

Exit criteria:

- Consensus peers can be discovered and authenticated with current fork/custody metadata.
- `v0.216.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.217.0 - GossipSub Topics And Subnet Management

Status: planned.

Goal: deliver the GossipSub Topics And Subnet Management release with this required outcome: The node joins and leaves every required gossip domain at the correct time.

Deliverables:

- Fork-digest topics, beacon blocks, aggregate/attestation subnets, sync committees, data columns, operation topics, subscription rotation, and mesh policy.

Verification:

- Cross-client topic/subnet tests, fork-boundary simulations, bounded subscription tests.

Exit criteria:

- The node joins and leaves every required gossip domain at the correct time.
- `v0.217.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.218.0 - Staged Gossip Validation And Seen Caches

Status: planned.

Goal: deliver the Staged Gossip Validation And Seen Caches release with this required outcome: Gossip reaches pools or fork choice only after all required validation stages pass.

Deliverables:

- Decode/signature/state/fork-choice validation stages, dependency deferral, duplicate suppression, seen caches, invalid-message penalties, and no partial promotion.

Verification:

- Official gossip validation functions, malformed/future/dependency fuzzing, cache pressure tests.

Exit criteria:

- Gossip reaches pools or fork choice only after all required validation stages pass.
- `v0.218.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.219.0 - Consensus Req Resp Protocols

Status: planned.

Goal: deliver the Consensus Req Resp Protocols release with this required outcome: Required sync and serving protocols are complete and bounded.

Deliverables:

- Status, metadata, blocks, blobs, data columns, light-client data, states/checkpoints where admitted, chunk framing, context bytes, error responses, and cancellation.

Verification:

- Cross-client protocol matrix, malformed chunk fuzzing, unavailable-data cases.

Exit criteria:

- Required sync and serving protocols are complete and bounded.
- `v0.219.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.220.0 - Consensus Peer Scoring And Backpressure

Status: planned.

Goal: deliver the Consensus Peer Scoring And Backpressure release with this required outcome: Malicious or slow peers cannot create unbounded work or dominate peer selection.

Deliverables:

- Peer reputation, topic scores, custody-response scoring, bans, diversity, request budgets, rate limits, fair queues, clock disparity, and eclipse defenses.

Verification:

- Byzantine peer/flood/partition simulations, resource-ceiling benchmarks.

Exit criteria:

- Malicious or slow peers cannot create unbounded work or dominate peer selection.
- `v0.220.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.221.0 - Checkpoint And Weak-Subjectivity Sync

Status: planned.

Goal: deliver the Checkpoint And Weak-Subjectivity Sync release with this required outcome: Checkpoint sync either reaches the required anchor or terminates as a critical safety failure.

Deliverables:

- Trusted checkpoint/state acquisition, checkpoint-root enforcement, stale-checkpoint checks, fatal mismatch policy, bootstrap persistence, and provider quorum.

Verification:

- Official weak-subjectivity cases, malicious source tests, restart recovery.

Exit criteria:

- Checkpoint sync either reaches the required anchor or terminates as a critical safety failure.
- `v0.221.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.222.0 - Head And Range Sync

Status: planned.

Goal: deliver the Head And Range Sync release with this required outcome: A node reaches current head under bounded resources and adversarial peers.

Deliverables:

- Peer selection, finalized/head range download, block/sidecar validation pipeline, target updates, progress persistence, and failover.

Verification:

- Multi-peer local networks, missing/reordered/invalid range tests, restart tests.

Exit criteria:

- A node reaches current head under bounded resources and adversarial peers.
- `v0.222.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.223.0 - Finalized Backfill And State Reconstruction

Status: planned.

Goal: deliver the Finalized Backfill And State Reconstruction release with this required outcome: Historical data and states are reconstructed without weakening checkpoint trust.

Deliverables:

- Historical block/sidecar backfill, state reconstruction, checkpoint gaps, archive/history provider use, and consistency proofs.

Verification:

- Long-range backfill, pruned-provider, corruption, and interruption tests.

Exit criteria:

- Historical data and states are reconstructed without weakening checkpoint trust.
- `v0.223.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.224.0 - Optimistic Sync And Execution Recovery

Status: planned.

Goal: deliver the Optimistic Sync And Execution Recovery release with this required outcome: Optimistic progress cannot authorize duties and recovers correctly when execution rejects payloads.

Deliverables:

- Optimistic import policy, execution validation queues, `latestValidHash` invalidation, poisoned-fork recovery, reorgs, and validator-duty safety flags.

Verification:

- Official optimistic-sync scenarios and multi-execution-client fault injection.

Exit criteria:

- Optimistic progress cannot authorize duties and recovers correctly when execution rejects payloads.
- `v0.224.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.225.0 - PeerDAS Sync Custody And Backfill

Status: planned.

Goal: deliver the PeerDAS Sync Custody And Backfill release with this required outcome: Node and attached-validator custody obligations are met before availability-dependent acceptance or duties.

Deliverables:

- Custody-group calculation/history, data-column sampling, column sync/backfill, serving obligations, malicious proof handling, supernode policy, and progress persistence.

Verification:

- Official Fulu/current DA fixtures, custody changes, unavailable-column and restart tests.

Exit criteria:

- Node and attached-validator custody obligations are met before availability-dependent acceptance or duties.
- `v0.225.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 23: Engine Coordination Data Availability And Beacon Service

### v0.226.0 - Beacon Engine Coordinator

Status: planned.

Goal: build on the `v0.144.0` authenticated protocol/transport boundary and
own beacon-node fork-choice, payload-building, and execution-status
coordination policy.

Deliverables:

- Reuse the authenticated Engine transport from `v0.144.0`;
- capability negotiation;
- all supported `newPayload`, `forkchoiceUpdated`, and `getPayload` versions;
- payload-attribute construction;
- beacon fork-choice to Engine sequencing;
- timeout, retry, cancellation, and execution-status state machines;
- explicit evidence passed to block import and production services.

Verification:

- Execution-apis fixtures;
- at least two independent execution-client integrations;
- authentication, timeout, invalid-payload, and sequencing tests;
- checks proving transport concerns remain in `v0.144.0`.

Exit criteria:

- The beacon node can coordinate every claimed fork with an execution client
  through the previously admitted authenticated boundary without duplicating
  transport or JWT ownership.
- `v0.226.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.227.0 - Multi-Execution-Client Failover

Status: planned.

Goal: deliver the Multi-Execution-Client Failover release with this required outcome: Execution failover is explicit and cannot silently mix incompatible payload state.

Deliverables:

- Multiple endpoints, health/latency metrics, method/version capabilities, failover policy, disagreement evidence, circuit breaking, retry bounds, and recovery.

Verification:

- Independent execution-client outage/disagreement/latency simulations.

Exit criteria:

- Execution failover is explicit and cannot silently mix incompatible payload state.
- `v0.227.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.228.0 - Deposit Contract Tracking And Deposit Tree

Status: planned.

Goal: maintain a reorg-safe execution-layer deposit view and canonical deposit
tree for historical beacon operation.

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
- `v0.228.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.229.0 - Genesis Construction Eth1 Voting And Genesis Sync

Status: planned.

Goal: support historical `eth1_data` behavior and construct a beacon genesis
state from verified deposits for public, private, and test networks.

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
- `v0.229.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.230.0 - Availability Tracking And Block Admission

Status: planned.

Goal: deliver the Availability Tracking And Block Admission release with this required outcome: A block becomes fully available only from sufficient verified evidence under the active fork rules.

Deliverables:

- Per-block availability states, custody/sample evidence, gossip/ReqResp integration, acceptance gates, retention, invalidation, and recovery.

Verification:

- Partition, missing-column, malicious-proof, reorg, and restart simulations.

Exit criteria:

- A block becomes fully available only from sufficient verified evidence under the active fork rules.
- `v0.230.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.231.0 - Beacon Node Orchestration

Status: planned.

Goal: deliver the Beacon Node Orchestration release with this required outcome: The focused crates operate as one coherent beacon node with explicit terminal states.

Deliverables:

- Slot clock, import pipeline, transition/fork-choice/storage/network/Engine/DA coordination, operation pools, bounded task supervision, shutdown, and restart.

Verification:

- Deterministic in-process scenarios, fault injection at every subsystem boundary.

Exit criteria:

- The focused crates operate as one coherent beacon node with explicit terminal states.
- `v0.231.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.232.0 - Beacon Node REST And Event APIs

Status: planned.

Goal: deliver the Beacon Node REST And Event APIs release with this required outcome: External tooling can operate the beacon node through complete versioned server APIs.

Deliverables:

- Versioned JSON/SSZ Beacon API server, events, node identity/peers, config/spec, blocks/states, pools, light-client, debug, authentication, TLS, and rate limits.

Verification:

- Official Beacon API conformance, client compatibility, malformed/request-flood tests.

Exit criteria:

- External tooling can operate the beacon node through complete versioned server APIs.
- `v0.232.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.233.0 - Beacon Block Production Service

Status: planned.

Goal: give the beacon node sole default ownership of unsigned block
construction while keeping the service embeddable behind an explicit trait.

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
- unsigned block and blinded-block production APIs.

Verification:

- Official block-production and operation-ordering vectors;
- local execution-client and PeerDAS integration;
- reorg, timeout, invalid-payload, pool-conflict, and deadline tests;
- checks proving no validator secret or signature enters this service.

Exit criteria:

- The beacon node can produce a complete unsigned local or blinded block for
  every claimed fork, while signing authorization remains outside the service.
- `v0.233.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.234.0 - Validator API And Production Boundary

Status: planned.

Goal: expose complete safety-aware validator APIs while preserving beacon-node
ownership of block construction and validator-client ownership of independent
checks, slashing authorization, signing, and publication.

Deliverables:

- Duties, attestation data, aggregates, unsigned block and blinded-block
  requests, signed publication, sync contributions, proposer preparation, fee
  recipient, liveness, subscriptions, and optimistic/sync safety status;
- API evidence binding responses to head, fork, genesis, slot, and execution
  status;
- explicit separation from the `v0.233.0` production service.

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
- `v0.234.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 24: Slashing Protection And Validator Key Foundation

### v0.235.0 - Slashing Protection Model And Invariants

Status: planned.

Goal: deliver the Slashing Protection Model And Invariants release with this required outcome: Slashability decisions are a small first-party security kernel with explicit invariants.

Deliverables:

- Double-proposal, double-vote, surround-vote, repeat-signing, low-watermark, chain/genesis isolation, and fail-closed decision types.

Verification:

- Exhaustive bounded properties, official slashing cases, Kani candidate proofs.

Exit criteria:

- Slashability decisions are a small first-party security kernel with explicit invariants.
- `v0.235.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.236.0 - Transactional Slashing Database

Status: planned.

Goal: deliver the Transactional Slashing Database release with this required outcome: A signature cannot escape before its slashing record is durably committed.

Deliverables:

- Record-before-signature-release transactions, durable writes, multiprocess locking, concurrency serialization, database-error refusal, backups, and recovery.

Verification:

- Process-kill, concurrent signer, lock loss, torn-write, and restore tests.

Exit criteria:

- A signature cannot escape before its slashing record is durably committed.
- `v0.236.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.237.0 - EIP-3076 Interchange And Safety Recovery

Status: planned.

Goal: deliver the EIP-3076 Interchange And Safety Recovery release with this required outcome: Validator histories move between clients without permitting previously slashable signatures.

Deliverables:

- Version-5 import/export, conservative missing-root handling, merge/low-watermark rules, stopped-client requirement, chain binding, validation, and migration reports.

Verification:

- EIP-3076 schema/examples, cross-client interchange, gap/rollback attack tests.

Exit criteria:

- Validator histories move between clients without permitting previously slashable signatures.
- `v0.237.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.238.0 - Validator Key Foundation And Deposit Data

Status: planned.

Goal: generate validator identities and deposit artifacts with strict
separation between signing keys and withdrawal authority.

Deliverables:

- EIP-2333 BLS key generation and child derivation;
- EIP-2334 validator derivation paths;
- cryptographically secure entropy requirements and deterministic test seams;
- validator signing-key and withdrawal-key role types;
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
- `v0.238.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.239.0 - Validator Signer And Local Keystores

Status: planned.

Goal: isolate local signing behind final domain and slashing authorization,
using the key roles and derivation rules established at `v0.238.0`.

Deliverables:

- Consensus-domain signing packages;
- EIP-2335 keystore import/export and password policy;
- locked and sanitized key memory;
- final fork, genesis, domain, signing-root, and duty-context validation;
- mandatory transactional slashing check before signature release;
- signing audit log, refusal policy, and local key lifecycle;
- hard rejection of withdrawal keys in validator-signing slots.

Verification:

- Official and independent signing/keystore vectors;
- memory-sanitization review;
- wrong-domain, wrong-genesis, withdrawal-key, slashing-DB failure, and
  audit-redaction tests.

Exit criteria:

- Local validator signing is isolated, domain-safe, coupled to durable
  slashing protection, and incapable of consuming withdrawal authority.
- `v0.239.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 25: Complete Validator Client Duties

### v0.240.0 - Validator Duty Scheduler And Safety State

Status: planned.

Goal: deliver the Validator Duty Scheduler And Safety State release with this required outcome: No duty reaches signing unless timing, chain, quorum, and safety preconditions hold.

Deliverables:

- Drift-aware slot clock, duty lookahead/cache, reorg refresh, multi-beacon-node quorum/failover, doppelganger detection, and optimistic/unsafe refusal.

Verification:

- Clock skew, reorg, conflicting-node, startup, and doppelganger simulations.

Exit criteria:

- No duty reaches signing unless timing, chain, quorum, and safety preconditions hold.
- `v0.240.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.241.0 - Proposer Duties Signing And Publication

Status: planned.

Goal: let the validator client request, independently validate, authorize,
sign, and publish proposer duties without owning block construction.

Deliverables:

- RANDAO reveal signing;
- proposer preparation and configuration submission;
- unsigned local/blinded block requests from `v0.233.0`;
- independent slot, parent, fork, fee-recipient, gas-limit, execution-status,
  and data-availability context checks;
- transactional slashing authorization;
- block and sidecar signature production;
- publication deadlines, retries, and duplicate prevention.

Verification:

- Official proposer behavior;
- malicious or stale beacon-node response tests;
- slashing-database failure and duplicate proposal tests;
- local and blinded publication deadline/failure tests.

Exit criteria:

- The validator client can safely sign and publish complete proposer duties
  for claimed forks while parent selection, operation packing, Engine calls,
  and DA construction remain beacon-node responsibilities.
- `v0.241.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.242.0 - Attester And Aggregator Duties

Status: planned.

Goal: deliver the Attester And Aggregator Duties release with this required outcome: Attestation and aggregation duties are complete and slash-safe.

Deliverables:

- Committee assignments, attestation construction, timing, selection proofs,
  aggregation, publication, duplicate prevention, fork-aware signing domains,
  and mandatory transactional slashing authorization before every attestation
  signature.

Verification:

- Official validator vectors, timing/reorg/duplicate simulations.

Exit criteria:

- Attestation and aggregation duties are complete and slash-safe through the
  already admitted `v0.235.0` and `v0.236.0` kernel and database.
- `v0.242.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.243.0 - Sync Committee Duties

Status: planned.

Goal: deliver the Sync Committee Duties release with this required outcome: Sync-committee participation is complete and refuses unsafe chain views.

Deliverables:

- Membership tracking, messages, selection proofs, contributions,
  aggregation/publication, subnet subscriptions, optimistic-node refusal, and
  durable duplicate-signing records before signature release.

Verification:

- Official sync-committee vectors and timing/fork-boundary tests.

Exit criteria:

- Sync-committee participation is complete and refuses unsafe chain views.
- `v0.243.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.244.0 - Validator Lifecycle Requests And Operations

Status: planned.

Goal: deliver the Validator Lifecycle Requests And Operations release with this required outcome: Operators can manage validator lifecycle without bypassing signer or slashing policy.

Deliverables:

- Voluntary exits, BLS-to-execution changes, consolidation/lifecycle requests,
  deposit-data import from `v0.238.0`, fee/graffiti config, key enable/disable,
  authorization checks, and audit records.

Verification:

- Official operation vectors, authorization/policy tests, local testnet workflows.

Exit criteria:

- Operators can manage validator lifecycle without bypassing signer or slashing policy.
- `v0.244.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 26: External And Distributed Validator Key Custody

### v0.245.0 - Keymanager Operator API

Status: planned.

Goal: implement the operator-to-validator-client Keymanager trust direction
without conflating it with outbound remote signing or custody backends.

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
- `v0.245.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.246.0 - Remote Signer Protocol And Slashing Authority

Status: planned.

Goal: define the validator-client-to-signing-service trust direction and make
the authoritative slashing database location explicit for every deployment.

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
- `v0.246.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.247.0 - HSM KMS And Hardware Custody Adapters

Status: planned.

Goal: define the signer-to-custody-backend trust direction for non-exporting
validator keys.

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
- `v0.247.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.248.0 - Threshold DVT And Distributed Slashing Coordination

Status: planned.

Goal: support threshold and distributed validator signing without weakening
single-signature slashing invariants.

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
- `v0.248.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 27: Builder And MEV Integration

### v0.249.0 - Builder API And Blinded Proposals

Status: planned.

Goal: deliver the Builder API And Blinded Proposals release with this required outcome: A validator can use one reviewed relay without trusting its bid or reveal blindly.

Deliverables:

- Validator registration, bid requests, signature/value/header validation, blinded block construction, payload reveal, fork-versioned Builder API, and deadline policy.

Verification:

- Official Builder API conformance, malformed bid/reveal tests, local relay integration.

Exit criteria:

- A validator can use one reviewed relay without trusting its bid or reveal blindly.
- `v0.249.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.250.0 - Relay Multiplexing Local Fallback And PBS Evolution

Status: planned.

Goal: deliver the Relay Multiplexing Local Fallback And PBS Evolution release with this required outcome: External builders cannot prevent a safe local proposal when a viable local payload exists.

Deliverables:

- Multiple relays, bid comparison/minimums, deadline-aware circuit breakers, withholding/invalid-reveal defenses, local-builder fallback, audit evidence, and protocol-native PBS/ePBS adapter boundary.

Verification:

- Relay outage/equivocation/withholding simulations and guaranteed local fallback tests.

Exit criteria:

- External builders cannot prevent a safe local proposal when a viable local payload exists.
- `v0.250.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 28: Consensus Safety Operations And Executables

### v0.251.0 - Optional Slasher Service

Status: planned.

Goal: detect slashable network messages and feed verified evidence into
operation pools without placing detection on the validator signing path.

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
- `v0.251.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.252.0 - Consensus Connectivity And NAT Diagnostics

Status: planned.

Goal: make peer reachability, NAT behavior, subnet participation, and eclipse
risk diagnosable without weakening network policy.

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
- `v0.252.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.253.0 - Consensus Operations Monitoring And Analytics

Status: planned.

Goal: operate and monitor beacon and validator services with stable schemas,
including validator performance and safety analytics.

Deliverables:

- Versioned shared config and CLI schemas;
- structured logs and tracing;
- Prometheus metrics;
- health, readiness, and task diagnostics;
- authentication, TLS, and rate limits;
- safe shutdown and service supervision;
- validator inclusion distance, effectiveness, missed-duty, balance, reward,
  sync participation, proposal, and slashing-risk analytics;
- privacy/redaction policy for validator identifiers and endpoints.

Verification:

- Config compatibility;
- redaction and cardinality tests;
- overload, shutdown, restart, and observability tests;
- analytics differentials against beacon-state outcomes.

Exit criteria:

- Beacon and validator services can be operated, monitored, and performance-
  analyzed without hidden state, secret leakage, or unbounded metric labels.
- `v0.253.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.254.0 - Beacon Node Executable And Packaging

Status: planned.

Goal: ship an explicit production beacon-node executable rather than only
orchestration crates.

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
- `v0.254.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.255.0 - Validator Client Executable And Packaging

Status: planned.

Goal: ship an explicit production validator-client executable with signer and
slashing safety enabled by construction.

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
- `v0.255.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.256.0 - Database Inspection Migration And Recovery Tools

Status: planned.

Goal: deliver the Database Inspection Migration And Recovery Tools release with this required outcome: Operators can diagnose and recover storage without ad hoc database mutation.

Deliverables:

- Read-only inspection, consistency checks, migration commands, checkpoint/state import, backup/restore, pruning controls, repair plans, and dangerous-operation confirmations.

Verification:

- Corrupt/mixed-version database drills and operator workflow tests.

Exit criteria:

- Operators can diagnose and recover storage without ad hoc database mutation.
- `v0.256.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.257.0 - Deterministic Consensus Simulator

Status: planned.

Goal: deliver the Deterministic Consensus Simulator release with this required outcome: Consensus and validator regressions can be reproduced without an external testnet.

Deliverables:

- In-process beacon nodes, validators, execution engines, networks, clocks, partitions, latency, equivocation, invalid payloads, DA loss, builder failures, and reproducible seeds.

Verification:

- Scenario snapshots, determinism checks, mutation testing, regression corpus.

Exit criteria:

- Consensus and validator regressions can be reproduced without an external testnet.
- `v0.257.0 implementation stop reached. Run pentest for this exact
  commit.`

## Phase 29: Full Consensus Assurance And Final 1.0 Admission

### v0.258.0 - Production Acceptance Matrix And Quantitative Budgets

Status: planned.

Goal: replace subjective production gates with a versioned, numeric acceptance
contract before interoperability, longevity, and performance claims run.

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
- required restart, database recovery, execution disagreement, reorg,
  clock-skew, network partition, DA loss, and builder-withholding scenarios;
- numeric mainnet-scale CPU, RAM, stack, disk-growth, disk-I/O, bandwidth,
  API-latency, duty-latency, and startup/recovery budgets on a reproducible
  reference hardware profile;
- an exception process requiring written security review and a replacement
  gate, never silent threshold reduction.

Verification:

- Machine-readable acceptance-policy schema and validator;
- scenario coverage audit;
- hardware-profile reproducibility check;
- dry-run reports that fail on every deliberately violated threshold.

Exit criteria:

- Every remaining interoperability, longevity, performance, and release gate
  has a numeric pass/fail condition and an identified evidence artifact.
- `v0.258.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.259.0 - Hive And Multi-Consensus-Client Interoperability

Status: planned.

Goal: deliver the Hive And Multi-Consensus-Client Interoperability release with this required outcome: The beacon node interoperates with the broader consensus-client ecosystem.

Deliverables:

- Run every required Ethereum Hive consensus suite named by `v0.258.0`;
- consensus P2P, API, state-transition, sync, builder, and validator scenarios;
- compatibility with the full named independent consensus-client matrix;
- explicit issue ownership and waiver prohibition for unexplained failures.

Verification:

- Published Hive and interop reports;
- zero unexplained failures in claimed scope;
- no substitution of private or self-authored tests for a required Hive suite.

Exit criteria:

- The beacon node interoperates with the broader consensus-client ecosystem.
- `v0.259.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.260.0 - Multi-Execution-Client Interoperability

Status: planned.

Goal: deliver the Multi-Execution-Client Interoperability release with this required outcome: Beacon correctness is not coupled to one execution-client implementation.

Deliverables:

- Full Engine workflows against the complete execution-client matrix fixed at
  `v0.258.0`, including payload invalidation, failover, disagreement, reorg,
  blobs/data columns, and restart recovery.

Verification:

- Long-running mixed-client scenarios;
- required restart, partition, reorg, disagreement, and latency cases;
- published failure and recovery reports.

Exit criteria:

- Beacon correctness is not coupled to one execution-client implementation.
- `v0.260.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.261.0 - Long-Running Validator Testnet

Status: planned.

Goal: deliver the Long-Running Validator Testnet release with this required outcome: The complete beacon-node and validator stack demonstrates stable operation under realistic faults.

Deliverables:

- At least the `v0.258.0` minimum 30-day, 4-node, 4,096-validator sustained
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
- `v0.261.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.262.0 - Consensus Client Performance Gate

Status: planned.

Goal: deliver the Consensus Client Performance Gate release with this required outcome: Mainnet-scale consensus workloads meet documented CPU, memory, disk, network, and timing budgets.

Deliverables:

- Enforce the numeric `v0.258.0` budgets for SSZ roots, BLS batches,
  transition/epoch processing, fork choice, pools, storage, networking, sync,
  DA, validator duties, slashing DB, APIs, startup, and recovery.

Verification:

- Reproducible reference hardware profile;
- mainnet-scale load tests;
- threshold validator;
- regression alarms that fail the release.

Exit criteria:

- Mainnet-scale consensus workloads meet documented CPU, memory, disk, network, and timing budgets.
- `v0.262.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.263.0 - Kani State Transition And Fork-Choice Proofs

Status: planned.

Goal: deliver the Kani State Transition And Fork-Choice Proofs release with this required outcome: Selected consensus-state and fork-choice safety invariants have machine-checked evidence.

Deliverables:

- Bounded proofs for transition rollback, balance/churn arithmetic, checkpoint monotonicity, latest-message handling, invalidation, and transactional fork-choice handlers.

Verification:

- Pinned Kani runs with documented assumptions and bounds.

Exit criteria:

- Selected consensus-state and fork-choice safety invariants have machine-checked evidence.
- `v0.263.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.264.0 - Kani Slashing And Duty-Safety Proofs

Status: planned.

Goal: deliver the Kani Slashing And Duty-Safety Proofs release with this required outcome: Selected validator and slashing invariants have machine-checked evidence.

Deliverables:

- Bounded proofs for double proposal/vote, surround vote, record-before-release state machines, optimistic-node refusal, duty uniqueness, and signer-domain binding.

Verification:

- Pinned Kani runs and cross-checks against property suites.

Exit criteria:

- Selected validator and slashing invariants have machine-checked evidence.
- `v0.264.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.265.0 - SSZ BLS PeerDAS And Acceleration Audit

Status: planned.

Goal: independently audit the cryptographic and authenticated-data
implementations introduced after the earlier core audit.

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
- `v0.265.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.266.0 - State Transition And Fork-Choice Audit

Status: planned.

Goal: deliver the State Transition And Fork-Choice Audit release with this required outcome: No unresolved critical/high transition or fork-choice finding remains.

Deliverables:

- Independent audit of fork upgrades, per-slot/epoch transition, deposit and
  genesis services, fork choice, optimistic execution, operation pools,
  Engine evidence consumption, and persistence, using the implementation audit
  from `v0.265.0` as a prerequisite.

Verification:

- Published report, remediation register, clean retest.

Exit criteria:

- No unresolved critical/high transition or fork-choice finding remains.
- `v0.266.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.267.0 - Consensus Network Sync And DA Audit

Status: planned.

Goal: deliver the Consensus Network Sync And DA Audit release with this required outcome: No unresolved critical/high network, sync, or data-availability finding remains.

Deliverables:

- Independent audit of discv5/libp2p/GossipSub, NAT/connectivity adapters,
  validation, scoring, ReqResp, sync, weak subjectivity, PeerDAS consumers,
  custody, availability, slasher ingestion, and DoS controls;
- integration review proving every PeerDAS consumer validates through the
  `v0.193.0` core audited at `v0.265.0`.

Verification:

- Published report, adversarial retest, clean pentest.

Exit criteria:

- No unresolved critical/high network, sync, or data-availability finding remains.
- `v0.267.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.268.0 - Validator Slashing Keymanager And Builder Audit

Status: planned.

Goal: deliver the Validator Slashing Keymanager And Builder Audit release with this required outcome: No unresolved critical/high signing, slashing, key-custody, or builder finding remains.

Deliverables:

- Independent audit of duties, timing, doppelganger detection, key generation,
  withdrawal-key separation, deposit data, slashing DB, EIP-3076, keystores,
  Keymanager API, remote signer slashing authority, HSM/KMS/hardware adapters,
  threshold/DVT coordination, relays, and local fallback.

Verification:

- Published report, slashing-safety retest, clean pentest.

Exit criteria:

- No unresolved critical/high signing, slashing, key-custody, or builder finding remains.
- `v0.268.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.269.0 - Beacon Storage API And Operations Audit

Status: planned.

Goal: deliver the Beacon Storage API And Operations Audit release with this required outcome: No unresolved critical/high storage, API, or operational finding remains.

Deliverables:

- Independent audit of hot/cold storage, snapshots, migrations, repair,
  Beacon/Validator APIs, configuration, authentication, metrics, validator
  analytics, supervision, executable startup/shutdown, data directories,
  containers/system services, and binary upgrade/rollback behavior.

Verification:

- Published report, recovery/authorization retest, clean pentest.

Exit criteria:

- No unresolved critical/high storage, API, or operational finding remains.
- `v0.269.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.270.0 - Complete Consensus Remediation

Status: planned.

Goal: deliver the Complete Consensus Remediation release with this required outcome: The entire beacon-node and validator finding register is closed or explicitly accepted.

Deliverables:

- Resolve all consensus-client findings, rerun official vectors, Hive, long-running testnet, performance, formal, and platform gates, and document accepted low risks.

Verification:

- All audit retests clean, zero unexplained conformance failures, updated SBOM/provenance.

Exit criteria:

- The entire beacon-node and validator finding register is closed or explicitly accepted.
- `v0.270.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.271.0 - Complete Public API Freeze

Status: planned.

Goal: deliver the Complete Public API Freeze release with this required outcome: No foundational API invention remains before 1.0.

Deliverables:

- Freeze all core, SDK, execution, provider, wallet, contract, storage, beacon, validator, network, slashing, builder, and operational APIs/features with migration/deprecation policy.

Verification:

- Whole-workspace semver and feature review, generated docs/README compatibility checks.

Exit criteria:

- No foundational API invention remains before 1.0.
- `v0.271.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.272.0 - Complete Release Evidence Dry Run

Status: planned.

Goal: deliver the Complete Release Evidence Dry Run release with this required outcome: The exact 1.0 release mechanics and operator upgrade path have been exercised.

Deliverables:

- Rehearse ordered crate publication, signed manifests/checksums,
  SBOM/provenance, reproducible packages, platform images, database migrations,
  config migration, binary rollback, and the exact `v1.0.0-rc.1` same-commit
  promotion procedure.

Verification:

- Full dry run from a clean environment;
- deliberately failed version-promotion and changed-candidate tests;
- proof that final tagging can occur without rebuilding or changing the
  candidate commit.

Exit criteria:

- The exact 1.0 release mechanics and operator upgrade path have been exercised.
- `v0.272.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.273.0 - Version-Only 1.0 Promotion Rehearsal

Status: planned.

Goal: prove that changing workspace package versions to `1.0.0` is an
isolated, reviewable promotion step whose outputs are fully regenerated and
revalidated.

Deliverables:

- Rehearse the exact manifest, workspace dependency, lockfile, crate-version
  matrix, release-plan metadata, release notes, SBOM, checksum, and provenance
  changes required for `1.0.0`;
- require a version-only promotion commit with no implementation changes;
- regenerate every package and compare source contents apart from expected
  version metadata;
- document rollback and repeated-RC handling.

Verification:

- Clean-tree promotion rehearsal;
- package-content semantic diff;
- full release-integrity gate on the promoted rehearsal;
- proof that any non-version change rejects the promotion.

Exit criteria:

- The project can produce a `1.0.0`-versioned candidate commit through a
  constrained, audited process rather than pretending a `0.x` artifact is
  byte-identical after version changes.
- `v0.273.0 implementation stop reached. Run pentest for this exact
  commit.`

### v0.274.0 - Production Candidate Admission Gate

Status: planned.

Goal: freeze the final `0.x` implementation and authorize one version-only
promotion commit to the exact `v1.0.0-rc.1` candidate.

Deliverables:

- Freeze all implementation;
- run every official conformance, required Hive, live-node, quantitative
  long-testnet, audit, formal, compatibility, platform, packaging, and release
  gate;
- publish final migration and support matrices;
- authorize only the constrained `v0.273.0` version-promotion operation;
- invalidate admission if any implementation or non-version metadata changes.

Verification:

- Exact implementation-candidate pentest and retest;
- green CI and CodeQL;
- reproducible packages;
- quantitative acceptance-policy pass;
- no post-review implementation changes.

Exit criteria:

- The only permitted next change is the audited version-only promotion commit
  that will be tagged `v1.0.0-rc.1`.
- `v0.274.0 implementation stop reached. Run pentest for this exact
  commit.`

## v1.0.0-rc.1 - Exact Production Candidate

Status: planned release candidate.

Goal: create the actual `1.0.0`-versioned artifact once, test that exact commit,
and use it unchanged for the final stable tag.

Deliverables:

- Apply only the version-promotion changes rehearsed at `v0.273.0`;
- set publishable workspace manifests to their approved `1.0.0` versions;
- regenerate lockfiles, crate-version matrices, SBOM, checksums, provenance,
  package archives, and release metadata;
- tag the exact promoted commit as `v1.0.0-rc.1`;
- publish no stable crate until final admission.

Verification:

- Full release gate on the promoted commit;
- exact-candidate pentest and clean retest;
- green CI and CodeQL on the promoted commit;
- reproducible package and checksum verification;
- semantic package diff proving only approved version metadata differs from
  `v0.274.0`;
- repeat as `v1.0.0-rc.N` from a newly reviewed commit if any change is needed.

Exit criteria:

- The exact commit is approved for the final `v1.0.0` tag with no further
  source, manifest, lockfile, documentation, SBOM, or packaging changes.
- `v1.0.0-rc.1 implementation stop reached. Run final candidate pentest for
  this exact commit.`

## v1.0.0 - Complete Production Ethereum Toolkit

Goal: publish the first serious production-ready release only after the complete
roadmap above has reached its exit criteria for every capability claimed by
the support matrix.

Deliverables:

- complete owned SDK, first-party execution, provider, wallet, contract,
  storage, canonical-chain, full beacon-node, validator-client,
  consensus-light-client, execution and consensus networking, sync,
  slashing-protection, builder, and stateless-support surfaces described above;
- historical and current fork support backed by pinned official conformance
  evidence;
- a production beacon node that coordinates with independent execution clients
  through the Engine API, plus a validator client that refuses unsafe duties;
- transactional slashing protection, EIP-3076 interchange, Keymanager API,
  local/remote/HSM signer boundaries, and safe builder fallback;
- explicit unsupported/future-fork behavior with no silent fallback;
- stable API, feature, MSRV, platform, and migration policy;
- signed release manifest, checksums, SBOM, provenance, audit references, and
  dependency/feature compatibility matrix.

Verification:

- `scripts/checks.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/generate-sbom.sh --check`
- all official conformance, interoperability, fuzz, Kani, Miri, sanitizer,
  platform, performance, Hive, long-running validator, and live-node gates
  assigned above;
- `scripts/validate-release-readiness.sh v1.0.0`
- verify that `v1.0.0` points to the exact already approved
  `v1.0.0-rc.N` commit;
- do not rebuild, regenerate, or modify candidate artifacts;
- verify published crate checksums against the approved candidate;
- pentest and clean retest evidence for the exact candidate commit.

Exit criteria:

- no unresolved critical or high finding;
- no unexplained conformance skip for any claimed feature or fork;
- every public capability is implemented, tested, documented, and represented
  accurately in the support matrix;
- the final `v1.0.0` tag points to the unchanged approved
  `v1.0.0-rc.N` commit.
