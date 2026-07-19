# Current Status

Release snapshot: `v0.52.3` implementation complete; independent pentest
pending.

This document summarizes what the workspace can do now. The
[Specification Matrix](SPEC_MATRIX.md) is the source of truth for exact
protocol claims, and the [Release Plan](RELEASE_PLAN.md) assigns every
incomplete area to a concrete later release.

Legend:

- 🟢 Available: usable for the stated scope.
- 🟡 Partial: meaningful implementation exists, but the complete Ethereum
  validity or execution contract is not yet available.
- 🔴 Planned: no production implementation is claimed.

## Protocol And Wire Support

| Capability | Status | Current scope |
| --- | --- | --- |
| Primitive domains | 🟢 Available | Chain ID, block number, gas, nonce, timestamp, address, hash, Wei, and transaction-type newtypes |
| Canonical RLP | 🟢 Available | Bounded scalar, list, integer, exact-consumption, encoding, primitive bridges, conservative derives, and operation-wide shared decode sessions |
| EIP-2718 envelopes | 🟢 Available | Legacy and typed outer-envelope classification |
| Legacy transactions | 🟡 Partial | Canonical field decode/encode, EIP-155 replay checks, signing hashes, and signature validation |
| EIP-2930 | 🟡 Partial | Access-list decode/encode, signing hashes, replay checks, and signature validation |
| EIP-1559 | 🟡 Partial | Dynamic-fee decode/encode, signing hashes, replay checks, and signature validation |
| EIP-4844 | 🟡 Partial | Blob-transaction decode/encode and signing support; KZG, blob-count, fee, and full fork/state validity remain incomplete |
| EIP-7702 | 🟡 Partial | Set-code decode/encode, transaction and authorization signing/recovery, plus the non-cryptographic context validity gate |
| EIP-712 | 🟢 Available | Bounded borrowed typed-data encoding and hashing, domain checks, recovery helper, and optional JSON boundary |
| Chain and fork context | 🟢 Available | Caller-provided chain/fork specifications with monotonic activation and chain-binding checks |
| Block headers | 🟡 Partial | Legacy through Prague syntactic field decode and canonical header hashing; full header validity is not implemented |
| Receipts | 🟡 Partial | Legacy and typed receipt decode with bloom, logs, topics, status/root shape, and bounded borrowed data |
| EIP-4895 withdrawals | 🟡 Partial | Canonical withdrawal-list decode; consensus dequeue, root matching, and state application remain incomplete |

## Proof And Cryptography Support

| Capability | Status | Current scope |
| --- | --- | --- |
| Keccak-256 | 🟢 Available | First-party trait boundary plus optional reviewed `tiny-keccak` backend |
| secp256k1 recovery | 🟢 Available | First-party validation boundary plus optional reviewed `k256` adapter |
| Transaction signing | 🟢 Available | Signing preimages and hashes for legacy, EIP-2930, EIP-1559, EIP-4844, and EIP-7702 |
| MPT node decoding | 🟢 Available | Bounded branch, extension, leaf, compact-path, inline-reference, and proof-list parsing with optional shared-session accounting |
| MPT inclusion proofs | 🟢 Available | Transaction, receipt, account, and storage inclusion against caller-trusted roots |
| Secret sanitization | 🟢 Optional | Explicit opt-in bridge to the separately published `sanitization` crate |

## EVM Support

| Capability | Status | Current scope |
| --- | --- | --- |
| EVM domains | 🟢 Available | Dependency-free word, stack, memory, gas, fork, opcode, program-counter, access-set, and host-state types |
| Native interpreter | 🟡 Partial | Bounded basic stack, arithmetic, control-flow, memory, selected state-read execution, and consensus-correct truncated PUSH zero-padding |
| Historical fork rules | 🟡 Partial | Explicit fork identifiers and admitted gas/opcode boundaries; full historical execution remains versioned |
| Call and create | 🟡 Partial | Stack/memory/static/depth planning and journal policy; nested host execution and commits remain fail closed |
| Identity, SHA-256, RIPEMD-160 | 🟢 Available | First-party dependency-free execution through charged plans |
| ECRECOVER | 🟢 Boundary | Charged execution through caller-provided secp256k1 and Keccak backends |
| ModExp | 🟢 Available | Bounded first-party EIP-198/EIP-2565 execution |
| BN254 | 🟢 Available | Add, multiplication, subgroup checks, Miller loop, final exponentiation, and pairing result admission |
| BLAKE2F | 🟢 Available | Exact EIP-152 frame validation and execution |
| BLS12-381 | 🟡 Partial | Exact gas/frame planning and canonical Fp, Fr, Fp2, scalar, G1/G2 wire parsing; curve arithmetic and precompile execution remain fail closed |
| KZG point evaluation | 🟡 Partial | Exact frame, output, and gas planning; trusted setup and cryptographic verification remain fail closed |
| Full state transition | 🔴 Planned | Transaction execution, state commits, logs, refunds, roots, and complete fork conformance remain assigned later releases |

## Product Boundaries

| Capability | Status | Current scope |
| --- | --- | --- |
| Owned SDK models and interoperability | 🔴 Planned | Assigned to `v0.53.0..=v0.68.0` |
| Complete execution and tracing | 🔴 Planned | Assigned to `v0.69.0..=v0.91.0` |
| Typed providers and transaction lifecycle | 🔴 Planned | Assigned to `v0.92.0..=v0.108.0` |
| Signers, wallets, and account abstraction | 🔴 Planned | Assigned to `v0.109.0..=v0.119.0` |
| ABI, contracts, and application standards | 🔴 Planned | Assigned to `v0.120.0..=v0.129.0` |
| Storage, canonical chain, and runtime | 🔴 Planned | Assigned to `v0.130.0..=v0.140.0` |
| Consensus primitives, Engine API, and light client | 🔴 Planned | Assigned to `v0.141.0..=v0.153.0` |
| DevP2P, RLPx, txpool, and sync | 🔴 Planned | Assigned to `v0.154.0..=v0.164.0` |
| Witnesses, stateless execution, and commitment evolution | 🔴 Planned | Assigned to `v0.165.0..=v0.174.0` |
| Foundation assurance and compatibility | 🔴 Planned | Assigned to `v0.175.0..=v0.188.0` |
| Full beacon node, consensus networking, sync, deposits, and production APIs | 🔴 Planned | Assigned to `v0.189.0..=v0.234.0` |
| Slashing protection, validator keys, duties, and external custody | 🔴 Planned | Assigned to `v0.235.0..=v0.248.0` |
| Builder and relay integration | 🔴 Planned | Assigned to `v0.249.0..=v0.250.0`; relay communication is beacon-node owned |
| Consensus safety services, operations, and production executables | 🔴 Planned | Assigned to `v0.251.0..=v0.257.0` |
| Consensus assurance and product baseline | 🔴 Planned | Assigned to `v0.258.0..=v0.274.0`; this is not the final 1.0 gate |
| First-party core cryptography and historical PoW | 🔴 Planned | Keccak-256, secp256k1, ECDSA/ECDH, and transport/keystore primitives are assigned to `v0.52.22..=v0.52.26`; full-stack crypto revalidation is `v0.275.0..=v0.278.0`, followed by Ethash and genesis-to-Merge validation at `v0.279.0..=v0.281.0` |
| Standalone production execution client | 🔴 Planned | Database backend, staged sync, payload builder, Engine/RPC servers, networking operations, binary, tooling, and controls are assigned to `v0.282.0..=v0.291.0` |
| Execution-client production assurance | 🔴 Planned | Hive/RPC compatibility, independent consensus-client interoperability, public sync, performance, audit, and remediation are assigned to `v0.292.0..=v0.297.0` |
| Integrated Ethereum node product | 🔴 Planned | Full-node orchestration, devnets, mixed-client tests, long-running operation, recovery, guides, audit, and remediation are assigned to `v0.298.0..=v0.305.0` |
| Final 1.0 production admission | 🔴 Planned | Acceptance, complete API/crate freeze, release rehearsal, promotion, and candidate admission are assigned to `v0.306.0..=v0.310.0` plus exact candidate `v1.0.0-rc.N` |
| Reth and ecosystem integration | 🔴 Planned | Optional conversion/reference adapters are assigned to `v0.67.0`; no external implementation becomes the first-party core |

## Security And Release Baseline

| Area | Current policy |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Pinned stable | Rust `1.97.1` |
| Default target | `no_std` |
| Unsafe code | Forbidden in first-party crates |
| Default networking/signing | None |
| Runtime dependency policy | Reference and optional backend crates are excluded from the default facade graph |
| Release evidence | Formatting, strict clippy, tests, doctests, package verification, fuzz compilation, cargo-deny, cargo-audit, SBOM, pentest, and retest |
| Formal verification | Kani is planned as additional evidence, not a replacement for testing or review |

## Current Release

`v0.52.3` adds a non-copyable `DecodeSession` and reviewed policy for one
complete untrusted operation. Session-aware RLP, legacy and typed transaction,
nested access-list/blob-hash/authorization, and MPT syntax entry points share
cumulative counters instead of resetting local budgets between stages.

Structural RLP validation charges each encoded byte once. Later zero-copy
semantic reparses have public session-aware traversal and are charged as actual
additional work. Nested-list metadata recounts debit each immediate child
before parsing it. The ledger covers
bytes, headers, items, nesting, requested allocation capacity, proof nodes,
hashes, hash bytes, and aggregate work with checked arithmetic. Current
session-aware transaction and MPT models are borrowed and allocate nothing.

Complete MPT proof preflight and reject-before-hash behavior remain assigned
to `v0.52.4`; this release does not broaden proof-validity claims.

The current workspace uses Rust `1.97.1` for the full gate and checks every
supported Rust toolchain from `1.90.0` through `1.97.0` with
`cargo check --workspace --all-features`.
