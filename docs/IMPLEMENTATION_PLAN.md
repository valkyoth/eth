# eth Implementation Plan

Status: planning document

Crate name: `eth`

1.0 target: a serious production-ready Ethereum toolkit for bounded decoding,
fork-aware validation, cryptographic verification, first-party execution-layer
state transition and block validation, contract ABI helpers, consensus and
Engine API boundaries, explicit RPC trust policy, optional signer isolation,
optional networking/sync boundaries, and optional Reth integration.

## Core Position

`eth` is not a generic re-export of upstream Ethereum crates and must not hide
networking, signing, consensus, execution, or node behavior behind defaults. It
is a security-oriented workspace that gives applications stable, testable,
explicit boundaries around Ethereum operations. Core Ethereum behavior should
be first-party where practical. Third-party crates are acceptable only as
reviewed optional backends, reference implementations, compatibility adapters,
or explicitly justified cryptographic backends with conformance evidence and a
boundary. The 1.0 roadmap includes optional contract, consensus, Engine API,
networking, sync, and node-adjacent tracks, but the default facade remains
conservative and explicit.

The first production value is:

- bounded canonical decoding of untrusted Ethereum data;
- explicit chain and fork context;
- stable protocol types and validation states;
- first-party execution-layer state transition, trie-root construction, and
  block-validity support for every claimed fork;
- signer and key isolation;
- RPC trust models that do not imply state correctness;
- contract ABI and common standard helpers that do not imply contract trust;
- optional consensus, Engine, networking, and sync boundaries with explicit
  trust and resource policies;
- conformance evidence against pinned upstream specification revisions.

## Non-Negotiable Engineering Rules

- Rust stable `1.96.1`, edition 2024, workspace resolver `3`.
- MSRV is Rust `1.90.0`; compatibility must be checked through `1.96.1`.
- Latest crate and tool versions are checked before dependency or tooling edits.
- Official Ethereum sources are checked before consensus-sensitive
  implementation work; exact revisions are pinned in `spec-lock.toml`.
- Consensus-sensitive behavior is never implemented from memory.
- Core crates are `#![no_std]` and do not depend on network, filesystem, clock,
  TLS, async runtime, signer, Reth, or P2P code.
- Main crate `eth` is a facade over focused crates.
- Third-party crates require review, current-version checks, license checks,
  feature review, and tests before admission.
- Third-party crates that implement core Ethereum behavior require an explicit
  boundary and classification: optional backend, reference-only,
  compatibility adapter, temporary debt, or documented cryptographic exception.
- Core protocol claims must not depend solely on an upstream implementation
  unless the release plan contains a replacement, differential, or audit
  milestone for that dependency.
- First-party protocol-facing crates use `#![forbid(unsafe_code)]`.
- Normal `.rs` files must stay below 500 lines.
- Security documentation, release notes, and test evidence are release
  requirements, not cleanup work.

## Workspace Shape

- `eth-valkyoth-primitives`: no_std chain, block, gas, nonce, address, hash, fork, and
  bounded value primitives.
- `eth-valkyoth-hash`: no_std Keccak-256 trait boundary for caller-provided
  hashing implementations plus optional reviewed backend admission outside the
  default graph.
- `eth-valkyoth-codec`: canonical RLP and typed-envelope decoding policy, exact
  consumption, decode budgets, and the single source of truth for RLP
  canonicality rules.
- `eth-valkyoth-protocol`: transaction, set-code transaction, block, receipt,
  withdrawal, log, and fork validation states.
- `eth-valkyoth-verify`: transaction signing hashes, full transaction
  signature validation, sender recovery, replay-domain checks, EIP-712
  validation and typed-data hashing, header/hash checks, and MPT proof
  verification.
- `eth-valkyoth-evm`: optional execution boundary. REVM may be admitted only as
  a temporary/reference adapter, while production execution moves through the
  first-party audited native EVM plan.
- `eth-valkyoth-evm-core`: no_std-first native EVM engine crate. `v0.40.0`
  starts it with dependency-free word, stack, memory, opcode, fork,
  program-counter, and error domains. `v0.41.0` adds the first bounded basic
  opcode execution pass; later releases add gas, state, call-frame,
  precompile, and commit policy.
- `eth-valkyoth-rpc`: optional RPC policy over admitted provider transports.
- `eth-valkyoth-abi`: optional ABI, contract-call, event, error, and common
  contract-standard helpers.
- `eth-valkyoth-consensus`: optional SSZ, beacon, and light-client boundaries.
- `eth-valkyoth-engine`: optional Engine API type and validation boundary.
- `eth-valkyoth-p2p`: optional DevP2P/RLPx, discovery, eth, and snap message
  boundary.
- `eth-valkyoth-txpool`: optional transaction-pool policy helpers.
- `eth-valkyoth-sync`: optional sync orchestration state machines.
- `eth-valkyoth-sanitization`: optional bridge to the `sanitization` crate for
  secret-bearing Ethereum data.
- `eth-valkyoth-derive`: optional derive macros for explicit sanitization users
  and, after review, public RLP derives that preserve bounded decode and
  primitive-domain invariants.
- `eth-valkyoth-signer`: optional signer isolation and domain-specific signing APIs.
- `eth-valkyoth-reth`: optional Reth adapter boundary.
- `eth-valkyoth-testkit`: fixtures, adversarial inputs, conformance helpers, and
  regression utilities.
- `eth`: facade crate that re-exports stable admitted surfaces.

## Spec Source Discipline

Every protocol milestone begins with a source check against the official
Ethereum repositories documented in [Spec Source Policy](spec-source-policy.md).
The milestone must pin exact upstream revisions in `spec-lock.toml`, import
only required fixtures or spec files into the configured external reference
store, and update [Spec Matrix](SPEC_MATRIX.md) before claiming support.

If a behavior is consensus-sensitive and no pinned source or fixture exists,
implementation stops until the ambiguity is documented and reviewed.

Execution and fork-aware maintenance must include an advisory upstream checker.
The checker tracks latest REVM, official Ethereum hardfork/spec sources, and
execution fixture revisions so new fork rules or execution changes become a
planned maintenance release instead of an accidental drift.

## Phase 1: Repository Foundation

Create the workspace, policy docs, local check scripts, CI, dependency policy,
release notes, and first no_std crate boundaries.

Release gate:

- `scripts/checks.sh`
- `scripts/release_0_1_gate.sh`
- `cargo check --workspace --all-features`

## Phase 2: Primitives And Decode Budgets

Implement domain newtypes, chain/fork specs, typed decode limits, exact
consumption checks, and resource-exhaustion policy.

Release gate:

- all primitive constructors and decode-budget branches tested;
- no network, signer, Reth, EVM, or P2P dependency in the default graph.

## Phase 3: Canonical RLP And Typed Transactions

Admit current Alloy codec/primitives crates only after version, license,
feature, and no_std review. Implement bounded canonical RLP, EIP-2718 typed
transaction envelopes, canonical integer rejection, and trailing-data rejection.
Before transaction envelope work grows, remove duplicated primitive/codec
canonical integer parsing and add a primitive RLP bridge so domain types do not
require ad hoc caller glue.

Release gate:

- relevant execution-spec and EIP revisions are pinned in `spec-lock.toml`;
- cargo-fuzz infrastructure is bootstrapped before the first RLP parser lands;
- official and adversarial RLP fixtures pass;
- fuzz targets exist for every decoder;
- malformed input never panics;
- primitives delegate RLP integer canonicality to codec helpers rather than
  maintaining duplicated parsing logic;
- primitive RLP encode/decode helpers exist before transaction structs depend
  on those domains.

## Hashing Boundary

Keccak-256 is required for transaction hashes, recovered sender addresses,
execution headers, receipts, and proof verification. The implementation must
decide the hashing boundary before those features land.

The preferred default is a small no_std trait boundary so callers can provide
hardware, platform, WASM, or audited software hashing without forcing one hash
crate into the core graph. Any built-in implementation remains an explicitly
admitted dependency with license, feature, maintenance, and audit evidence.

`v0.9.3` establishes that boundary in `eth-valkyoth-hash` and documents the
dependency decision in [Keccak Boundary](keccak-boundary.md). `v0.10.0`
establishes the RLP fuzz harness baseline, committed hex seed corpus, and crash
reproduction process in [Fuzzing](fuzzing.md) before transaction envelope
parsers are admitted.

## Phase 4: Fork-Aware Validation

Implement transaction, header, receipt, withdrawal, and fork validation states
with explicit `ChainId`, `ForkSpec`, block number, timestamp, and validation
context.

Release gate:

- relevant execution-spec and EIP revisions are pinned in `spec-lock.toml`;
- wrong-chain, wrong-fork, unsupported type, and noncanonical encodings are
  rejected deterministically;
- public protocol APIs do not infer "latest" rules globally.

## Phase 5: Verification And Proofs

Implement replay-domain checks, transaction signing hashes, full transaction
signature validation, sender recovery, EIP-712 safety rules, EIP-712 typed-data
hashing, the optional EIP-712 JSON typed-data parser boundary, header hash
consistency, transaction and receipt inclusion proofs, and MPT proof
verification.

Release gate:

- relevant execution-spec, EIP, and fixture revisions are pinned in
  `spec-lock.toml`;
- official and independent proof fixtures pass;
- invalid proofs fail closed;
- raw digest signing is not the primary API for standard transaction or EIP-712
  flows;
- signature and domain errors never expose secret material.

## Phase 6: Optional Execution Boundary And Native EVM

Review REVM only as a temporary/reference adapter behind `eth-valkyoth-evm`.
The long-term production path is a first-party native EVM engine and full
execution-layer state transition built in small audited releases. Require
explicit fork/spec ID, block environment, transaction environment, state
snapshot, execution limits, gas accounting, state commit policy, genesis
handling, transaction semantic validity, block validity, trie-root
construction, blob/KZG boundary decisions, official execution fixture evidence,
and differential evidence where available.

Release gate:

- core dependency independence audit exists before deeper execution work;
- REVM cannot enter the graph unless dependency, MSRV, license, feature, and
  cargo-deny policy pass;
- native engine milestones are covered by official Ethereum state tests for
  claimed forks;
- full execution claims pass the relevant `TransactionTests`, `BlockchainTests`,
  `GenesisTests`, `TrieTests`, `DifficultyTests`, EOF tests, and state tests for
  the claimed fork set;
- differential tests compare claimed behavior against at least one independent
  implementation when available;
- gas estimation is bounded by execution count, gas cap, and timeout policy.

## Phase 7: Optional RPC And Signer Boundaries

Admit RPC transports and signer backends only behind features. Implement
endpoint allowlists, no implicit public endpoint fallback, no automatic
transaction rebroadcast, redacted logs, trusted/quorum/verified response
models, and external-signer-first APIs.

Release gate:

- malicious RPC fixtures pass;
- secrets, bearer tokens, calldata, and raw signed transactions are redacted by
  default;
- local signer remains non-default.

## Phase 8: Optional Reth, Contract, Consensus, Networking, And Node Tracks

Add Reth, ABI/contract helpers, consensus/Engine boundaries, P2P, txpool, and
sync only after threat-model expansion and dependency review. Keep every
network, node, signer, and consensus adapter outside the default graph unless a
future major version explicitly changes that policy.

Release gate:

- separate fuzz corpus and load tests;
- reviewed dependency expansion;
- no Reth, P2P, consensus, Engine, txpool, sync, RPC, or signer dependency in
  the default graph.

## Phase 9: 1.0 Production Readiness

Complete independent security review, conformance matrix, SBOM, provenance,
signed release manifest, release notes, supported fork matrix, and migration
guidance.

Release gate:

- bounded Kani formal verification harnesses pass for selected arithmetic,
  parser, and typestate invariants;
- no unresolved critical or high dependency/advisory/audit findings;
- official conformance suites pass for every claimed feature;
- all supported Rust versions from `1.90.0` through `1.96.1` are checked.

Kani is extra assurance. It does not replace fuzzing, official Ethereum
conformance tests, pentest, cargo-audit/cargo-deny, or independent security
review.
