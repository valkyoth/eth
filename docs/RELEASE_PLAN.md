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
| Formal verification evidence was not scheduled. | Added `v0.71.0 - Kani Formal Verification Harness` as extra assurance, not a replacement for fuzzing, conformance tests, pentest, or audit. |
| ABI encoding, Engine API, SSZ, and DevP2P/RLPx were marked deferred. | Added `v0.47.0` through `v0.69.0` feature tracks so they are versioned before 1.0. |
| ENS and common ERC/application standards were not scheduled. | Added `v0.53.0` through `v0.55.0` for ENS and common contract standards. |
| Node-level sync, txpool, mining/validator boundaries, and observability were not scheduled. | Added `v0.65.0` through `v0.69.0` with explicit library-boundary scope and validation gates. |

## Phase 0: Repository And Release Discipline

### v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.96.1` pinned.
- Rust `1.90.0` through `1.96.1` compatibility policy.
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

Status: tagged as `v0.28.0`.

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

Status: tagged as `v0.29.0`.

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

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

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

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

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

Status: implementation, pentest remediation, and clean retest complete; waiting
for final GitHub checks before tagging.

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

Status: implementation ready for external pentest.

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

Goal: verify transaction and receipt inclusion proofs.

Deliverables:

- transaction proof verification;
- receipt proof verification;
- proof root hashing uses the `eth-valkyoth-hash` trait boundary;
- transaction hashes, receipt roots, and proof roots use distinct domain
  newtypes instead of raw `B256`;
- invalid proof fixtures.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Inclusion proof APIs distinguish malformed, absent, and wrong-root proofs.

### v0.33.0 - Account And Storage Proofs

Goal: verify account and storage proofs against trusted roots.

Deliverables:

- account proof verification;
- storage proof verification;
- account and storage proof root hashing uses the `eth-valkyoth-hash` trait
  boundary;
- account and storage proof roots use distinct domain newtypes instead of raw
  `B256`;
- missing-node and wrong-value tests.

Verification:

- `cargo test -p eth-valkyoth-verify`

Exit criteria:

- Verified RPC state has a cryptographic proof path separate from trusted RPC.

## Phase 6: Conformance And Test Infrastructure

### v0.34.0 - Spec Lock And Fixture Import

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

### v0.35.0 - Execution Test Harness

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

### v0.37.0 - REVM Dependency Admission

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

### v0.38.0 - Explicit Execution Environment

Goal: execute with explicit fork, block, transaction, and snapshot inputs.

Deliverables:

- environment conversion;
- state snapshot trait;
- execution result model.

Verification:

- `cargo test -p eth-valkyoth-evm`

Exit criteria:

- Simulation reports the exact state and fork configuration used.

### v0.39.0 - Bounded Gas Estimation

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

### v0.40.0 - RPC Dependency Admission

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

### v0.41.0 - RPC Trust Models

Goal: implement trusted, quorum, and verified response models.

Deliverables:

- trust model APIs;
- chain/genesis verification at connection setup;
- response size and batch limits.

Verification:

- malicious RPC fixture tests.

Exit criteria:

- TLS endpoint trust is documented as separate from Ethereum state trust.

### v0.42.0 - RPC Retry And Redaction

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

### v0.43.0 - Signer Interface

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

### v0.44.0 - Local Signer Fallback

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

### v0.45.0 - Reth Dependency Admission

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

### v0.46.0 - P2P Threat Model Decision

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

## Phase 10: Contract Interaction And Application Standards

### v0.47.0 - ABI Type System

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

### v0.48.0 - ABI Encode And Decode

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

### v0.49.0 - Contract Event And Error Decoding

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

### v0.50.0 - Contract Call Builders

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

### v0.51.0 - Common Token Standards

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

### v0.52.0 - ENS Namehash And Resolution Primitives

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

### v0.53.0 - Permit And Typed Authorization Standards

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

### v0.54.0 - Contract Interface Registry

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

### v0.55.0 - ABI And Contract Fuzzing

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

## Phase 11: Consensus, Engine, And Beacon Boundaries

### v0.56.0 - SSZ Codec Boundary

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

### v0.57.0 - Beacon Block And State Headers

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

### v0.58.0 - Consensus Light Client Updates

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

### v0.59.0 - Engine API Types

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

### v0.60.0 - Engine API Validation Helpers

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

### v0.61.0 - Beacon API Boundary

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

### v0.62.0 - Consensus And Engine Fuzzing

Goal: fuzz consensus and Engine API parsers before claiming support.

Deliverables:

- SSZ fuzz targets;
- Engine/Beacon response fuzz targets if JSON is admitted;
- malformed fork payload seed corpus.

Verification:

- `cargo check --manifest-path fuzz/Cargo.toml`

Exit criteria:

- Consensus and Engine parser boundaries have adversarial coverage.

## Phase 12: Networking, Node, And Operations Boundaries

### v0.63.0 - DevP2P And Discovery Threat Model

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

### v0.64.0 - RLPx And Discovery Dependency Admission

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

### v0.65.0 - Eth Wire Protocol Messages

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

### v0.66.0 - Snap Protocol Messages

Goal: encode and decode snap-sync protocol messages.

Deliverables:

- account range, storage range, bytecode, and trie-node message models;
- response size and proof-count limits;
- malformed proof and oversized response tests.

Verification:

- `cargo test -p eth-valkyoth-p2p`

Exit criteria:

- Snap data remains untrusted until verified by trie/proof helpers.

### v0.67.0 - Txpool And Mempool Policy

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

### v0.68.0 - Sync Orchestration Boundaries

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

### v0.69.0 - Mining, Builder, And Validator Boundary Decision

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

## Phase 13: Production Hardening

### v0.70.0 - Platform Matrix

Goal: verify supported operating systems and targets.

Deliverables:

- Linux, Windows, BSD, macOS, Android, and iOS build notes;
- no_std target checks;
- CI matrix expansion where practical.

Verification:

- documented platform check commands.

Exit criteria:

- Platform support claims match tested evidence.

### v0.71.0 - Kani Formal Verification Harness

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

### v0.72.0 - Public API Stability Pass

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

### v0.73.0 - Independent Audit Remediation

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

### v0.74.0 - Release Evidence Dry Run

Goal: prove the release-evidence process before 1.0.

Deliverables:

- signed release manifest draft;
- SBOM;
- provenance notes;
- conformance report;
- dependency compatibility matrix.

Verification:

- release-readiness script for `v0.74.0`.

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
