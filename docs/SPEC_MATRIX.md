# eth Specification Matrix

Status: source revisions pinned for `v0.37.4`; scalar, list, and canonical
integer RLP decoding, canonical RLP encoding helpers, primitive RLP bridging,
Keccak-256 trait boundary, RLP fuzz harness baseline, external execution
fixture coverage, dev-only independent RLP differential coverage, and
transaction envelope shell plus unvalidated legacy, EIP-2930 access-list,
EIP-1559 dynamic-fee, EIP-4844 blob, and EIP-7702 set-code transaction decoding
and canonical encoding implemented. Explicit chain and fork activation context
is available for caller-reviewed specs.
Transaction typestate promotion is proof-gated. Transaction signing-hash
construction, digest-level sender recovery, and decoded transaction signature
validation are available behind the caller-provided Keccak-256 boundary,
including EIP-7702 transaction and authorization signature validation.
EIP-7702 set-code transaction context validity is available through a
caller-provided authority and account-state boundary.
EIP-712 domain-safety helpers require `chainId` and `verifyingContract` before
structured-data sender recovery is trusted, and the borrowed typed-data encoder
can compute `encodeType`, `encodeData`, `hashStruct`, domain separators, and
the final `0x1901` digest. The optional v0.26.1 JSON boundary parses JSON-RPC
typed-data payloads into the same hashing path when explicitly enabled.
`v0.27.0` admits an optional non-default `tiny-keccak` backend with
Keccak-256 KAT coverage. `v0.28.0` adds syntactic execution block header
decoding and hashing. `v0.29.0` adds syntactic legacy and typed receipt
decoding with borrowed logs and explicit status/root handling. `v0.30.0` adds
syntactic EIP-4895 withdrawal-list decoding with borrowed entries. `v0.31.0`
adds bounded syntactic MPT branch, extension, and leaf node decoding. `v0.32.0`
adds transaction and receipt inclusion proof verification against trusted
roots. `v0.33.0` adds account and storage inclusion proof verification against
trusted roots. `v0.34.0` refreshes all official Ethereum source and fixture
pins and documents the reproducible external reference store. `v0.35.0` adds
the first external execution fixture harness and claims pinned `ethereum/tests`
`RLPTests` for the codec surface. `v0.36.0` adds a dev-only `alloy-rlp`
structural RLP differential reference path. `v0.37.0` reviews REVM for the
EVM adapter boundary and does not admit it until dependency policy passes.
`v0.37.1` adds the safe upstream advisory checker for REVM registry metadata
and official Ethereum source drift. `v0.37.2` adds the core dependency
independence audit and schedules every remaining third-party core dependency
classification follow-up before execution work continues. `v0.37.3` moves
secp256k1 sender recovery behind a first-party backend boundary and keeps the
reviewed `k256` adapter behind an explicit feature. `v0.38.0` adds the
explicit EVM execution environment, transaction input, state snapshot, and
result report boundary. `v0.39.0` adds the bounded gas-estimation boundary,
`v0.40.0` adds the dependency-free native EVM core type foundation, and
`v0.41.0` adds the first bounded native execution pass and `v0.42.0` adds
gas-metering for basic stack/control-flow bytecode, without admitting a
concrete production execution backend.

Official source and fixture revisions are governed by
[Spec Source Policy](spec-source-policy.md). `v0.34.0` checked upstream `HEAD`
on 2026-07-04 and pinned execution-specs, execution tests, EIPs,
execution-apis, and consensus-specs in `spec-lock.toml`. The external
reference-store path and license notes are documented in
[Ethereum Reference Store](reference-store.md). Consensus-sensitive behavior
must not be implemented from memory.
Execution fixture claims are tracked in
[Execution Fixture Harness](execution-fixture-harness.md),
[Execution Fixture Report](execution-fixture-report.md), and
[Unsupported Execution Fixtures](unsupported-execution-fixtures.md).
Differential claims are tracked in
[Differential Test Harness](differential-test-harness.md) and
[Differential Test Report](differential-test-report.md).
Upstream drift is tracked with
[Ethereum Upstream Check](ethereum-upstream-check.md).

| Area | Status | Evidence |
| --- | --- | --- |
| Execution RLP | claimed for pinned `RLPTests` plus differential structural RLP | `ethereum/tests` pinned in `spec-lock.toml`; scalar byte-string, list, canonical integer decoders, canonical encoding helpers, and public conservative RLP derives implemented; v0.35.0 runs the pinned `RLPTests` corpus through `eth-valkyoth-codec::execution_rlp_fixtures`; v0.36.0 compares curated structural RLP cases against `alloy-rlp` |
| RLP derives | public conservative surface | v0.25.0 adds public `RlpEncode`/`RlpDecode` traits and derive macros for reviewed structs; generated decoders require `DecodeLimits`; generics, enums, unions, and transaction derives remain rejected |
| RLP fuzz harness | baseline | `fuzz/` workspace builds; committed hex seeds live under `fuzz/seed-corpus/`; crash reproduction is documented |
| Keccak-256 hashing | boundary plus optional backend | `eth-valkyoth-hash` defines a caller-provided Keccak-256 trait boundary; v0.27.0 admits `TinyKeccak256` behind the non-default `tiny-keccak` support-crate feature and `keccak-tiny` facade feature, with empty-input, `abc`, and chunking KAT coverage |
| EIP-712 structured data | typed-data encoder plus optional JSON boundary | EIP-712 defines the `0x1901` signing digest, `encodeType`, `encodeData`, `hashStruct`, and optional domain fields; v0.21.0 checks required caller-provided `chainId` and `verifyingContract` fields and builds the signing digest from supplied domain/message hashes; v0.26.0 adds a no-alloc borrowed typed-data encoder for admitted atomic, dynamic, array, and struct fields plus domain separator construction and the official Ether Mail recovery KAT; v0.26.1 adds an opt-in `std` JSON-RPC typed-data parser boundary with duplicate-key rejection and explicit limits; v0.52.0 bounds both paths to 64 types, visits each reachable dependency once, and redacts non-copyable signing values; v0.52.1 rejects malformed, duplicate, or atomic-looking borrowed schema/value identifiers, caps borrowed structs at 64 fields and values, borrowed arrays at 256 elements per dimension, and complete borrowed/JSON operations at 4,096 recursive value visits, validates schemas once per public operation, caches type hashes during recursive hashing, applies identifier checks to JSON schemas, and clears partial encode-data output on failure. |
| EIP-2718 typed transactions | partial | `ethereum/EIPs` pinned in `spec-lock.toml`; envelope classification implemented; EIP-2930 type `0x01`, EIP-1559 type `0x02`, EIP-4844 type `0x03`, and EIP-7702 set-code type `0x04` field decode and canonical encode implemented; later typed transaction payloads remain opaque until explicitly admitted |
| Legacy transactions | field decode/encode | EIP-2718 defines the legacy transaction field list; v0.12.0 decodes fields into an unvalidated model and v0.16.0 encodes that admitted model without signature, sender, chain, or fork validation |
| EIP-2930 access-list transactions | field decode/encode | EIP-2930 defines type `0x01`, eleven payload fields, and access-list shape; v0.13.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, gas, duplicate, chain, account-state, or fork validation |
| EIP-1559 dynamic-fee transactions | field decode/encode | EIP-1559 defines type `0x02`, twelve payload fields, and access-list inheritance from EIP-2930; v0.14.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, fee-order, gas, duplicate, chain, account-state, or fork validation |
| EIP-4844 blob transactions | field decode/encode | EIP-4844 defines type `0x03`, fourteen payload fields, required 20-byte `to`, max blob fee, and blob versioned hash list; v0.15.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, blob fee, KZG, data availability, blob-hash version, blob count, chain, account-state, block blob-gas, or fork validation |
| EIP-7702 set-code transactions | validity gate | EIP-7702 defines type `0x04`, thirteen payload fields, required 20-byte destination, authorization tuples shaped `[chain_id, address, nonce, y_parity, r, s]`, transaction signing over `0x04 || payload`, authorization signing over `0x05 || rlp([chain_id, address, nonce])`, non-empty authorization lists, authorization chain binding, nonce policy, and empty-or-delegated authority code. v0.24.0 decodes and encodes the admitted model, v0.24.1 adds transaction signing-hash plus authorization signer recovery, and v0.24.2 adds the non-cryptographic context validity gate with EIP-7702 per-tuple skip accounting. |
| Chain and fork specs | explicit context | `execution-specs` and EIPs are pinned in `spec-lock.toml`; v0.17.0 adds caller-provided `ChainSpec`, `ForkSpec`, hardfork identity, block/timestamp activation checks, unsupported-fork errors, chain-mismatch errors, duplicate-fork errors, and non-monotonic fork/activation ordering errors without hardcoding mainnet validation rules |
| Transaction validation | partial | `execution-specs` pinned in `spec-lock.toml`; v0.18.0 adds proof-gated decoded/canonical/fork-valid/sender-recovered transaction state transitions, v0.19.0 adds replay-domain checks, v0.20.0 adds digest-level sender recovery with low-s and y-parity policy, v0.22.0 adds transaction signing-hash construction for legacy EIP-155, EIP-2930, EIP-1559, and EIP-4844, v0.23.0 adds decoded transaction signature validation helpers, v0.24.1 adds EIP-7702 set-code transaction and authorization signature validation, v0.24.2 adds the EIP-7702 set-code context validity gate, and v0.37.3 moves sender recovery behind `RecoverableSecp256k1` plus `_with_backend` validation APIs. Remaining concrete proof constructors remain planned. |
| Header decoding and hashing | syntactic decode/hash | `execution-specs` pinned in `spec-lock.toml`; v0.28.0 decodes legacy, London, Shanghai, Cancun, and Prague header field sets and hashes canonical header RLP through the Keccak trait boundary without claiming full header validity |
| Receipt decoding | syntactic decode | EIP-658 and EIP-2718 checked for status/root and typed receipt envelopes; v0.29.0 decodes legacy and typed receipts, validates bloom/log/topic shape, and does not itself claim receipt-trie or block-root validity |
| Withdrawal decoding | syntactic decode | EIP-4895 checked for withdrawal list and entry shape; v0.30.0 decodes canonical withdrawal lists with `uint64` indexes, 20-byte recipient addresses, and nonzero Gwei amounts, and does not claim consensus-layer dequeue correctness, header `withdrawals_root` matching, or state-balance application |
| Core dependency independence | audited, third remediation complete | `v0.37.2` audits every dependency that touches core Ethereum behavior, classifying `k256` as default temporary debt, `subtle` as a reviewed-exception candidate, `tiny-keccak`, `serde`, `serde_json`, and `sanitization` as optional paths, and `alloy-rlp` plus `sha3` as dev/reference paths. `v0.37.3` removes default `k256`, removes direct verify-test `sha3`, adds the `RecoverableSecp256k1` boundary, and keeps the reviewed adapter under `secp256k1-k256`; `v0.37.4` retains `subtle` as a narrow reviewed exception and gates reference-only dependency quarantine; `v0.37.5` documents and gates optional parser/sanitization feature paths. |
| EVM execution boundary | explicit boundary | `v0.38.0` adds `ExecutionEnvironment`, `ExecutionTransaction`, `StateSnapshot`, `ExecutionRequest`, and `ExecutionReport` so future execution attempts must bind active fork context, block context, decoded transaction shell, caller-computed transaction hash, and snapshot identity. `v0.39.0` adds bounded gas-estimation policy, request, report, status, deterministic errors, and hard release ceilings so future estimators must carry maximum attempts, a gas cap, and a termination guard that cannot be practically infinite. No backend is admitted yet. |
| Native EVM execution | basic opcode subset plus BLS wire parsing | `v0.40.0` through `v0.51.0` build the dependency-free bounded interpreter, historical gas/state boundaries, call/create planning, and native precompiles through EIP-152 BLAKE2F, including complete BN254 pairing. `v0.52.0` adds exact EIP-4844 KZG and EIP-2537 BLS12-381 frame, output, fixed-gas, MSM-discount, and pairing-gas planning. `v0.52.1` adds canonical dependency-free EIP-2537 Fp, Fr, Fp2, unrestricted MSM scalar, G1/G2 coordinate, infinity, and complete `0x0b..=0x11` frame parsing. `v0.52.2..=v0.52.10` close consensus-correctness, shared-work, proof-composition, execution-admission, access-tracking, precompile-contract, and ModExp gaps before advanced BLS execution. Parsed point coordinates do not yet prove curve or subgroup membership; first-party BLS arithmetic and execution remain assigned through `v0.52.11..=v0.52.18`, followed by architecture, resource-governor, and cryptographic-contract gates through `v0.52.21`. Complete EVM/state-transition behavior, KZG/blob integration, EOF, current-fork admission, fixtures, tracing, and simulation are assigned through `v0.69.0..=v0.91.0`; first-party Keccak/secp completion and historical PoW consensus are separately assigned to `v0.275.0..=v0.281.0`. REVM remains reference/compatibility only if admitted. |
| Owned SDK and shared domains | scheduled | General integers, bytes/hash domains, owned transaction/block/state models, conversions, bounded allocation helpers, payload-bound typestates, fork-rule redesign, and shared protocol/EVM domains are assigned to `v0.53.0..=v0.68.0`. |
| Header validation | scheduled | `execution-specs` pinned in `spec-lock.toml`; `v0.73.0` schedules parent, gas, base-fee, blob-gas, fork-activation, difficulty/TTD, optional field, and block-root validation for claimed forks. |
| Receipt and withdrawal validation | scheduled | `execution-specs` pinned in `spec-lock.toml`; `v0.75.0` schedules receipt construction, receipt trie/root matching, cumulative gas, bloom validation, withdrawal root validation, and withdrawal state application. |
| MPT node decoding | syntactic decode | `execution-specs` pinned in `spec-lock.toml`; v0.31.0 decodes branch, extension, and leaf node shape with compact-path and child-reference checks plus cumulative proof-node count/byte accounting |
| MPT proofs | transaction/receipt/account/storage inclusion | `execution-specs` pinned in `spec-lock.toml`; v0.32.0 verifies transaction and receipt inclusion at `rlp(transaction_index)` against trusted root newtypes through the Keccak trait boundary, and v0.33.0 verifies account and storage inclusion at `keccak256(address)` and `keccak256(slot_key)` with distinct root/key domains. The APIs distinguish malformed, absent, and wrong-root/value-mismatch proofs without claiming header-root, account-state, or storage-root composition validity. |
| Trie construction and roots | scheduled | `v0.76.0` schedules first-party transaction, receipt, account, and storage trie root builders so root values can be computed, not only verified from supplied proofs. |
| Blob/KZG validation | scheduled | `v0.77.0..=v0.81.0` schedule first-party trusted setup, KZG arithmetic, commitments/proofs, point-evaluation execution, blob-hash/count/fee/sidecar validation, fork integration, and official fixtures. |
| EOF and current forks | scheduled | `v0.82.0..=v0.86.0` schedule EOF validation/execution/deployment and source-driven current-fork manifest and execution changes. |
| Full execution fixtures and tooling | scheduled | `v0.69.0`, `v0.87.0..=v0.91.0` schedule official state tests, complete execution fixture admission, differential/performance gates, inspectors, traces, state diffs, and deterministic simulation. |
| JSON-RPC and providers | scheduled | `execution-apis` pinned in `spec-lock.toml`; outbound typed methods, HTTP/WS/IPC/EIP-1193 transports, batching, subscriptions, middleware, quorum/proof trust, and full transaction lifecycle are assigned to `v0.92.0..=v0.108.0`; inbound execution JSON-RPC, filters, subscriptions, GraphQL, and operational namespaces are assigned to `v0.286.0..=v0.287.0`. |
| Wallets and account abstraction | scheduled | Signer interfaces, local/remote/hardware custody, keystores, BIP-39/BIP-32/BIP-44, ERC-1271, Safe, ERC-4337, paymasters, session keys, and delegated accounts are assigned to `v0.109.0..=v0.119.0`. |
| ABI encoding and contract tooling | scheduled | ABI types/codec, artifacts, code generation, deployment/linking, events/errors, multicall, standards, ENS, permits, and hardening are assigned to `v0.120.0..=v0.129.0`. |
| Storage and canonical chain | scheduled | Database contracts, chain/state schemas, crash consistency, migrations, pruning/archive/history expiry, canonical import/reorg, heads, payload invalidation, runtime, and foundation performance are assigned to `v0.130.0..=v0.140.0`; production database admission, staged pipeline wiring, and operator tools are assigned to `v0.282.0..=v0.283.0` and `v0.290.0`. |
| Engine API | scheduled | Complete Engine types and validation are assigned to `v0.143.0`; runtime-neutral client/server transport contracts are assigned to `v0.144.0`; local payload construction and the authenticated execution-client Engine server are assigned to `v0.284.0..=v0.285.0`. |
| SSZ beacon consensus and light client | scheduled | Foundational SSZ, beacon types, Engine/Beacon API boundaries, weak-subjectivity bootstrap, BLS committees, rotation/persistence, finality scoring, execution proof binding, recovery, complete light-client vectors, and the PeerDAS threat/admission plan are assigned to `v0.141.0..=v0.153.0`; the complete mutable SSZ/BLS client surfaces and PeerDAS cryptographic core are assigned to `v0.191.0..=v0.193.0`. |
| DevP2P RLPx and discovery | scheduled | First-party Ethereum protocol ownership, threat/dependency admission, discovery/RLPx, `eth`/`snap` messages, peer management, request scheduling, txpool, sync, Portal/history acquisition, and hardening are assigned to `v0.154.0..=v0.164.0`; DNS discovery, boot/static/trusted peers, NAT, serving ranges, and public-network operational wiring are assigned to `v0.288.0`. |
| Witnesses and stateless execution | scheduled | Proof abstraction, execution witnesses, MPT/stateless execution, successor commitment support, state evolution, ZK proof boundaries, and fork automation are assigned to `v0.165.0..=v0.174.0`. |
| Foundation assurance | scheduled | Platform/performance evidence, Kani, Miri/sanitizers, compatibility gates, task documentation, foundation audits, remediation, and integration evidence are assigned to `v0.175.0..=v0.188.0`. |
| Full beacon state transition | scheduled | Consensus architecture/configuration, complete SSZ/BLS surfaces, PeerDAS cryptographic core, committees/domains, per-slot/per-epoch processing, validator lifecycle, rewards/penalties, operations, execution requests, data availability, fork upgrades, and official vectors are assigned to `v0.189.0..=v0.204.0`. |
| Consensus fork choice and beacon storage | scheduled | Transactional LMD-GHOST/Casper FFG, proposer boost/reorg policy, optimistic invalidation, persistence, operation pools, hot/finalized stores, state reconstruction, custody retention, migration, and repair are assigned to `v0.205.0..=v0.214.0`. |
| Consensus networking and sync | scheduled | Separate consensus discv5/libp2p/GossipSub/ReqResp networking, first-party Ethereum codecs/validation/state machines, scoring, backpressure, checkpoint/weak-subjectivity sync, head/range sync, backfill, optimistic recovery, and PeerDAS custody sync are assigned to `v0.215.0..=v0.225.0`; generic transport/runtime adapters cannot own consensus validity. |
| Beacon node Engine DA production and APIs | scheduled | Authenticated multi-execution-client coordination, historical deposit tracking, genesis construction, availability admission, beacon-node orchestration, Beacon REST/Event APIs, beacon-owned unsigned block production, and Validator API are assigned to `v0.226.0..=v0.234.0`. |
| Validator client slashing keys and custody | scheduled | Slashing invariants, transactional records, EIP-3076, EIP-2333/EIP-2334 key foundations, withdrawal-key separation, local signing, duty safety, proposals, attestations, sync committees, lifecycle operations, Keymanager, remote signing, HSM/KMS/hardware custody, and threshold/DVT coordination are assigned to `v0.235.0..=v0.248.0`. |
| Builder and relay integration | scheduled | Beacon-node-owned Builder API and relay communication, validator preference submission and independent blinded-block validation/signing, relay multiplexing, withholding defenses, and local fallback are assigned to `v0.249.0..=v0.250.0`. |
| Consensus safety operations and executables | scheduled | Optional slasher, NAT/connectivity diagnostics, validator analytics, beacon-node and validator-client binaries/packaging, database tools, and deterministic simulation are assigned to `v0.251.0..=v0.257.0`. |
| Consensus production assurance and baseline | scheduled | Quantitative attribution and resource gates, mandatory consensus Hive/client matrices, long validator testnet, performance, Kani proofs, SSZ/BLS/PeerDAS and consensus integration audits, remediation, stability baseline, and RC-aware tooling foundations are assigned to `v0.258.0..=v0.274.0`. |
| First-party core execution cryptography | scheduled | Dependency-free Keccak-256, secp256k1 arithmetic, ECDSA signing/verification/recovery, ECDH, production-path integration, and cryptography audit are assigned to `v0.275.0..=v0.278.0`. |
| Historical proof-of-work execution | scheduled | Ethash seal verification, historical difficulty/ommers/rewards/irregular transitions, terminal-total-difficulty handling, and genesis-to-Merge conformance are assigned to `v0.279.0..=v0.281.0`. |
| Standalone execution-client product | scheduled | Production database backend, staged sync/unwind/healing, local payload builder, authenticated Engine server, public JSON-RPC/GraphQL, operational DevP2P discovery, execution-node binary, recovery tools, and resource controls are assigned to `v0.282.0..=v0.291.0`. |
| Complete execution-client assurance | scheduled | Execution Hive and RPC compatibility, independent consensus-client Engine interoperability, public-network sync/follow evidence, performance, independent audit, and remediation are assigned to `v0.292.0..=v0.297.0`. |
| Integrated Ethereum node | scheduled | Execution/beacon orchestration, reproducible devnets, independently replaceable client roles, long-running integrated operation, combined performance/recovery, operator guides, full-stack audit, and remediation are assigned to `v0.298.0..=v0.305.0`. |
| Final production assurance and admission | scheduled | Final quantitative acceptance, complete public API/crate freeze, exact release evidence, version-only promotion, archive publication, and candidate admission are assigned to `v0.306.0..=v0.310.0` plus `v1.0.0-rc.N`. |

Every release that claims support for a fork, EIP, RPC method, or wire protocol
must update this matrix and `spec-lock.toml`.
