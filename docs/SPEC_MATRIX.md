# eth Specification Matrix

Status: source revisions pinned for `v0.24.2`; scalar, list, and canonical
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
structured-data sender recovery is trusted.

Official source and fixture revisions are governed by
[Spec Source Policy](spec-source-policy.md). Revisions were checked against
upstream `HEAD` on 2026-07-01 for `v0.18.0`; later execution-apis,
consensus-specs, ABI, contract-standard, and networking milestones must refresh
their own pinned source evidence before implementation. Consensus-sensitive
behavior must not be implemented from memory.

| Area | Status | Evidence |
| --- | --- | --- |
| Execution RLP | partial | `ethereum/tests` pinned in `spec-lock.toml`; scalar byte-string, list, canonical integer decoders, and canonical encoding helpers implemented |
| RLP fuzz harness | baseline | `fuzz/` workspace builds; committed hex seeds live under `fuzz/seed-corpus/`; crash reproduction is documented |
| Keccak-256 hashing | boundary only | `eth-valkyoth-hash` defines caller-provided Keccak-256 trait boundary; no concrete backend admitted |
| EIP-712 structured data | domain gate | EIP-712 defines the `0x1901` signing digest and optional domain fields; v0.21.0 checks required caller-provided `chainId` and `verifyingContract` fields and builds the signing digest from supplied domain/message hashes; full typed-data encoding is scheduled for v0.26.0 |
| EIP-2718 typed transactions | partial | `ethereum/EIPs` pinned in `spec-lock.toml`; envelope classification implemented; EIP-2930 type `0x01`, EIP-1559 type `0x02`, EIP-4844 type `0x03`, and EIP-7702 set-code type `0x04` field decode and canonical encode implemented; later typed transaction payloads remain opaque until explicitly admitted |
| Legacy transactions | field decode/encode | EIP-2718 defines the legacy transaction field list; v0.12.0 decodes fields into an unvalidated model and v0.16.0 encodes that admitted model without signature, sender, chain, or fork validation |
| EIP-2930 access-list transactions | field decode/encode | EIP-2930 defines type `0x01`, eleven payload fields, and access-list shape; v0.13.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, gas, duplicate, chain, account-state, or fork validation |
| EIP-1559 dynamic-fee transactions | field decode/encode | EIP-1559 defines type `0x02`, twelve payload fields, and access-list inheritance from EIP-2930; v0.14.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, fee-order, gas, duplicate, chain, account-state, or fork validation |
| EIP-4844 blob transactions | field decode/encode | EIP-4844 defines type `0x03`, fourteen payload fields, required 20-byte `to`, max blob fee, and blob versioned hash list; v0.15.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, blob fee, KZG, data availability, blob-hash version, blob count, chain, account-state, block blob-gas, or fork validation |
| EIP-7702 set-code transactions | validity gate | EIP-7702 defines type `0x04`, thirteen payload fields, required 20-byte destination, authorization tuples shaped `[chain_id, address, nonce, y_parity, r, s]`, transaction signing over `0x04 || payload`, authorization signing over `0x05 || rlp([chain_id, address, nonce])`, non-empty authorization lists, authorization chain binding, nonce policy, and empty-or-delegated authority code. v0.24.0 decodes and encodes the admitted model, v0.24.1 adds transaction signing-hash plus authorization signer recovery, and v0.24.2 adds the non-cryptographic context validity gate with EIP-7702 per-tuple skip accounting. |
| Chain and fork specs | explicit context | `execution-specs` and EIPs are pinned in `spec-lock.toml`; v0.17.0 adds caller-provided `ChainSpec`, `ForkSpec`, hardfork identity, block/timestamp activation checks, unsupported-fork errors, chain-mismatch errors, duplicate-fork errors, and non-monotonic fork/activation ordering errors without hardcoding mainnet validation rules |
| Transaction validation | partial | `execution-specs` pinned in `spec-lock.toml`; v0.18.0 adds proof-gated decoded/canonical/fork-valid/sender-recovered transaction state transitions, v0.19.0 adds replay-domain checks, v0.20.0 adds digest-level sender recovery with low-s and y-parity policy, v0.22.0 adds transaction signing-hash construction for legacy EIP-155, EIP-2930, EIP-1559, and EIP-4844, v0.23.0 adds decoded transaction signature validation helpers, v0.24.1 adds EIP-7702 set-code transaction and authorization signature validation, and v0.24.2 adds the EIP-7702 set-code context validity gate. Remaining concrete proof constructors remain planned. |
| Header validation | planned | `execution-specs` pinned in `spec-lock.toml`; validation not implemented |
| Receipt validation | planned | `execution-specs` pinned in `spec-lock.toml`; validation not implemented |
| MPT proofs | planned | `ethereum/tests` pinned in `spec-lock.toml`; proof verification not implemented |
| JSON-RPC | scheduled | `execution-apis` pinned in `spec-lock.toml`; RPC dependency admission starts at v0.40.0 and trust models follow at v0.41.0 |
| ABI encoding | scheduled | ABI type modeling starts at v0.47.0, value encode/decode at v0.48.0, and contract event/error decoding at v0.49.0 |
| Contract standards | scheduled | Common token standards, ENS, permit helpers, and interface helpers are scheduled for v0.51.0 through v0.54.0 |
| Engine API | scheduled | Engine API types and validation helpers are scheduled for v0.59.0 and v0.60.0 |
| SSZ and beacon consensus | scheduled | SSZ, beacon headers, light-client updates, and Beacon API boundaries are scheduled for v0.56.0 through v0.58.0 and v0.61.0 |
| DevP2P/RLPx and discovery | scheduled | Networking threat model, dependency admission, eth wire messages, and snap messages are scheduled for v0.63.0 through v0.66.0 |
| Txpool, sync, and node-adjacent boundaries | scheduled | Txpool policy, sync orchestration, and mining/builder/validator scope decisions are scheduled for v0.67.0 through v0.69.0 |

Every release that claims support for a fork, EIP, RPC method, or wire protocol
must update this matrix and `spec-lock.toml`.
