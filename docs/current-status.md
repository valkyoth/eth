# Current Status

Release snapshot: `v0.52.1` implementation complete; independent pentest and
retest still required before tagging.

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
| Canonical RLP | 🟢 Available | Bounded scalar, list, integer, exact-consumption, encoding, primitive bridges, and conservative derives |
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
| MPT node decoding | 🟢 Available | Bounded branch, extension, leaf, compact-path, inline-reference, and proof-list parsing |
| MPT inclusion proofs | 🟢 Available | Transaction, receipt, account, and storage inclusion against caller-trusted roots |
| Secret sanitization | 🟢 Optional | Explicit opt-in bridge to the separately published `sanitization` crate |

## EVM Support

| Capability | Status | Current scope |
| --- | --- | --- |
| EVM domains | 🟢 Available | Dependency-free word, stack, memory, gas, fork, opcode, program-counter, access-set, and host-state types |
| Native interpreter | 🟡 Partial | Bounded basic stack, arithmetic, control-flow, memory, and selected state-read execution |
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
| JSON-RPC transport | 🔴 Planned | Trust-model types exist, but no production transport/client is implemented |
| Signer and key storage | 🔴 Planned | No local key store or signing service is enabled |
| ABI and contract helpers | 🔴 Planned | Assigned to `v0.70.0..=v0.78.0` |
| Consensus and Engine API | 🔴 Planned | Assigned to `v0.79.0..=v0.85.0` |
| DevP2P, RLPx, discovery, and sync | 🔴 Planned | Assigned to `v0.86.0..=v0.92.0` |
| Reth integration | 🔴 Planned | Boundary crate only; no runtime integration is admitted |

## Security And Release Baseline

| Area | Current policy |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Pinned stable | Rust `1.97.0` |
| Default target | `no_std` |
| Unsafe code | Forbidden in first-party crates |
| Default networking/signing | None |
| Runtime dependency policy | Reference and optional backend crates are excluded from the default facade graph |
| Release evidence | Formatting, strict clippy, tests, doctests, package verification, fuzz compilation, cargo-deny, cargo-audit, SBOM, pentest, and retest |
| Formal verification | Kani is planned as additional evidence, not a replacement for testing or review |

## Current Release

`v0.52.1` adds canonical dependency-free EIP-2537 BLS12-381 field,
full-width multiplication scalar, point-coordinate, infinity, and all
`0x0b..=0x11` frame parsers. Point values are wire-valid only: curve membership,
subgroup validation, mapping, arithmetic, MSM, pairing, and precompile execution
remain assigned to `v0.52.2..=v0.52.9`.

The release also zero-initializes EVM memory, makes execution contexts one-shot
until destructive reset, restores warm/cold tracking after failed stateful
runs, and hardens EIP-712 identifier, uniqueness, and partial-output handling.

The release uses Rust `1.97.0` for the full gate and checks every supported Rust
toolchain from `1.90.0` through `1.96.1` with
`cargo check --workspace --all-features`.
