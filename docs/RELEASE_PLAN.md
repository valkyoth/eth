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
| Formal verification evidence was not scheduled. | Added `v0.94.0 - Kani Formal Verification Harness` as extra assurance, not a replacement for fuzzing, conformance tests, pentest, or audit. |
| ABI encoding, Engine API, SSZ, and DevP2P/RLPx were marked deferred. | Added `v0.70.0` through `v0.92.0` feature tracks so they are versioned before 1.0. |
| ENS and common ERC/application standards were not scheduled. | Added `v0.74.0` through `v0.77.0` for common token, ENS, permit, and interface helpers. |
| Node-level sync, txpool, mining/validator boundaries, and observability were not scheduled. | Added `v0.90.0` through `v0.92.0` with explicit library-boundary scope and validation gates. |
| REVM dependency admission failed the existing dependency policy. | Added `v0.37.1 - REVM Dependency Recheck` before execution work may continue. |
| Native audited EVM execution was not explicitly versioned; REVM could look like the long-term core. | Added `v0.40.0` through `v0.54.0` as the first-party EVM engine phase and shifted later versions upward. |
| Default verification previously depended directly on `k256` and used direct `sha3` test wrappers, which conflicted with the long-term first-party-core goal. | Added `v0.37.2` and `v0.37.3` to audit core dependencies, move cryptographic implementation crates behind explicit boundaries/features, and document any accepted cryptographic backend plan. |
| `subtle`, `alloy-rlp`, dev `serde_json`, optional `serde`/`serde_json`, and optional `sanitization` need explicit long-term dependency classifications before execution grows. | Added `v0.37.4` and `v0.37.5` so constant-time helpers, reference oracles, JSON parser support, and sanitization bridges remain deliberate dependency choices. |
| `v0.45.0` deliberately admits cryptographic precompiles as fail-closed descriptors without concrete execution backends. | Added `v0.46.0` through `v0.52.0` for SHA-256, RIPEMD-160, ECRECOVER, ModExp, BN254, BLAKE2F, KZG/BLS backend planning, conformance vectors, fuzzing, dependency review, and pentest gates before state-test claims depend on them. |
| Native opcodes alone do not make full Ethereum execution support; genesis, full block validity, trie-root construction, state transition integration, blob/KZG validation, and full execution fixtures were not versioned before RPC/Reth work. | Added `v0.55.0` through `v0.62.0` for full execution state and block-validity work, then shifted later integration tracks upward. |
| The native EVM state-access pass intentionally fails closed for pre-London forks until historical gas/opcode rules are implemented. | Added `v0.43.1 - Native EVM Historical Fork Matrix` and `v0.43.2 - Native EVM Pre-Berlin State Gas Schedules` before calls/create build on state access. |

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

Verification:

- official EIP-2537 field, point, infinity, and scalar encoding rules;
- boundary tests for `0`, `p - 1`, `p`, non-zero top padding, `q`, and scalars
  greater than `q`;
- round-trip tests for every admitted wire domain;
- `cargo test -p eth-valkyoth-evm-core`;
- strict core and fuzz clippy gates;
- pentest and retest before tagging.

Exit criteria:

- Every BLS input byte is either decoded into one canonical bounded domain or
  rejected before arithmetic, without allocation or alternate encodings.
- Arithmetic precompiles remain fail closed until their assigned releases.

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

### v0.53.0 - Native EVM Ethereum State Tests

Goal: claim execution behavior only where official Ethereum state tests pass.

Deliverables:

- pinned official state-test revision;
- harness mapping tests to supported forks and unsupported-skip reasons;
- differential comparison against REVM or another independent engine if
  admitted as a reference path;
- precompile-dispatch integration tests proving unavailable native backends
  revert rather than succeed or no-op;
- precompile gas-order integration tests proving gas is deducted before
  invoking expensive validation or execution paths such as BN254 pairing
  subgroup checks;
- report of claimed and unclaimed forks/opcodes.

Verification:

- state-test harness command documented and passing for claimed support.

Exit criteria:

- Native execution claims are backed by pinned official test evidence.

### v0.54.0 - Native EVM Audit Hardening

Goal: harden the native engine before broader integration depends on it.

Deliverables:

- fuzz targets for bytecode, stack, memory, gas, state, and call-frame paths;
- Kani candidate list for bounded arithmetic and stack invariants;
- DoS/load tests for configured execution limits;
- unsafe-code and dependency review;
- pentest-focused remediation pass.

Verification:

- `cargo check --manifest-path fuzz/Cargo.toml`
- engine-specific hardening report.

Exit criteria:

- Native execution is the preferred long-term path; any REVM adapter remains a
  reference or compatibility layer.

## Phase 9: Full Execution State And Block Validity

This phase turns native opcode execution into full execution-layer behavior.
It covers the upstream fixture groups currently listed as unsupported:
`TransactionTests`, `BlockchainTests`, `GenesisTests`, `TrieTests`,
`DifficultyTests`, and fork-specific EOF/state-transition tests.

### v0.55.0 - Genesis And Chain Configuration Import

Goal: construct initial state and chain configuration from explicit genesis
inputs without trusting a node client.

Deliverables:

- genesis account, storage, code, balance, nonce, and allocation model;
- chain configuration import with fork activation rules;
- genesis header construction and hash calculation;
- genesis state root construction boundary;
- `GenesisTests` fixture harness.

Verification:

- `GenesisTests` pass for claimed forks/chains;
- `cargo test -p eth-valkyoth-protocol -p eth-valkyoth-verify`.

Exit criteria:

- A claimed chain can start from reproducible first-party genesis data.

### v0.56.0 - Transaction Semantic Validity

Goal: validate transaction semantics before state transition execution.

Deliverables:

- intrinsic gas calculation for every admitted transaction type;
- nonce, balance, gas-limit, fee-cap, priority-fee, blob-fee, and access-list
  semantic checks;
- EIP-4844 blob-hash count/version/fork checks;
- EIP-7702 account/delegation integration with sender-state validation;
- `TransactionTests` harness for claimed transaction families.

Verification:

- official transaction fixtures for claimed forks;
- adversarial fee, nonce, access-list, blob, and authorization tests.

Exit criteria:

- Decoded and signature-valid transactions are not treated as executable until
  semantic validity passes against explicit state and fork context.

### v0.57.0 - Header And Block Validity

Goal: validate execution block headers and block-level constraints.

Deliverables:

- parent/number/timestamp/gas/base-fee/blob-gas/excess-blob-gas validation;
- difficulty and pre/post-Merge terminal-total-difficulty boundary rules;
- withdrawals-root, transactions-root, receipts-root, logs-bloom, state-root,
  and requests-hash validation hooks where fork-applicable;
- ommers/uncles handling for pre-Merge forks and explicit post-Merge rejection;
- fork activation and optional field consistency checks.

Verification:

- `BlockchainTests` header/block-validity fixture subset for claimed forks;
- `DifficultyTests` for claimed historical forks.

Exit criteria:

- Block headers are no longer only syntactically decoded and hashed; they can be
  validated against parent/fork context for claimed forks.

### v0.58.0 - State Transition Integration

Goal: apply valid transactions through the native EVM and update account state.

Deliverables:

- account journal and commit/revert model wired to native EVM execution;
- gas purchase/refund, miner/beneficiary fee accounting, and sender nonce
  updates;
- contract code deployment and account creation/destruction rules;
- access-list warm/cold state integration;
- fork-specific state-transition hooks.

Verification:

- official state tests for claimed forks and opcode families;
- differential comparison against admitted reference engines when available.

Exit criteria:

- The crate can execute claimed transactions against explicit state and produce
  deterministic post-state.

### v0.59.0 - Receipts, Logs, Bloom, And Withdrawals State Application

Goal: produce and validate execution outputs that bind state transition to
block roots.

Deliverables:

- receipt construction from execution results;
- cumulative-gas accounting and receipt type matching;
- log bloom construction and validation;
- withdrawal state application and withdrawals-root validation;
- receipt trie construction boundary.

Verification:

- receipt, withdrawal, and block fixture subsets for claimed forks;
- malformed cumulative-gas and bloom tests.

Exit criteria:

- Receipts and withdrawals are tied to execution results and block roots, not
  only decoded as standalone data.

### v0.60.0 - Trie Construction And Root Computation

Goal: build canonical Merkle Patricia tries, not only verify supplied proofs.

Deliverables:

- first-party trie insertion/update/delete model;
- transaction trie, receipt trie, account trie, and storage trie root builders;
- compact-path and node-encoding reuse from the bounded MPT decoder;
- state-root and storage-root recomputation fixtures;
- memory and node-count limits for hostile trie construction inputs.

Verification:

- `TrieTests` pass for claimed trie behavior;
- fuzz target for trie construction and root computation.

Exit criteria:

- Root values can be computed first-party for claimed block/state data.

### v0.61.0 - Blob, KZG, And Data-Availability Boundaries

Goal: validate blob-transaction execution-layer commitments without hiding
cryptographic backend assumptions.

Deliverables:

- KZG commitment/proof backend boundary with dependency/first-party decision;
- blob versioned-hash validation, count limits, fee accounting, and fork rules;
- point-evaluation precompile integration if fork-applicable;
- trusted setup handling policy;
- blob-related official fixture coverage.

Verification:

- KZG and blob transaction vectors for claimed forks;
- `cargo deny check`;
- documented backend conformance command.

Exit criteria:

- Blob transaction support is no longer only syntactic; every cryptographic
  assumption is explicit.

### v0.61.1 - Trusted Setup Provenance And Loading

Status: planned.

Goal: pin, fingerprint, parse, and fail closed around the canonical KZG trusted
setup without runtime network downloads or ambient file assumptions.

Deliverables:

- canonical trusted-setup source, revision, license, byte length, and
  cryptographic digest recorded in the source lock and supply-chain policy;
- explicit decision between a reviewed embedded setup and caller-supplied
  bytes bound to the same pinned digest;
- bounded no-allocator parser with caller-provided storage/scratch where the
  setup cannot be represented statically;
- exact point-count, line/byte-format, compressed-point, curve, subgroup, and
  duplicate/ordering checks required by the selected canonical format;
- typed setup identifier carried by every later commitment or verification
  context so proofs cannot silently use a different ceremony;
- no runtime download, environment lookup, current-directory lookup, or
  implicit user-home fallback.

Verification:

- reproducible fetch/sync command against the pinned official source;
- checksum test over the exact admitted bytes;
- truncated, extended, reordered, malformed-point, wrong-subgroup, and
  wrong-digest negative fixtures;
- tests proving default builds perform no filesystem or network access;
- `cargo deny check`, SBOM review, and source-license review;
- fuzz and pentest gates before tagging.

Exit criteria:

- A KZG verification context can only be constructed from the one documented
  trusted setup (or an explicitly versioned replacement), and setup mismatch
  fails before proof arithmetic.

### v0.61.2 - KZG Field And Polynomial Foundation

Status: planned.

Goal: implement bounded first-party BLS scalar-field and polynomial operations
needed by EIP-4844 proof verification.

Deliverables:

- canonical 32-byte BLS scalar-field element decoding for blob values,
  challenges, and claimed evaluations;
- fixed-width scalar-field arithmetic reusing reviewed BLS constants without
  conflating EIP-2537's unrestricted MSM scalar wire domain;
- explicit 4,096-field-element blob view with exact length and per-element
  canonicality checks;
- bounded polynomial evaluation, roots-of-unity domain, and barycentric/FFT
  operations required by the pinned Ethereum KZG specification;
- caller-provided or fixed scratch storage with documented memory ceilings and
  no hidden allocator requirement;
- transcript/challenge hashing boundary with exact domain bytes and no generic
  hash substitution.

Verification:

- official scalar-field, blob, roots-of-unity, polynomial, and challenge
  vectors from the pinned consensus KZG specification;
- field algebra and polynomial identity properties;
- differential checks against an independent KZG reference implementation;
- malformed blob, non-canonical element, transcript-domain, and scratch-limit
  fuzz coverage;
- release-mode memory, stack, and CPU measurements;
- pentest and retest before tagging.

Exit criteria:

- Every blob polynomial operation is deterministic and bounded over exactly
  the Ethereum evaluation domain, with no ambiguity between field decoding,
  unrestricted MSM scalars, and transcript challenges.
- Commitment/proof success remains unavailable until `v0.61.3`.

### v0.61.3 - KZG Commitment And Proof Verification

Status: planned.

Goal: implement first-party commitment/proof parsing and verification against
the pinned setup with official and independent vectors.

Deliverables:

- canonical 48-byte compressed G1 commitment and proof parsing with infinity,
  curve, and subgroup policy matching the pinned KZG specification;
- first-party `verify_kzg_proof`, `verify_blob_kzg_proof`, and bounded batch
  verification primitives;
- exact Fiat-Shamir challenge derivation and setup-identity binding;
- reuse of reviewed BLS MSM/pairing foundations without exposing EIP-2537 wire
  formats as KZG commitment formats;
- typed verified-proof results that cannot be constructed without a successful
  cryptographic equation;
- deterministic batch-verification challenge generation with no ambient RNG
  or caller-controlled coefficient bypass.

Verification:

- official Ethereum KZG valid/invalid proof and blob-proof vectors;
- independent differential checks against the canonical C KZG implementation
  or another admitted reference used only as an oracle;
- single-proof versus batch-proof equivalence tests;
- wrong setup, commitment, proof, point, value, blob, subgroup, and transcript
  negative tests;
- proof-parser and verification fuzz targets with bounded scratch variants;
- release-mode CPU evidence and pentest/retest gates.

Exit criteria:

- Proof acceptance requires canonical encodings, the pinned setup, the exact
  Ethereum transcript, and a successful first-party pairing equation.
- No public API can turn unverified commitment/proof bytes into a verified
  result token.

### v0.61.4 - KZG Point-Evaluation Precompile Execution

Status: planned.

Goal: admit address `0x0a` execution with versioned-hash binding, canonical
field checks, exact output constants, gas-first execution, and fuzz coverage.

Deliverables:

- exact 192-byte frame parser for versioned hash, evaluation point, claimed
  value, commitment, and proof;
- commitment-to-versioned-hash calculation and equality check using the
  EIP-4844 version byte;
- canonical scalar-field validation for the point and claimed value;
- charged 50,000-gas execution that verifies the KZG proof only after the gas
  meter succeeds;
- exact 64-byte output containing the field-elements-per-blob constant and BLS
  scalar modulus in the specified order;
- output-unchanged and backend-not-reached tests for out-of-gas and malformed
  frames;
- explicit precompile error classification for future CALL gas-burning logic.

Verification:

- official EIP-4844 point-evaluation vectors and invalid-frame cases;
- independent cross-client/reference differential results;
- wrong version, wrong hash, non-canonical field, malformed compressed point,
  invalid proof, short output, and repeated-charge tests;
- fuzz target for all 192-byte subdomains plus arbitrary-length rejection;
- release-mode fixed-gas CPU evidence;
- pentest and retest before tagging.

Exit criteria:

- Address `0x0a` returns only the exact EIP-4844 output after a canonical frame,
  matching versioned hash, successful proof, and mandatory gas charge.

### v0.61.5 - Blob/KZG Execution Integration

Status: planned.

Goal: integrate KZG verification with blob transaction, fee, count, fork, and
block data-availability validation and run the complete claimed fixture set.

Deliverables:

- fork-aware non-empty blob-versioned-hash validation, `0x01` version-byte
  enforcement, and per-transaction/per-block count ceilings;
- blob sidecar model binding each blob, commitment, proof, and transaction
  versioned hash by count and order;
- single and batch blob-proof verification against the pinned setup;
- blob gas used, excess blob gas, base fee, fee-cap, and block-limit validation;
- Cancun-and-later activation rules for blob transactions and the `0x0a`
  precompile;
- block/data-availability validation result types that distinguish missing
  sidecars, malformed data, invalid proofs, and semantic fee/count failures;
- fixture and support matrices that do not imply consensus-layer networking,
  custody, or data retrieval inside this crate.

Verification:

- official EIP-4844 execution and consensus KZG fixture sets at pinned
  revisions;
- valid and adversarial blob transaction/block integration tests;
- wrong count/order/version/hash/proof/setup/fork and blob-gas boundary tests;
- differential validation against at least one independent client and the
  canonical KZG reference;
- batch-size CPU/memory/DoS evidence and fuzz corpus;
- `scripts/checks.sh`, dependency/security gates, pentest, and clean retest.

Exit criteria:

- Claimed blob transactions and blocks are cryptographically and semantically
  validated first-party against explicit fork context and the pinned setup,
  rather than being treated as syntactically decoded data.
- Any consensus-network data-availability behavior remains explicitly outside
  the claim or assigned to a named later release.

### v0.62.0 - Full Execution Fixture Admission

Goal: claim full execution support only where all relevant official fixture
groups pass.

Deliverables:

- `TransactionTests`, `BlockchainTests`, `GenesisTests`, `TrieTests`,
  `DifficultyTests`, EOF tests, and state tests admitted or explicitly scoped
  with unsupported reasons;
- supported fork matrix with per-fork feature flags and exclusions;
- conformance report generated by local scripts;
- release-blocking fixture drift check.

Verification:

- full execution fixture command documented and passing for claimed fork set;
- differential report against at least one independent client or engine for
  the claimed fork set.

Exit criteria:

- Execution-layer support claims are backed by official fixtures and a
  published unsupported-scope list.

## Phase 10: Optional RPC And Signer Boundaries

### v0.63.0 - RPC Dependency Admission

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

### v0.64.0 - RPC Trust Models

Goal: implement trusted, quorum, and verified response models.

Deliverables:

- trust model APIs;
- chain/genesis verification at connection setup;
- response size and batch limits.

Verification:

- malicious RPC fixture tests.

Exit criteria:

- TLS endpoint trust is documented as separate from Ethereum state trust.

### v0.65.0 - RPC Retry And Redaction

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

### v0.66.0 - Signer Interface

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

### v0.67.0 - Local Signer Fallback

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

## Phase 11: Optional Reth And P2P Decisions

### v0.68.0 - Reth Dependency Admission

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

### v0.69.0 - P2P Threat Model Decision

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

## Phase 12: Contract Interaction And Application Standards

### v0.70.0 - ABI Type System

Goal: model Solidity ABI types without pulling contract interaction into the
default core graph.

Deliverables:

- ABI primitive, fixed array, dynamic array, tuple, bytes, string, address, and
  integer domain model;
- strict type parser with recursion and size limits;
- canonical type string rendering;
- selector input model for functions, errors, and events;
- malformed type grammar tests.

Verification:

- `cargo test -p eth-valkyoth-abi`
- ABI fixture tests from pinned official or independently reviewed sources.

Exit criteria:

- ABI types can be parsed and rendered without encoding values yet.

### v0.71.0 - ABI Encode And Decode

Goal: encode and decode ABI values under explicit size limits.

Deliverables:

- ABI value model;
- calldata and return-data encode/decode;
- offset, dynamic-tail, and nested-array validation;
- no unbounded allocation from hostile calldata;
- official and adversarial ABI vectors.

Verification:

- `cargo test -p eth-valkyoth-abi`
- ABI fuzz target builds.

Exit criteria:

- Contract call payloads can be handled without ad hoc downstream encoders.

### v0.72.0 - Contract Event And Error Decoding

Goal: decode logs and revert data with ABI-aware domain types.

Deliverables:

- event topic hashing and indexed parameter policy;
- anonymous-event handling;
- custom error selector decoding;
- revert reason handling with redaction policy;
- malformed log and revert-data tests.

Verification:

- `cargo test -p eth-valkyoth-abi -p eth-valkyoth-protocol`

Exit criteria:

- Contract events and errors can be decoded without implying RPC trust or
  contract semantic validity.

### v0.73.0 - Contract Call Builders

Goal: provide safe contract call builders over ABI primitives.

Deliverables:

- function selector helpers;
- calldata builder;
- return-data decoder;
- no automatic network call or signing behavior;
- examples for read-only call payloads and transaction input construction.

Verification:

- docs examples compile.
- `cargo test -p eth-valkyoth-abi`

Exit criteria:

- Users can build and decode contract calls while networking and signing remain
  explicit separate layers.

### v0.74.0 - Common Token Standards

Goal: add typed helpers for the most common token standards without making
contract wrappers part of the core protocol.

Deliverables:

- ERC-20 function/event selectors and ABI bindings;
- ERC-721 function/event selectors and ABI bindings;
- ERC-1155 function/event selectors and ABI bindings;
- no network, signer, or balance-trust assumptions;
- examples for calldata/log decoding only.

Verification:

- `cargo test -p eth-valkyoth-abi`

Exit criteria:

- Common token interaction helpers are typed convenience APIs over ABI, not
  trusted contract clients.

### v0.75.0 - ENS Namehash And Resolution Primitives

Goal: support ENS primitives and resolver-call construction.

Deliverables:

- ENS name normalization policy decision;
- namehash implementation and vectors;
- resolver and registry ABI call builders;
- text/address/contenthash record decode helpers;
- phishing and Unicode caveats documented.

Verification:

- ENS vector tests.
- docs examples compile.

Exit criteria:

- ENS support is available as explicit contract-call construction and decoding,
  not as hidden RPC behavior.

### v0.76.0 - Permit And Typed Authorization Standards

Goal: support common signature-based contract authorization flows.

Deliverables:

- EIP-2612 permit typed-data helpers;
- Permit2 or explicit deferral decision;
- ERC-721 permit variants where standardized;
- typed-data domain checks wired through the EIP-712 encoder;
- replay-domain and chain mismatch tests.

Verification:

- `cargo test -p eth-valkyoth-abi -p eth-valkyoth-verify`

Exit criteria:

- Permit helpers use the same EIP-712 safety boundary as core signing APIs.

### v0.77.0 - Contract Interface Registry

Goal: expose safe helpers for interface identifiers and contract metadata.

Deliverables:

- ERC-165 interface ID helpers;
- selector collision documentation;
- optional metadata URI decode helpers;
- no HTTP fetching or remote metadata trust by default.

Verification:

- selector and interface ID vector tests.

Exit criteria:

- Contract interface helpers are deterministic local computations only.

### v0.78.0 - ABI And Contract Fuzzing

Goal: fuzz ABI and contract-helper parsers before they are treated as stable.

Deliverables:

- ABI type parser fuzz target;
- ABI calldata/revert/log decode fuzz targets;
- seed corpus for nested offsets and malformed dynamic tails;
- corpus materialization docs.

Verification:

- `cargo check --manifest-path fuzz/Cargo.toml`

Exit criteria:

- Every ABI parser that accepts untrusted bytes has fuzz coverage.

## Phase 13: Consensus, Engine, And Beacon Boundaries

### v0.79.0 - SSZ Codec Boundary

Goal: admit or implement bounded SSZ encoding and decoding for consensus-layer
data.

Deliverables:

- dependency admission or first-party SSZ design;
- bounded container/list/vector decode policy;
- Merkleization boundary decision;
- official consensus-spec vectors.

Verification:

- `cargo test -p eth-valkyoth-consensus`
- `cargo deny check`

Exit criteria:

- Consensus data can be decoded without weakening the execution-layer default
  graph.

### v0.80.0 - Beacon Block And State Headers

Goal: model beacon-chain headers and execution payload references.

Deliverables:

- beacon block/header primitives;
- execution payload header model;
- fork-aware optional fields;
- hash tree root helpers through the SSZ boundary;
- vector tests.

Verification:

- `cargo test -p eth-valkyoth-consensus`

Exit criteria:

- Execution and consensus headers can be linked without claiming full consensus
  validation.

### v0.81.0 - Consensus Light Client Updates

Goal: verify consensus light-client update structures.

Deliverables:

- sync committee domain types;
- finalized header and optimistic header inputs;
- branch verification helpers;
- fork/version context;
- invalid branch and wrong-period tests.

Verification:

- consensus light-client vector tests.

Exit criteria:

- Beacon evidence used by execution-layer callers has an explicit verification
  path.

### v0.82.0 - Engine API Types

Goal: model Engine API request and response types without implementing a client.

Deliverables:

- fork-aware payload attributes;
- execution payload and forkchoice state models;
- payload status and validation-error model;
- JSON serialization policy if serde is admitted;
- execution-apis revision pinned.

Verification:

- `cargo test -p eth-valkyoth-engine`

Exit criteria:

- Engine API data can be represented without opening networking or consensus
  validation claims.

### v0.83.0 - Engine API Validation Helpers

Goal: validate Engine API payload boundaries before optional transport work.

Deliverables:

- forkchoice state consistency checks;
- payload timestamp/fork checks;
- block hash and parent hash domain checks;
- invalid status transition tests.

Verification:

- `cargo test -p eth-valkyoth-engine`

Exit criteria:

- Engine API helpers fail closed on malformed or inconsistent payload context.

### v0.84.0 - Beacon API Boundary

Goal: model Beacon REST API responses as optional, trust-scoped inputs.

Deliverables:

- endpoint trust model;
- response size limits;
- versioned response envelope model;
- no default public endpoint.

Verification:

- `cargo test -p eth-valkyoth-consensus`

Exit criteria:

- Beacon API data is treated as untrusted transport data until verified.

### v0.85.0 - Consensus And Engine Fuzzing

Goal: fuzz consensus and Engine API parsers before claiming support.

Deliverables:

- SSZ fuzz targets;
- Engine/Beacon response fuzz targets if JSON is admitted;
- malformed fork payload seed corpus.

Verification:

- `cargo check --manifest-path fuzz/Cargo.toml`

Exit criteria:

- Consensus and Engine parser boundaries have adversarial coverage.

## Phase 14: Networking, Node, And Operations Boundaries

### v0.86.0 - DevP2P And Discovery Threat Model

Goal: decide and document the exact networking scope before implementation.

Deliverables:

- Discovery v4/v5 decision;
- RLPx handshake threat model;
- peer identity and ENR policy;
- resource, timeout, and process-isolation plan;
- default-off feature policy.

Verification:

- security review of the networking decision document.

Exit criteria:

- No P2P code lands before the attack surface is scoped.

### v0.87.0 - RLPx And Discovery Dependency Admission

Goal: admit networking dependencies behind optional crates only.

Deliverables:

- latest-version, license, feature, MSRV, and maintenance review;
- no default graph expansion;
- test-only loopback transport;
- timeout and message-size policy.

Verification:

- `cargo check --workspace --all-features`
- `cargo deny check`

Exit criteria:

- Networking dependencies are isolated from protocol-core users.

### v0.88.0 - Eth Wire Protocol Messages

Goal: encode and decode Ethereum wire protocol messages with strict limits.

Deliverables:

- status, block header/body, receipt, pooled transaction, and node-data message
  models for selected protocol versions;
- message-size and batch limits;
- fork/protocol-version negotiation policy;
- malformed message tests.

Verification:

- `cargo test -p eth-valkyoth-p2p`
- P2P fuzz target builds.

Exit criteria:

- Wire messages are parser-safe before peer management is attempted.

### v0.89.0 - Snap Protocol Messages

Goal: encode and decode snap-sync protocol messages.

Deliverables:

- account range, storage range, bytecode, and trie-node message models;
- response size and proof-count limits;
- malformed proof and oversized response tests.

Verification:

- `cargo test -p eth-valkyoth-p2p`

Exit criteria:

- Snap data remains untrusted until verified by trie/proof helpers.

### v0.90.0 - Txpool And Mempool Policy

Goal: provide transaction pool policy helpers without running an implicit node.

Deliverables:

- transaction admission policy model;
- replacement and nonce-gap rules;
- local/private transaction redaction rules;
- no automatic rebroadcast;
- adversarial replacement tests.

Verification:

- `cargo test -p eth-valkyoth-txpool`

Exit criteria:

- Mempool helpers are deterministic policy tools, not a hidden networking
  service.

### v0.91.0 - Sync Orchestration Boundaries

Goal: model sync workflows as explicit state machines.

Deliverables:

- header sync state machine;
- body/receipt retrieval state machine;
- snap-sync state machine hooks;
- peer trust and proof-validation boundaries;
- structured progress/error observability hooks with redaction policy;
- cancellation and resource-limit tests.

Verification:

- `cargo test -p eth-valkyoth-sync`

Exit criteria:

- Sync orchestration cannot imply verified state without proof or consensus
  evidence, and operational diagnostics do not leak sensitive transaction or
  endpoint material.

### v0.92.0 - Mining, Builder, And Validator Boundary Decision

Goal: decide what local block production or validator-adjacent support belongs
in this crate family.

Deliverables:

- threat model for block building, MEV/builder APIs, validator duties, and key
  custody;
- explicit implement/defer decision;
- if implemented, split follow-up release plan before 1.0;
- no validator key material in default crates.

Verification:

- security review of the decision document.

Exit criteria:

- Block-production and validator-adjacent scope is either versioned or
  explicitly excluded from 1.0 with documented rationale.

## Phase 15: Production Hardening

### v0.93.0 - Platform Matrix

Goal: verify supported operating systems and targets.

Deliverables:

- Linux, Windows, BSD, macOS, Android, and iOS build notes;
- no_std target checks;
- CI matrix expansion where practical.

Verification:

- documented platform check commands.

Exit criteria:

- Platform support claims match tested evidence.

### v0.94.0 - Kani Formal Verification Harness

Goal: add bounded formal verification evidence for the highest-risk arithmetic,
parser, and typestate invariants.

Deliverables:

- Kani dependency/tooling admission and install/update policy;
- proof harnesses for decode-limit arithmetic and overflow rejection;
- proof harnesses for output-buffer no-mutation-on-error invariants;
- proof harnesses for canonical integer and fixed-width primitive rejection;
- proof harnesses for transaction typestate transitions that must not skip
  required validation proofs;
- proof harnesses for EIP-712 missing/wrong-domain fail-closed behavior;
- documentation that Kani is extra assurance, not a replacement for fuzzing,
  official conformance tests, pentest, or independent audit.

Verification:

- Kani proof command documented and passing for admitted harnesses.
- `scripts/checks.sh`

Exit criteria:

- Formal verification covers selected bounded invariants before API stability
  and external audit remediation begin.

### v0.95.0 - Public API Stability Pass

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

### v0.96.0 - Independent Audit Remediation

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

### v0.97.0 - Release Evidence Dry Run

Goal: prove the release-evidence process before 1.0.

Deliverables:

- signed release manifest draft;
- SBOM;
- provenance notes;
- conformance report;
- dependency compatibility matrix.

Verification:

- release-readiness script for `v0.97.0`.

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
- `scripts/generate-sbom.sh --check`
- `scripts/validate-release-readiness.sh v1.0.0`

Exit criteria:

- `v1.0.0 implementation stop reached. Run pentest for this exact commit.`
