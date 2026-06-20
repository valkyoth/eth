# eth Implementation Plan

Status: planning document

Crate name: `eth`

1.0 target: a serious production-ready Ethereum execution-layer toolkit for
bounded decoding, fork-aware validation, cryptographic verification, explicit
RPC trust policy, optional EVM execution, optional signer isolation, and
optional Reth integration.

## Core Position

`eth` is not a full execution client, consensus client, validator client, or
generic re-export of upstream Ethereum crates. It is a security-oriented
workspace that gives applications stable, testable, explicit boundaries around
Ethereum execution-layer operations.

The first production value is:

- bounded canonical decoding of untrusted Ethereum data;
- explicit chain and fork context;
- stable protocol types and validation states;
- signer and key isolation;
- RPC trust models that do not imply state correctness;
- conformance evidence against pinned upstream specification revisions.

## Non-Negotiable Engineering Rules

- Rust stable `1.96.0`, edition 2024, workspace resolver `3`.
- MSRV is Rust `1.90.0`; compatibility must be checked through `1.96.0`.
- Latest crate and tool versions are checked before dependency or tooling edits.
- Official Ethereum sources are checked before consensus-sensitive
  implementation work; exact revisions are pinned in `spec-lock.toml`.
- Consensus-sensitive behavior is never implemented from memory.
- Core crates are `#![no_std]` and do not depend on network, filesystem, clock,
  TLS, async runtime, signer, Reth, or P2P code.
- Main crate `eth` is a facade over focused crates.
- Third-party crates require review, current-version checks, license checks,
  feature review, and tests before admission.
- First-party protocol-facing crates use `#![forbid(unsafe_code)]`.
- Normal `.rs` files must stay below 500 lines.
- Security documentation, release notes, and test evidence are release
  requirements, not cleanup work.

## Workspace Shape

- `eth-valkyoth-primitives`: no_std chain, block, gas, nonce, address, hash, fork, and
  bounded value primitives.
- `eth-valkyoth-codec`: canonical RLP and typed-envelope decoding policy, exact
  consumption, and decode budgets.
- `eth-valkyoth-protocol`: transaction, block, receipt, withdrawal, log, and fork
  validation states.
- `eth-valkyoth-verify`: sender recovery, replay-domain checks, EIP-712 validation,
  header/hash checks, and MPT proof verification.
- `eth-valkyoth-evm`: optional REVM adapter boundary with explicit fork, block,
  transaction, snapshot, limit, and commit policy.
- `eth-valkyoth-rpc`: optional RPC policy over admitted provider transports.
- `eth-valkyoth-sanitization`: optional bridge to the `sanitization` crate for
  secret-bearing Ethereum data.
- `eth-valkyoth-derive`: optional derive macros for explicit sanitization users.
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

Release gate:

- relevant execution-spec and EIP revisions are pinned in `spec-lock.toml`;
- cargo-fuzz infrastructure is bootstrapped before the first RLP parser lands;
- official and adversarial RLP fixtures pass;
- fuzz targets exist for every decoder;
- malformed input never panics.

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

Implement replay-domain checks, sender recovery, EIP-712 safety rules, header
hash consistency, transaction and receipt inclusion proofs, and MPT proof
verification.

Release gate:

- relevant execution-spec, EIP, and fixture revisions are pinned in
  `spec-lock.toml`;
- official and independent proof fixtures pass;
- invalid proofs fail closed;
- signature and domain errors never expose secret material.

## Phase 6: Optional REVM Adapter

Admit REVM behind `eth-valkyoth-evm`. Require explicit fork/spec ID, block environment,
transaction environment, state snapshot, execution limits, and commit policy.

Release gate:

- Ethereum state tests agree with reference outputs for claimed forks;
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

## Phase 8: Optional Reth And P2P Integration

Add Reth and P2P only after a threat-model expansion. Keep Reth type conversion
at the adapter boundary and avoid making node internals part of core protocol
APIs.

Release gate:

- separate fuzz corpus and load tests;
- reviewed dependency expansion;
- no Reth or P2P dependency in the default graph.

## Phase 9: 1.0 Production Readiness

Complete independent security review, conformance matrix, SBOM, provenance,
signed release manifest, release notes, supported fork matrix, and migration
guidance.

Release gate:

- no unresolved critical or high dependency/advisory/audit findings;
- official conformance suites pass for every claimed feature;
- all supported Rust versions from `1.90.0` through `1.96.0` are checked.
