# Current Status

Release snapshot: `v0.55.0` release candidate; pentest and retest passed;
awaiting green GitHub CI and CodeQL.

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
| Canonical RLP | 🟢 Available | Bounded scalar, list, integer, exact-consumption, encoding, primitive bridges, conservative derives, and operation-wide shared decode sessions with trie-work ceilings |
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
| MPT node decoding | 🟢 Available | Strict locally canonical branch, extension, leaf, compact-path, inline-reference, and proof-list parsing with shared-session accounting |
| MPT inclusion proofs | 🟢 Available | Full-proof preflight, hash-addressed snapshot-bound multiproofs, transaction/receipt inclusion, canonical account decoding, account-bound storage verification, and canonical absence/zero semantics |
| Secret sanitization | 🟢 Optional | Explicit opt-in bridge to `sanitization 2.0.3`, canonical wiping, drop-safety contracts, and runtime protection reports |

## EVM Support

| Capability | Status | Current scope |
| --- | --- | --- |
| EVM domains | 🟢 Available | Dependency-free word, stack, memory, gas, fork, opcode, program-counter, injectable access-tracker, and host-state types |
| Execution admission | 🟢 Available | Non-forgeable classified, canonical, fork-bound, and execution-ready transaction typestates; unknown and empty typed envelopes fail closed |
| Host capabilities | 🟢 Available | Separate state-view, journal, block, access, crypto, inspector, and resettable bounded-arena contracts; embedded-linear and optional fixed-width-radix node trackers; explicit transaction resource governor |
| Native interpreter | 🟡 Partial | Bounded basic stack, arithmetic, control-flow, memory, selected state-read execution, and consensus-correct truncated PUSH zero-padding |
| Historical fork rules | 🟡 Partial | Explicit fork identifiers and admitted gas/opcode boundaries; full historical execution remains versioned |
| Call and create | 🟡 Partial | Stack/memory/static/depth planning and journal policy; nested host execution and commits remain fail closed |
| Identity, SHA-256, RIPEMD-160 | 🟢 Available | First-party dependency-free execution through exact-input quotes and one-shot paid capabilities |
| ECRECOVER | 🟢 Boundary | Paid execution through caller-provided secp256k1 and Keccak backends |
| ModExp | 🟢 Available | First-party EIP-198/EIP-2565 execution through Prague with 256-bit length admission, virtual padding, and caller-owned gas-bounded workspace; Osaka changes are assigned to `v0.116.0` |
| BN254 | 🟢 Available | Add, multiplication, subgroup checks, Miller loop, final exponentiation, and pairing result admission |
| BLAKE2F | 🟢 Available | Exact EIP-152 frame validation and execution |
| BLS12-381 | 🟡 Partial | Exact gas/frame planning and canonical Fp, Fr, Fp2, scalar, G1/G2 wire parsing; curve arithmetic and precompile execution remain fail closed |
| KZG point evaluation | 🟡 Partial | Exact frame, output, and gas planning; trusted setup and cryptographic verification remain fail closed |
| Full state transition | 🔴 Planned | Transaction execution, state commits, logs, refunds, roots, and complete fork conformance remain assigned later releases |

## Product Boundaries

| Capability | Status | Current scope |
| --- | --- | --- |
| Owned SDK models and interoperability | 🔴 Planned | Assigned to `v0.83.0..=v0.98.0` |
| Complete execution and tracing | 🔴 Planned | Assigned to `v0.99.0..=v0.121.0` |
| Typed providers and transaction lifecycle | 🔴 Planned | Assigned to `v0.122.0..=v0.138.0` |
| Signers, wallets, and account abstraction | 🔴 Planned | Assigned to `v0.139.0..=v0.149.0` |
| ABI, contracts, and application standards | 🔴 Planned | Assigned to `v0.150.0..=v0.159.0` |
| Storage, canonical chain, and runtime | 🔴 Planned | Assigned to `v0.160.0..=v0.170.0` |
| Consensus primitives, Engine API, and light client | 🔴 Planned | Assigned to `v0.171.0..=v0.183.0` |
| DevP2P, RLPx, txpool, and sync | 🔴 Planned | Assigned to `v0.184.0..=v0.194.0` |
| Witnesses, stateless execution, and commitment evolution | 🔴 Planned | Assigned to `v0.195.0..=v0.204.0` |
| Foundation assurance and compatibility | 🔴 Planned | Assigned to `v0.205.0..=v0.218.0` |
| Full beacon node, consensus networking, sync, deposits, and production APIs | 🔴 Planned | Assigned to `v0.219.0..=v0.264.0` |
| Slashing protection, validator keys, duties, and external custody | 🔴 Planned | Assigned to `v0.265.0..=v0.278.0` |
| Builder and relay integration | 🔴 Planned | Assigned to `v0.279.0..=v0.280.0`; relay communication is beacon-node owned |
| Consensus safety services, operations, and production executables | 🔴 Planned | Assigned to `v0.281.0..=v0.287.0` |
| Consensus assurance and product baseline | 🔴 Planned | Assigned to `v0.288.0..=v0.304.0`; this is not the final 1.0 gate |
| First-party core cryptography and historical PoW | 🔴 Planned | Keccak-256, secp256k1, ECDSA/ECDH, and transport/keystore primitives are assigned to `v0.67.0..=v0.71.0`; full-stack crypto revalidation is `v0.305.0..=v0.308.0`, followed by Ethash and genesis-to-Merge validation at `v0.309.0..=v0.311.0` |
| Standalone production execution client | 🔴 Planned | Database backend, staged sync, payload builder, Engine/RPC servers, networking operations, binary, tooling, and controls are assigned to `v0.312.0..=v0.321.0` |
| Execution-client production assurance | 🔴 Planned | Hive/RPC compatibility, independent consensus-client interoperability, public sync, performance, audit, and remediation are assigned to `v0.322.0..=v0.327.0` |
| Integrated Ethereum node product | 🔴 Planned | Full-node orchestration, devnets, mixed-client tests, long-running operation, recovery, guides, audit, and remediation are assigned to `v0.328.0..=v0.335.0` |
| Final 1.0 production admission | 🔴 Planned | Acceptance, complete API/crate freeze, release rehearsal, promotion, and candidate admission are assigned to `v0.336.0..=v0.340.0` plus exact candidate `v1.0.0-rc.N` |
| Reth and ecosystem integration | 🔴 Planned | Optional conversion/reference adapters are assigned to `v0.97.0`; no external implementation becomes the first-party core |

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

`v0.55.0` removes the private 64-byte ModExp operand ceiling. Length words stay
256-bit through gas calculation, unpayable declarations fail as out of gas,
and payable frames execute with streamed right-padding and caller-owned limb
workspace. Independent BigUint tests plus pinned Geth, Besu, and Nethermind
client comparisons cover adversarial operand shapes through 256 bytes, and
Berlin-priced release benchmarks cover dense, sparse, and zero exponents
through 1,024-byte frames. See
[ModExp Precompile](modexp-precompile.md).

`v0.54.0` removes production execution methods from informational precompile
plans. Native execution now requires a canonical exact-input gas quote,
pre-charge output admission, and a non-forgeable one-shot paid capability.
Immutable borrowing prevents safe-Rust input substitution, altered descriptor
metadata is rejected, and each execution returns a precise CALL-ready outcome.
Post-payment failures consume all gas in the dedicated child-call meter and
explicitly request rollback. Protocol gas replaces the former global calldata
ceiling. See [Precompile Authorization](precompile-authorization.md).

`v0.53.0` replaced the hardwired linear warm-access set with an injectable
tracker contract. The default embedded profile remains allocation-free and
fixed-capacity. The optional node profile pre-reserves bounded compressed-radix
indexes and undo storage, performs no allocation after construction, and
bounds lookup and insertion by fixed Ethereum key width. Rollback touches only
post-checkpoint insertions rather than rebuilding retained outer state.
Capacity failures are atomic, failed/reverted root attempts restore exact
warmth, and transaction reset retains only the constructor-bounded allocation.

The EVM boundary exposes validated per-transaction ceilings for warm access,
journals, checkpoints, frames, memory, reusable arenas, caches, and abstract
work. Its governor requires destructive reset, never refunds failed or
cancelled work, and delegates authority through non-copyable child tokens.

`v0.52.7` replaced shell-level execution admission with a non-forgeable
promotion chain from classified envelope through canonical type-specific
decode and active-fork/chain checks to an execution-ready transaction.
`ExecutionRequest` accepts only that final token. `ExecutionHost` derives state
and environment from the request, keeps journal/access/crypto/arena powers
private, makes its associated journal authoritative for current storage,
scopes nested children to LIFO closures, arms poisoning before checkpoint
creation and transaction reset, guards every direct root mutation, and returns
inspector events only after transitions complete.
This milestone does not claim sender recovery, intrinsic-gas, account-state,
fee, blob/KZG, authorization, or complete consensus validity.

`v0.52.6` added bounded hash-addressed node resolution, immutable snapshot
anchors, multiproof query/output accounting, and optional owned deduplicating
arenas with deterministic cancellable scheduling. Resolver traversal preserves
canonical short-child and empty-branch semantics from ordered proofs. Arena
admission bounds raw input, complete hash work, and retained capacities before
use and avoids infallible reallocation under attacker-controlled pressure.

`v0.52.5` decodes authenticated account values as
`[nonce, balance, storageRoot, codeHash]` and returns `VerifiedAccount`
capabilities with private constructors. Composed storage verification accepts
that capability rather than an independent root, so a valid account proof
cannot authorize storage from another account state. Account absence and
storage-path absence are successful verified outcomes; absent storage maps to
zero, and explicitly stored zero is rejected as noncanonical.

The operation preserves v0.52.4 complete proof preflight and can share one
`DecodeSession` across the account proof and all requested storage proofs. A
pinned Execution APIs Hive fixture verifies a real account-plus-storage
response end to end. The earlier byte-exact independently rooted APIs remain
available only as lower-level compatibility boundaries.

The current workspace uses Rust `1.97.1` for the full gate and checks every
supported Rust toolchain from `1.90.0` through `1.97.0` with
`cargo check --workspace --all-features`.
