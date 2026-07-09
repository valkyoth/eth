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
| EIP-712 structured data | typed-data encoder plus optional JSON boundary | EIP-712 defines the `0x1901` signing digest, `encodeType`, `encodeData`, `hashStruct`, and optional domain fields; v0.21.0 checks required caller-provided `chainId` and `verifyingContract` fields and builds the signing digest from supplied domain/message hashes; v0.26.0 adds a no-alloc borrowed typed-data encoder for admitted atomic, dynamic, array, and struct fields plus domain separator construction and the official Ether Mail recovery KAT; v0.26.1 adds an opt-in `std` JSON-RPC typed-data parser boundary with duplicate-key rejection and explicit limits. |
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
| Native EVM execution | basic opcode subset | `v0.40.0` adds `eth-valkyoth-evm-core` with first-party dependency-free EVM word, stack, borrowed memory, program-counter, opcode, fork, opcode-table skeleton, and deterministic error domains. `v0.41.0` adds bounded no-alloc execution for `STOP`, `ADD`, `MUL`, `SUB`, `LT`, `GT`, `EQ`, `ISZERO`, `AND`, `OR`, `XOR`, `NOT`, `POP`, `PC`, `PUSH1..=PUSH32`, `DUP1..=DUP16`, `SWAP1..=SWAP16`, `JUMP`, `JUMPI`, `JUMPDEST`, `RETURN`, and `REVERT`, with mandatory step limits, an EIP-170-sized bytecode ceiling, and one-time no-alloc jumpdest bitmap validation. `v0.42.0` adds fork-scoped gas scheduling, explicit gas limits, gas-used reports, out-of-gas errors, and memory-expansion gas for the currently executable subset. `v0.43.0` adds bounded state access for the currently claimed fork range while failing closed where historical schedules are not yet implemented. `v0.43.1` adds the full historical fork/opcode matrix, and `v0.43.2` implements pre-Berlin state gas schedules for the current state-read subset so older forks are handled deliberately rather than collapsed into London/Berlin pricing. `v0.44.0` adds call/create planning domains, static/depth/return-data/journal policy, and fail-closed interpreter handling for CALL, CALLCODE, DELEGATECALL, STATICCALL, CREATE, and CREATE2. `v0.45.0` adds a fork-aware precompile registry, bounded precompile plans, dependency-free identity execution, and fail-closed cryptographic precompile descriptors. `v0.46.0` adds dependency-free SHA-256 and RIPEMD-160 execution with known-answer vectors and bounded output behavior. `v0.47.0` adds ECRECOVER execution through caller-provided secp256k1 and Keccak backends, preserving the precompile-specific high-s acceptance required by EIP-2. `v0.48.0` adds bounded first-party ModExp input parsing, EIP-198/EIP-2565 gas, execution, and fuzzing with an explicit operand cap. `v0.49.0` adds dependency-free BN254 add and scalar-multiplication execution with canonical field and point validation. `v0.50.0` adds the BN254 pairing frame boundary with empty-input execution, G2 curve validation, and non-empty fail-closed behavior. `v0.50.1` adds G2 subgroup validation and a precomputed twist coefficient. `v0.50.2` adds the Fp6/Fp12 tower foundation. `v0.50.3` adds validated tuple streaming and atomic gas-meter charging for dispatcher-facing pairing execution. `v0.50.4` adds the line-function foundation and extends dispatcher-facing gas-meter charging to ModExp and BN254 add/mul plan execution. `v0.50.5` adds internal Miller-loop accumulation over validated tuples while keeping non-empty execution fail-closed. `v0.50.6` adds sparse Miller-loop multiplication and gas/CPU benchmark evidence, `v0.50.7` adds bounded final exponentiation, `v0.50.8` adds Frobenius Q1/-Q2 point mapping behind the fail-closed pairing boundary, and `v0.50.9`/`v0.50.10` schedule projective post-loop line-carrier completion plus non-empty result admission. `v0.51.0` and `v0.52.0` remain reserved for BLAKE2F and KZG/BLS backend-admission decisions before state-test claims depend on those precompiles. Nested call/create execution, logs, refunds, full official state-test execution, and broader fixture claims remain scheduled through `v0.54.0` and later. REVM remains reference/compatibility only if admitted. |
| Header validation | scheduled | `execution-specs` pinned in `spec-lock.toml`; `v0.57.0` schedules parent, gas, base-fee, blob-gas, fork-activation, difficulty/TTD, optional field, and block-root validation for claimed forks. |
| Receipt and withdrawal validation | scheduled | `execution-specs` pinned in `spec-lock.toml`; `v0.59.0` schedules receipt construction, receipt trie/root matching, cumulative gas, bloom validation, withdrawal root validation, and withdrawal state application. |
| MPT node decoding | syntactic decode | `execution-specs` pinned in `spec-lock.toml`; v0.31.0 decodes branch, extension, and leaf node shape with compact-path and child-reference checks plus cumulative proof-node count/byte accounting |
| MPT proofs | transaction/receipt/account/storage inclusion | `execution-specs` pinned in `spec-lock.toml`; v0.32.0 verifies transaction and receipt inclusion at `rlp(transaction_index)` against trusted root newtypes through the Keccak trait boundary, and v0.33.0 verifies account and storage inclusion at `keccak256(address)` and `keccak256(slot_key)` with distinct root/key domains. The APIs distinguish malformed, absent, and wrong-root/value-mismatch proofs without claiming header-root, account-state, or storage-root composition validity. |
| Trie construction and roots | scheduled | `v0.60.0` schedules first-party transaction, receipt, account, and storage trie root builders so root values can be computed, not only verified from supplied proofs. |
| Blob/KZG validation | scheduled | `v0.61.0` schedules KZG/backend policy, blob versioned-hash validation, blob count, blob fee accounting, point-evaluation precompile integration, trusted setup handling, and blob fixture coverage. |
| Full execution fixtures | scheduled | `v0.62.0` schedules `TransactionTests`, `BlockchainTests`, `GenesisTests`, `TrieTests`, `DifficultyTests`, EOF tests, and state tests for claimed fork sets with unsupported-scope reporting. |
| JSON-RPC | scheduled | `execution-apis` pinned in `spec-lock.toml`; RPC dependency admission starts at v0.63.0 and trust models follow at v0.64.0 |
| ABI encoding | scheduled | ABI type modeling starts at v0.70.0, value encode/decode at v0.71.0, and contract event/error decoding at v0.72.0 |
| Contract standards | scheduled | Common token standards, ENS, permit helpers, and interface helpers are scheduled for v0.74.0 through v0.77.0 |
| Engine API | scheduled | Engine API types and validation helpers are scheduled for v0.82.0 and v0.83.0 |
| SSZ and beacon consensus | scheduled | SSZ, beacon headers, light-client updates, and Beacon API boundaries are scheduled for v0.79.0 through v0.81.0 and v0.84.0 |
| DevP2P/RLPx and discovery | scheduled | Networking threat model, dependency admission, eth wire messages, and snap messages are scheduled for v0.86.0 through v0.89.0 |
| Txpool, sync, and node-adjacent boundaries | scheduled | Txpool policy, sync orchestration, and mining/builder/validator scope decisions are scheduled for v0.90.0 through v0.92.0 |

Every release that claims support for a fork, EIP, RPC method, or wire protocol
must update this matrix and `spec-lock.toml`.
