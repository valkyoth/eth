# eth Implementation Plan

Status: planning document

Crate name: `eth`

1.0 target: a complete security-oriented Ethereum toolkit with bounded wire
handling, owned SDK models, first-party historical and current execution,
typed providers and transaction workflows, wallet and account-abstraction
support, contract tooling, persistent canonical-chain storage, consensus and
Engine integration, a complete light client, a production beacon node and
validator client, execution and consensus networking/sync, transactional
slashing protection, builder integration, stateless execution, and explicit
optional ecosystem adapters.

## Core Position

`eth` is not a generic re-export of upstream Ethereum crates and must not hide
networking, signing, consensus, execution, or node behavior behind defaults. It
is a security-oriented workspace that gives applications stable, testable,
explicit boundaries around Ethereum operations. Core Ethereum behavior should
be first-party where practical. Third-party crates are acceptable only as
reviewed optional backends, reference implementations, compatibility adapters,
or explicitly justified cryptographic backends with conformance evidence and a
boundary. The 1.0 roadmap includes optional contract, execution-client,
consensus-client, validator, Engine API, networking, sync, builder, and node
tracks, but the default facade remains conservative and explicit.

The first production value is:

- bounded canonical decoding of untrusted Ethereum data;
- explicit chain and fork context;
- stable protocol types and validation states;
- owned, interoperable application models and task-oriented workflows;
- first-party execution-layer state transition, trie-root construction, and
  block-validity support for every claimed fork;
- signer, wallet, key-custody, multisig, and account-abstraction isolation;
- RPC trust models that do not imply state correctness;
- typed provider transports, simulation, broadcast, confirmation, replacement,
  and reorg-aware transaction lifecycle support;
- contract ABI, artifacts, bindings, deployment, events, and common standard
  helpers that do not imply contract trust;
- persistent storage, canonical-chain import, pruning/archive policy, and
  supervised client-runtime boundaries;
- consensus, Engine, light-client, full beacon-node, validator, slashing,
  builder, execution/consensus networking, txpool, sync, witness, and
  stateless-execution support with explicit trust and resource policies;
- conformance evidence against pinned upstream specification revisions.

## Non-Negotiable Engineering Rules

- Rust stable `1.97.1`, edition 2024, workspace resolver `3`.
- MSRV is Rust `1.90.0`; compatibility must be checked through `1.97.1`.
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
  opcode execution pass; `v0.42.0` adds fork-scoped gas metering for the
  admitted subset; `v0.43.0` adds bounded state access for the currently
  claimed fork range; `v0.43.1` and `v0.43.2` add the historical fork matrix
  and pre-Berlin state gas schedules so older forks are implemented explicitly
  before more stateful execution layers depend on them; `v0.44.0` adds the
  call/create safety boundary with explicit frame, return-data, and journal
  policy; `v0.45.0` adds the fork-aware precompile registry, bounded
  precompile planning, and dependency-free identity execution; `v0.46.0` adds
  dependency-free SHA-256 and RIPEMD-160 precompile execution; `v0.47.0` adds
  ECRECOVER execution through explicit caller-provided secp256k1 and Keccak
  boundaries; `v0.48.0` adds bounded first-party ModExp parsing, fork-aware
  gas, execution, and fuzzing; `v0.49.0` adds dependency-free BN254 add/mul
  execution; `v0.50.0` adds the BN254 pairing frame boundary with empty-input
  execution and non-empty fail-closed behavior; `v0.50.1` adds G2 subgroup
  validation and a precomputed twist coefficient; `v0.50.2` adds the Fp6/Fp12
  tower foundation; `v0.50.3` adds validated tuple streaming and the
  atomic gas-meter charging for dispatcher-facing pairing execution; `v0.50.4`
  adds the line-function foundation and extends dispatcher-facing gas-meter
  charging to ModExp and BN254 add/mul plan execution; `v0.50.5` adds the
  internal Miller-loop accumulator; `v0.50.6` adds sparse Miller-loop
  multiplication and gas/CPU benchmark evidence; `v0.50.7` adds bounded final
  exponentiation behind the fail-closed pairing boundary; `v0.50.8` adds
  Frobenius Q1/-Q2 point mapping; `v0.50.9` completes the projective post-loop
  line carrier; `v0.50.10` admits non-empty pairing results; `v0.51.0` adds
  first-party dependency-free EIP-152 BLAKE2F execution and the optimized
  BN254 final-exponentiation remediation; `v0.52.0` fixes the remaining
  KZG/BLS frame, output, gas, conformance, and backend-admission contracts;
  `v0.52.1` defines BLS12-381 wire domains, `v0.52.2..=v0.52.10` close
  consensus/resource/precompile gaps, `v0.52.11..=v0.52.18` build first-party
  BLS12-381 execution, `v0.52.19..=v0.52.21` freeze architecture,
  resource-governor, and cryptographic-provider contracts, and
  `v0.52.22..=v0.52.34` implement/audit first-party core crypto plus shared
  cross-format accounting, session-safe clocks, atomically reserved bounded
  object/peer evidence, early secp proof, signing/transport separation,
  non-forgeable constant-size validation contexts, authority-tagged
  consensus/wire/operational limit contracts, and conserved hierarchical
  evidence capabilities for nested and batch validation, explicit operational
  collection modes, and immutable optional-sink access; and
  `v0.77.0` through `v0.81.0` build first-party KZG/blob verification before
  later releases claim complete affected-fork execution.
- `eth-valkyoth-sdk`: optional owned models, prelude, builders, and high-level
  workflows over the focused core crates.
- `eth-valkyoth-rpc`: typed RPC methods and explicit response trust policy.
- `eth-valkyoth-provider`: optional runtime-neutral provider, transport,
  middleware, subscription, and transaction-lifecycle orchestration.
- `eth-valkyoth-abi`: optional ABI, contract-call, event, error, and common
  contract-standard helpers.
- `eth-valkyoth-ssz`: complete no_std SSZ, Merkleization, generalized-index,
  proof, and incremental-root support.
- `eth-valkyoth-bls`: no_std BLS signing, verification, aggregation, and batch
  boundary with explicit reviewed backend admission where required.
- `eth-valkyoth-consensus-types`: fork-typed beacon state, block, operation,
  sidecar, duty, and API domains.
- `eth-valkyoth-consensus-config`: network presets, chain/genesis
  configurations, fork schedules, digests, domains, and generated fork modules.
- `eth-valkyoth-consensus-transition`: complete per-slot, per-epoch, operation,
  execution, data-availability, and fork-upgrade state transition.
- `eth-valkyoth-fork-choice`: transactional LMD-GHOST/Casper FFG, optimistic
  execution, persistence, recovery, and head/finality service.
- `eth-valkyoth-consensus`: optional facade over consensus primitives,
  transition, fork choice, light client, and client services.
- `eth-valkyoth-engine`: Engine API type, validation, server, and service
  boundary.
- `eth-valkyoth-engine-client`: authenticated multi-execution-client
  coordination, health, failover, and invalidation propagation.
- `eth-valkyoth-beacon-store`: hot/finalized blocks, states, sidecars,
  snapshots, reconstruction, pruning, migration, and repair.
- `eth-valkyoth-consensus-network`: optional discv5, ENR, libp2p, GossipSub,
  subnet, ReqResp, validation, scoring, and backpressure service.
- `eth-valkyoth-beacon-sync`: checkpoint, weak-subjectivity, head/range,
  backfill, optimistic, and PeerDAS synchronization.
- `eth-valkyoth-data-availability`: cells, data columns, custody,
  availability evidence, reconstruction, retention, and admission.
- `eth-valkyoth-beacon-node`: optional orchestration of transition, fork
  choice, storage, networking, sync, Engine, data availability, and APIs.
- `eth-valkyoth-validator`: duty scheduling, block production, attestations,
  aggregation, sync committees, lifecycle operations, and safety policy.
- `eth-valkyoth-slashing-protection`: transactional slashability kernel,
  durable record-before-release store, and EIP-3076 interchange.
- `eth-valkyoth-validator-signer`: consensus signing packages and
  local/remote/HSM signer orchestration coupled to slashing protection.
- `eth-valkyoth-keymanager`: official Keymanager REST API and validator-key
  lifecycle service.
- `eth-valkyoth-builder`: Builder API, relay multiplexing, blinded blocks,
  reveal validation, safe local fallback, and future PBS boundary.
- `eth-valkyoth-p2p`: optional DevP2P/RLPx, discovery, eth, and snap message
  boundary.
- `eth-valkyoth-txpool`: optional transaction-pool policy helpers.
- `eth-valkyoth-sync`: optional sync orchestration state machines.
- `eth-valkyoth-storage`: database traits, schemas, state/chain stores,
  migrations, snapshots, pruning, and archive policies.
- `eth-valkyoth-chain`: canonical import, reorg, fork-choice/head state,
  orphan, payload, and invalidation orchestration.
- `eth-valkyoth-runtime`: optional supervised operational tasks without
  selecting a runtime for core crates.
- `eth-valkyoth-witness`: proof-format abstraction, witness construction,
  stateless execution, and commitment-scheme evolution.
- `eth-valkyoth-sanitization`: optional bridge to the `sanitization` crate for
  secret-bearing Ethereum data.
- `eth-valkyoth-derive`: optional derive macros for explicit sanitization users
  and, after review, public RLP derives that preserve bounded decode and
  primitive-domain invariants.
- `eth-valkyoth-signer`: optional signer isolation and domain-specific signing APIs.
- `eth-valkyoth-wallet`: optional local, HD, keystore, remote, hardware,
  multisig, account-abstraction, and delegated-account workflows.
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

Full consensus-client maintenance must also track consensus-spec stable and
experimental forks, Beacon API, Keymanager API, Builder API, EIP-3076, and Hive
interoperability. New consensus work becomes a named release only after the
relevant source is pinned, implemented, tested, and pentested.

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

## Phase 7: Owned SDK And Shared Domains

Build general integer/byte/hash domains, owned transaction/block/state models,
lossless representation conversions, bounded allocation conveniences,
structured errors, payload-bound typestates, a fork-rule model that separates
identity from activation and parameters, and one shared protocol/execution
domain vocabulary. Add a curated facade prelude and optional ecosystem
adapters only after the first-party models are authoritative.

Release gate:

- owned/borrowed/validated/RPC/execution conversion matrices pass;
- no validation evidence is detached from its payload;
- fork behavior does not depend on enum ordering;
- features and README dependency snippets are generated and checked.

## Phase 8: Complete First-Party Execution

Finish semantic transaction/header/block validity, genesis import, complete
state transition and journaling, receipts/logs/withdrawals, trie construction,
first-party KZG/blob verification, EOF, current-fork ingestion, official
execution fixture coverage, differential evidence, tracing, and deterministic
simulation. Historical and current forks remain equally explicit.

Release gate:

- every claimed fork passes its official fixtures with no unexplained skip;
- state roots and execution outcomes match independent clients;
- unsupported future behavior fails closed;
- performance and resource ceilings are documented and enforced.

## Phase 9: Providers Transactions Signers And Contracts

Build typed runtime-neutral RPC transports, HTTP/WS/IPC/EIP-1193 adapters,
batching, cancellation, subscriptions, backpressure, middleware, quorum and
proof-backed trust models, and the complete transaction lifecycle. Add local,
HD, keystore, remote, hardware, multisig, Safe, ERC-4337, session-key, and
EIP-7702 wallet workflows. Complete ABI, artifact, codegen, deployment, event,
error, multicall, standard-contract, ENS, and permit tooling.

Release gate:

- malicious provider, disconnect, reorg, replacement, and key-domain fixtures
  pass;
- secret-bearing features remain opt-in and sanitized;
- generated contract and wallet APIs preserve validation and chain context;
- end-to-end workflows pass against self-managed local nodes.

## Phase 10: Storage Canonical Chain And Runtime

Add transactional database traits, chain/state schemas, crash consistency,
migrations, snapshots, pruning/archive/history-expiry modes, canonical import
and reorg, head/fork-choice/orphan tracking, payload invalidation, and bounded
operational supervision.

Release gate:

- process-kill, torn-write, migration, snapshot, prune, and deep-reorg tests
  pass;
- committed blocks and state/index roots remain atomically consistent;
- runtime tasks have explicit shutdown, restart, metrics, and resource policy.

## Phase 11: Consensus Primitives Light Client And Execution Networking

Complete SSZ, forked beacon models, all admitted Engine API versions, Engine
client/server services, Beacon API, weak-subjectivity bootstrap, BLS sync
committee verification, rotation, persistence, finality scoring, execution
proof binding, checkpoint recovery, PeerDAS primitives, plus execution-layer
Discovery/RLPx, eth/snap, peers, request scheduling, txpool, sync, and
Portal/history acquisition. These are foundations, not a complete beacon node
or validator client.

Release gate:

- official consensus, Engine, light-client, and wire fixtures pass;
- Byzantine peer/provider and restart/recovery simulations pass;
- all live networking remains outside the default graph.

## Phase 12: Statelessness And Commitment Evolution

Introduce commitment-neutral proof types, execution witnesses, MPT witness
construction, stateless execution, the officially selected successor
commitment backend, state/history evolution, an explicit ZK proof boundary,
and automated official-fork drift detection.

Release gate:

- stateful and stateless outcomes match;
- historical and successor commitment eras coexist under explicit fork rules;
- no unfinished commitment or ZK backend can be mistaken for consensus-valid.

## Phase 13: Full Beacon Node And Validator Client

Extend consensus primitives into complete fork-typed state transition,
transactional LMD-GHOST/Casper FFG fork choice, hot/finalized beacon storage,
operation pools, separate consensus libp2p/discv5/GossipSub/ReqResp networking,
checkpoint/head/backfill/optimistic/PeerDAS sync, authenticated multi-engine
coordination, availability tracking, beacon and validator API servers, and
coherent beacon-node orchestration.

Build the complete validator duty engine with proposer, attester, aggregator,
sync-committee, lifecycle, doppelganger, quorum, and optimistic-node refusal
policy. Add transactional record-before-release slashing protection, EIP-3076,
local/remote/HSM signers, Keymanager API, builder relay multiplexing, safe local
fallback, deterministic simulation, Hive interoperability, multi-execution
client testing, and a long-running slash-free validator testnet.

Release gate:

- all official stable-fork state-transition, fork-choice, networking, sync,
  honest-validator, PeerDAS, Beacon API, Keymanager, and Builder API suites
  pass with no unexplained skips;
- weak-subjectivity mismatches fail fatally and optimistic nodes cannot perform
  validator duties;
- invalid fork-choice handlers and failed state transitions leave stores
  unchanged;
- slashing records are durable before signatures are released;
- Hive/multi-client and long-running validator evidence is published.

## Phase 14: 1.0 Production Readiness

Complete platform and performance matrices, Kani proofs, Miri/sanitizer gates,
semver/feature compatibility checks, task-oriented documentation, independent
core/execution/SDK/network/consensus/validator audits, remediation, SBOM,
provenance, signed release manifest, supported-fork matrix, and migration
guidance.

Release gate:

- bounded Kani formal verification harnesses pass for selected arithmetic,
  parser, typestate, state-transition, fork-choice, slashing, and duty-safety
  invariants;
- no unresolved critical or high dependency/advisory/audit findings;
- official conformance suites pass for every claimed feature;
- Hive, multi-execution-client, and long-running validator gates pass;
- Rust `1.90.0` through the newest supported compatibility release pass
  all-feature workspace checks;
- the full release gate passes on pinned stable Rust `1.97.1`.

Kani is extra assurance. It does not replace fuzzing, official Ethereum
conformance tests, pentest, cargo-audit/cargo-deny, or independent security
review.
