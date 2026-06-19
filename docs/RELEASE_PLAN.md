# eth Release Plan To 1.0

Status: planning document

Tags use:

```text
v0.N.0      milestone release
v0.N.P      patch/fix release
v1.0.0      first serious production-ready Ethereum crate
```

## Release Principles

Every release must have:

- definition of done;
- local verification command;
- security review notes;
- known limitations;
- release notes;
- dependency-policy evidence;
- no hidden dependency on one developer machine.

## Clean Stop And Pentest Rule

Each version has a deliberate clean stop. When implementation criteria are done,
the work stops before tagging and the maintainer is told:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

No tag is created at that point.

Pentest flow:

1. Implementation reaches the version stop point.
2. Local gates pass: `scripts/checks.sh`, `cargo deny check`, and `cargo audit`.
3. The maintainer runs pentest and writes findings to root `PENTEST.md`.
4. Findings are reviewed and fixed.
5. `PENTEST.md` is removed after findings are handled.
6. Local gates are run again.
7. A permanent report is written at `security/pentest/<tag>.md` only when the
   exact commit is ready to tag and the result is `Status: PASS`.
8. Tagging and pushing tags happen only when explicitly requested.

## v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.96.0` pinned.
- Rust `1.90.0` through `1.96.0` compatibility policy.
- Focused no_std workspace crates.
- `scripts/checks.sh`.
- CI, dependency policy, security policy, release notes.
- Implementation, release, scope, threat-model, modularity, toolchain,
  unsafe, spec, and supply-chain docs.

Verification:

- `scripts/checks.sh`
- `scripts/release_0_1_gate.sh`
- `cargo test --workspace`

## v0.2.0 - Primitives And Decode Budgets

Goal: make Ethereum domain types and decoder budgets explicit.

Deliverables:

- chain, block, gas, nonce, timestamp, address, hash, and fork primitives;
- decode-budget types;
- checked arithmetic policy for lengths and allocations;
- exact-consumption helpers;
- negative tests for budget violations.

## v0.3.0 - Canonical RLP Foundation

Goal: implement bounded canonical RLP decoding.

Deliverables:

- admitted codec dependency or reviewed local implementation;
- canonical integer rejection;
- list and nesting limits;
- exact-consumption decoding;
- fuzz target for RLP.

## v0.4.0 - Typed Transaction Envelopes

Goal: parse and classify typed Ethereum transactions safely.

Deliverables:

- EIP-2718 envelope model;
- legacy and admitted typed transaction shells;
- unsupported type errors;
- round-trip and malformed-input tests.

## v0.5.0 - Fork Model And Chain Specs

Goal: make validation fork context explicit.

Deliverables:

- `ChainSpec` and `ForkSpec`;
- block-number and timestamp activation;
- unsupported fork handling;
- tests for fork boundary transitions.

## v0.6.0 - Transaction Validation Typestates

Goal: separate decode, canonicality, fork validation, and sender recovery.

Deliverables:

- transaction validation states;
- no partial mutation on failure;
- deterministic error categories;
- tests for every transition and invalid transition.

## v0.7.0 - Signature And Replay Verification

Goal: verify transaction replay domains and sender recovery.

Deliverables:

- secp256k1 dependency admission;
- low-s and recovery-id policy;
- wrong-chain rejection;
- EIP-155 and typed transaction replay tests.

## v0.8.0 - EIP-712 Safety Rules

Goal: validate structured-data signing domains safely.

Deliverables:

- complete domain requirement helpers;
- expected chain and verifying-contract checks;
- no raw-digest signing as the primary public API;
- tests for domain confusion.

## v0.9.0 - MPT Proof Verification

Goal: verify transaction, receipt, and account/storage proofs.

Deliverables:

- trie dependency admission;
- proof size and node-count limits;
- invalid proof rejection;
- fixture-backed tests.

## v0.10.0 - Header And Receipt Validation

Goal: validate block header and receipt relationships.

Deliverables:

- header hash consistency;
- receipts root checks;
- logs bloom policy;
- fork-specific header fields.

## v0.11.0 - REVM Adapter Preview

Goal: add optional execution with explicit environment and snapshot inputs.

Deliverables:

- REVM dependency admission;
- execution environment conversion;
- state snapshot trait;
- bounded execution result model.

## v0.12.0 - Gas Estimation And Trace Bounds

Goal: make simulation bounded and auditable.

Deliverables:

- maximum executions;
- gas cap;
- timeout or worker policy;
- trace-step limits;
- adversarial performance tests.

## v0.13.0 - RPC Trust Policy

Goal: add optional RPC with explicit endpoint and response-trust policy.

Deliverables:

- no default public endpoints;
- trusted, quorum, and verified modes;
- request/response size limits;
- retry policy by method class;
- redaction tests.

## v0.14.0 - Signer Boundary

Goal: add optional signer isolation without making local signing a default.

Deliverables:

- domain-specific transaction signing trait;
- external signer process model;
- secret redaction tests;
- local keystore only as a separate fallback feature.

## v0.15.0 - Conformance Harness

Goal: pin official Ethereum test-suite revisions and report status.

Deliverables:

- `spec-lock.toml`;
- fixture download or import process;
- pass/fail matrix;
- regression handling docs.

## v0.16.0 - Reth Adapter Preview

Goal: integrate selected Reth concepts without leaking node internals.

Deliverables:

- exact dependency review;
- type conversion boundary;
- direct database or transaction-pool adapter skeleton;
- no default graph expansion.

## v0.17.0 - P2P Threat Model Expansion

Goal: decide whether P2P belongs in this crate family.

Deliverables:

- expanded threat model;
- message-size and RLPx policy;
- process-isolation plan;
- defer or implement decision.

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
