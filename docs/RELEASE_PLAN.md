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
- negative and adversarial tests with each parser;
- explicit fork and chain context over global "latest" behavior;
- optional local Ethereum node smoke fixtures before RPC functionality is
  treated as complete;
- no default networking, signing, Reth, P2P, or local key storage.

## Pentest Before Tags

Every version must pass a security review and pentest before it is tagged. This
applies to `v0.N.P` patch tags as well as milestone tags.

A version is not tag-ready until:

- `scripts/checks.sh` passes;
- `cargo deny check` passes;
- `cargo audit` passes;
- `scripts/generate-sbom.sh` succeeds;
- release notes exist at `release-notes/RELEASE_NOTES_X.Y.Z.md`;
- a pentest report exists at `security/pentest/vX.Y.Z.md`;
- the pentest report names the exact full 40-character `Reviewed-Commit:`;
- the pentest report has `Status: PASS`;
- the pentest report has non-blank `Tester:` and `Scope:` fields;
- the pentest report has a `Date: YYYY-MM-DD` field;
- `sbom/eth.spdx.json` exists and is non-empty;
- the tag does not already exist locally;
- `scripts/validate-release-readiness.sh vX.Y.Z` passes.

`scripts/check_latest_tools.sh` is an advisory networked current-version check.
Run it before updating pinned tools and before release when network access is
available, but do not make tag readiness depend on live upstream state.

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
6. `PENTEST.md` is removed after findings are handled.
7. Local gates are run again.
8. GitHub CI and CodeQL default setup are checked after the fix commit.
9. A permanent report is written at `security/pentest/vX.Y.Z.md` only when the
   exact implementation commit has passed with `Status: PASS`.
10. Commit only the permanent report as the release report commit.
11. GitHub CI and CodeQL default setup are checked on the release report commit.
12. `scripts/validate-release-readiness.sh vX.Y.Z` passes.
13. Tagging and pushing tags happen only when explicitly requested.

Root `PENTEST.md` is temporary scratch input. It must not be committed.
The permanent report is part of the release tag. Because committing the report
changes `HEAD`, the report records `Reviewed-Commit:` rather than claiming to
hash itself. The release-readiness gate requires the tag candidate commit to
have the reviewed commit as its first parent and to change only the permanent
report file.

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

## Phase 0: Repository And Release Discipline

### v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.96.0` pinned.
- Rust `1.90.0` through `1.96.0` compatibility policy.
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
- if using a trait boundary, define the minimal hasher trait and test doubles;
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
- seed corpus from unit fixtures;
- crash reproduction docs.

Verification:

- fuzz target builds;
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

Goal: decide and prototype derive support for RLP encoding and decoding only
after hand-written primitive and transaction APIs have stabilized.

Deliverables:

- derive macro API design for `RlpEncode` and `RlpDecode`;
- explicit field-order policy and skip/default-field rules;
- generated code uses the same bounded codec helpers as hand-written paths;
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

Goal: recover senders through an admitted secp256k1 dependency.

Deliverables:

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

Goal: prevent structured-data domain confusion.

Deliverables:

- complete-domain helpers;
- expected chain and verifying-contract checks;
- tests for missing or wrong domain fields.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Raw digest signing is not the primary safe signing interface.

## Phase 5: Blocks, Receipts, And Proofs

### v0.22.0 - Header Decode And Hashing

Goal: parse and hash execution-layer block headers.

Deliverables:

- header field model;
- fork-specific optional field handling;
- hash consistency tests.

Verification:

- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify`

Exit criteria:

- Headers can be decoded without implying full block validity.

### v0.23.0 - Receipt Decode

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

### v0.24.0 - Withdrawal And Post-Merge Fields

Goal: model post-merge execution fields explicitly.

Deliverables:

- withdrawal structures;
- withdrawals-root input model;
- timestamp/fork interaction tests.

Verification:

- `cargo test -p eth-valkyoth-protocol`

Exit criteria:

- Post-merge fields are not bolted onto pre-merge validation paths.

### v0.25.0 - MPT Node Decoder

Goal: decode trie nodes with strict limits.

Deliverables:

- trie node representation;
- proof-node count and byte limits;
- malformed node tests.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Trie proof input cannot allocate or recurse without limits.

### v0.26.0 - Inclusion Proof Verification

Goal: verify transaction and receipt inclusion proofs.

Deliverables:

- transaction proof verification;
- receipt proof verification;
- invalid proof fixtures.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Inclusion proof APIs distinguish malformed, absent, and wrong-root proofs.

### v0.27.0 - Account And Storage Proofs

Goal: verify account and storage proofs against trusted roots.

Deliverables:

- account proof verification;
- storage proof verification;
- missing-node and wrong-value tests.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Verified RPC state has a cryptographic proof path separate from trusted RPC.

## Phase 6: Conformance And Test Infrastructure

### v0.28.0 - Spec Lock And Fixture Import

Goal: pin official Ethereum specification and fixture revisions.

Deliverables:

- populated `spec-lock.toml`;
- fixture import or download process;
- fixture license notes;
- reproducible fixture path.
- `/home/eldryoth/Work/test/eth` documented as the local reference store.

Verification:

- `scripts/checks.sh`

Exit criteria:

- Every conformance claim names exact upstream revisions.

### v0.29.0 - Execution Test Harness

Goal: run applicable Ethereum execution tests through protocol validation.

Deliverables:

- fixture runner;
- pass/fail report;
- known unsupported fixture list.

Verification:

- conformance runner command documented and passing for claimed fixtures.

Exit criteria:

- Validation behavior is tested against external Ethereum material.

### v0.30.0 - Differential Test Harness

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

### v0.31.0 - REVM Dependency Admission

Goal: admit REVM behind `eth-valkyoth-evm` with reviewed features.

Deliverables:

- dependency review;
- no default feature expansion;
- minimal compile-only adapter.

Verification:

- `cargo check --workspace --all-features`
- `cargo deny check`

Exit criteria:

- REVM is optional and cannot enter the default core graph.

### v0.32.0 - Explicit Execution Environment

Goal: execute with explicit fork, block, transaction, and snapshot inputs.

Deliverables:

- environment conversion;
- state snapshot trait;
- execution result model.

Verification:

- `cargo test -p eth-valkyoth-evm`

Exit criteria:

- Simulation reports the exact state and fork configuration used.

### v0.33.0 - Bounded Gas Estimation

Goal: make gas estimation bounded and auditable.

Deliverables:

- maximum execution count;
- gas cap;
- timeout or worker isolation policy;
- deterministic error classification.

Verification:

- adversarial gas-estimation tests.

Exit criteria:

- Gas estimation cannot become an unbounded execution loop.

## Phase 8: Optional RPC And Signer Boundaries

### v0.34.0 - RPC Dependency Admission

Goal: admit provider/transport crates behind `eth-valkyoth-rpc` and add the
local node harness used by later live integration tests.

Deliverables:

- dependency review;
- no hardcoded public endpoints;
- endpoint policy types;
- Podman-managed local Ethereum dev node fixture;
- start, health-check, and teardown script for the fixture;
- localhost-only JSON-RPC binding;
- pinned container image name/version or digest with an update note;
- no persisted wallet/key material and no default mainnet connection.

Verification:

- `cargo check --workspace --all-features`
- `cargo deny check`
- local node smoke script starts the fixture, verifies JSON-RPC health, and
  tears it down

Exit criteria:

- RPC support is optional and policy-first.
- The project can spin up and destroy a local Ethereum node for integration
  tests without depending on a developer's existing node.

### v0.35.0 - RPC Trust Models

Goal: implement trusted, quorum, and verified response models.

Deliverables:

- trust model APIs;
- chain/genesis verification at connection setup;
- response size and batch limits.

Verification:

- malicious RPC fixture tests.

Exit criteria:

- TLS endpoint trust is documented as separate from Ethereum state trust.

### v0.36.0 - RPC Retry And Redaction

Goal: make network behavior and logs safe by default.

Deliverables:

- method-class retry policy;
- no automatic transaction rebroadcast;
- manual redacted `Debug` for any RPC error type carrying request context;
- URL credential, calldata, and transaction redaction tests.

Verification:

- `cargo test -p eth-valkyoth-rpc`

Exit criteria:

- RPC errors and logs do not leak sensitive payloads by default.

### v0.37.0 - Signer Interface

Goal: add domain-specific signer APIs without local keys by default.

Deliverables:

- transaction signing trait;
- EIP-712 signing trait;
- no raw digest primary API;
- manual redacted `Debug` for signer error types;
- external signer model.

Verification:

- `cargo test -p eth-valkyoth-signer`

Exit criteria:

- Signing APIs make domain separation explicit.

### v0.38.0 - Local Signer Fallback

Goal: add optional local signing only after secret-handling review.

Deliverables:

- dependency review for secret handling;
- no-debug secret wrappers;
- manual redacted `Debug` for all local-signer errors;
- redaction tests;
- local signer feature remains non-default.

Verification:

- `cargo test -p eth-valkyoth-signer --all-features`
- `cargo deny check`

Exit criteria:

- Local key material cannot enter default builds.

## Phase 9: Optional Reth And P2P Decisions

### v0.39.0 - Reth Dependency Admission

Goal: integrate selected Reth concepts without leaking node internals.

Deliverables:

- exact dependency review;
- type conversion boundary;
- no default graph expansion;
- adapter threat-model update.

Verification:

- `cargo check --workspace --all-features`
- `cargo deny check`

Exit criteria:

- Reth remains an adapter, not a protocol-core foundation.

### v0.40.0 - P2P Threat Model Decision

Goal: decide whether P2P belongs in this crate family.

Deliverables:

- expanded threat model;
- message-size and RLPx policy;
- process-isolation plan;
- defer or implement decision.

Verification:

- security review of the decision document.

Exit criteria:

- P2P is either explicitly deferred or split into its own future release plan.

## Phase 10: Production Hardening

### v0.41.0 - Platform Matrix

Goal: verify supported operating systems and targets.

Deliverables:

- Linux, Windows, BSD, macOS, Android, and iOS build notes;
- no_std target checks;
- CI matrix expansion where practical.

Verification:

- documented platform check commands.

Exit criteria:

- Platform support claims match tested evidence.

### v0.42.0 - Public API Stability Pass

Goal: stabilize the public API shape before 1.0.

Deliverables:

- API stability policy update;
- deprecation policy;
- feature compatibility matrix;
- migration notes for all breaking changes.

Verification:

- docs and examples compile.

Exit criteria:

- The remaining 1.0 work is hardening, not API invention.

### v0.43.0 - Independent Audit Remediation

Goal: fix findings from external review.

Deliverables:

- audit report reference;
- remediation register;
- tests for fixed findings;
- risk acceptance for any residual issue.

Verification:

- `scripts/checks.sh`
- audit remediation review.

Exit criteria:

- No unresolved critical or high findings remain.

### v0.44.0 - Release Evidence Dry Run

Goal: prove the release-evidence process before 1.0.

Deliverables:

- signed release manifest draft;
- SBOM;
- provenance notes;
- conformance report;
- dependency compatibility matrix.

Verification:

- release-readiness script for `v0.44.0`.

Exit criteria:

- 1.0 release mechanics have already been exercised.

## v1.0.0 - Production Ethereum Toolkit

Goal: publish the first serious production-ready `eth` release.

Deliverables:

- all claimed conformance suites pass;
- supported-fork matrix;
- dependency compatibility matrix;
- signed release manifest and checksums;
- SBOM and provenance;
- independent security review report;
- migration guide;
- no unresolved critical or high findings.

Verification:

- `scripts/checks.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/generate-sbom.sh`
- `scripts/validate-release-readiness.sh v1.0.0`

Exit criteria:

- `v1.0.0 implementation stop reached. Run pentest for this exact commit.`
