# eth Specification Matrix

Status: source revisions pinned for `v0.33.0`; scalar, list, and canonical
integer RLP decoding, canonical RLP encoding helpers, primitive RLP bridging,
Keccak-256 trait boundary, RLP fuzz harness baseline, and transaction envelope
shell plus unvalidated legacy, EIP-2930 access-list, EIP-1559 dynamic-fee,
EIP-4844 blob, and EIP-7702 set-code transaction decoding and canonical
encoding implemented. Explicit chain and fork activation context is available
for caller-reviewed specs.
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
trusted roots.

Official source and fixture revisions are governed by
[Spec Source Policy](spec-source-policy.md). Revisions were checked against
upstream `HEAD` on 2026-07-01 for `v0.18.0`; later execution-apis,
consensus-specs, ABI, contract-standard, and networking milestones must refresh
their own pinned source evidence before implementation. Consensus-sensitive
behavior must not be implemented from memory. EIP-4895 was checked at the
pinned EIPs revision in `spec-lock.toml` on 2026-07-02 for `v0.30.0`. The
pinned `execution-specs` Merkle Patricia Trie source was checked on 2026-07-03
for `v0.31.0` and reused for v0.32.0 proof walking.

| Area | Status | Evidence |
| --- | --- | --- |
| Execution RLP | partial | `ethereum/tests` pinned in `spec-lock.toml`; scalar byte-string, list, canonical integer decoders, canonical encoding helpers, and public conservative RLP derives implemented |
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
| Transaction validation | partial | `execution-specs` pinned in `spec-lock.toml`; v0.18.0 adds proof-gated decoded/canonical/fork-valid/sender-recovered transaction state transitions, v0.19.0 adds replay-domain checks, v0.20.0 adds digest-level sender recovery with low-s and y-parity policy, v0.22.0 adds transaction signing-hash construction for legacy EIP-155, EIP-2930, EIP-1559, and EIP-4844, v0.23.0 adds decoded transaction signature validation helpers, v0.24.1 adds EIP-7702 set-code transaction and authorization signature validation, and v0.24.2 adds the EIP-7702 set-code context validity gate. Remaining concrete proof constructors remain planned. |
| Header decoding and hashing | syntactic decode/hash | `execution-specs` pinned in `spec-lock.toml`; v0.28.0 decodes legacy, London, Shanghai, Cancun, and Prague header field sets and hashes canonical header RLP through the Keccak trait boundary without claiming full header validity |
| Receipt decoding | syntactic decode | EIP-658 and EIP-2718 checked for status/root and typed receipt envelopes; v0.29.0 decodes legacy and typed receipts, validates bloom/log/topic shape, and does not itself claim receipt-trie or block-root validity |
| Withdrawal decoding | syntactic decode | EIP-4895 checked for withdrawal list and entry shape; v0.30.0 decodes canonical withdrawal lists with `uint64` indexes, 20-byte recipient addresses, and nonzero Gwei amounts, and does not claim consensus-layer dequeue correctness, header `withdrawals_root` matching, or state-balance application |
| Header validation | planned | `execution-specs` pinned in `spec-lock.toml`; ancestry, root, gas, base-fee, fork-activation, and consensus-layer commitment validation not implemented |
| Receipt and withdrawal validation | planned | `execution-specs` pinned in `spec-lock.toml`; receipt-trie membership, block `receipts_root` matching, transaction/receipt type matching, cumulative gas monotonicity, withdrawal trie-root matching, and withdrawal state application are not implemented |
| MPT node decoding | syntactic decode | `execution-specs` pinned in `spec-lock.toml`; v0.31.0 decodes branch, extension, and leaf node shape with compact-path and child-reference checks plus cumulative proof-node count/byte accounting |
| MPT proofs | transaction/receipt/account/storage inclusion | `execution-specs` pinned in `spec-lock.toml`; v0.32.0 verifies transaction and receipt inclusion at `rlp(transaction_index)` against trusted root newtypes through the Keccak trait boundary, and v0.33.0 verifies account and storage inclusion at `keccak256(address)` and `keccak256(slot_key)` with distinct root/key domains. The APIs distinguish malformed, absent, and wrong-root/value-mismatch proofs without claiming header-root, account-state, or storage-root composition validity. |
| JSON-RPC | scheduled | `execution-apis` pinned in `spec-lock.toml`; RPC dependency admission starts at v0.40.0 and trust models follow at v0.41.0 |
| ABI encoding | scheduled | ABI type modeling starts at v0.47.0, value encode/decode at v0.48.0, and contract event/error decoding at v0.49.0 |
| Contract standards | scheduled | Common token standards, ENS, permit helpers, and interface helpers are scheduled for v0.51.0 through v0.54.0 |
| Engine API | scheduled | Engine API types and validation helpers are scheduled for v0.59.0 and v0.60.0 |
| SSZ and beacon consensus | scheduled | SSZ, beacon headers, light-client updates, and Beacon API boundaries are scheduled for v0.56.0 through v0.58.0 and v0.61.0 |
| DevP2P/RLPx and discovery | scheduled | Networking threat model, dependency admission, eth wire messages, and snap messages are scheduled for v0.63.0 through v0.66.0 |
| Txpool, sync, and node-adjacent boundaries | scheduled | Txpool policy, sync orchestration, and mining/builder/validator scope decisions are scheduled for v0.67.0 through v0.69.0 |

Every release that claims support for a fork, EIP, RPC method, or wire protocol
must update this matrix and `spec-lock.toml`.
