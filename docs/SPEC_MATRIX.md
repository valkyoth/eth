# eth Specification Matrix

Status: source revisions pinned for `v0.18.0`; scalar, list, and canonical
integer RLP decoding, canonical RLP encoding helpers, primitive RLP bridging,
Keccak-256 trait boundary, RLP fuzz harness baseline, and transaction envelope
shell plus unvalidated legacy, EIP-2930 access-list, EIP-1559 dynamic-fee, and
EIP-4844 blob transaction decoding and canonical encoding implemented. Explicit
chain and fork activation context is available for caller-reviewed specs.
Transaction typestate promotion is proof-gated.

Official source and fixture revisions are governed by
[Spec Source Policy](spec-source-policy.md). Revisions were checked against
upstream `HEAD` on 2026-07-01 for `v0.18.0`; execution-apis and
consensus-specs remain deferred areas. Consensus-sensitive behavior must not be
implemented from memory.

| Area | Status | Evidence |
| --- | --- | --- |
| Execution RLP | partial | `ethereum/tests` pinned in `spec-lock.toml`; scalar byte-string, list, canonical integer decoders, and canonical encoding helpers implemented |
| RLP fuzz harness | baseline | `fuzz/` workspace builds; committed hex seeds live under `fuzz/seed-corpus/`; crash reproduction is documented |
| Keccak-256 hashing | boundary only | `eth-valkyoth-hash` defines caller-provided Keccak-256 trait boundary; no concrete backend admitted |
| EIP-2718 typed transactions | partial | `ethereum/EIPs` pinned in `spec-lock.toml`; envelope classification implemented; EIP-2930 type `0x01`, EIP-1559 type `0x02`, and EIP-4844 type `0x03` field decode and canonical encode implemented; later typed transaction payloads remain opaque |
| Legacy transactions | field decode/encode | EIP-2718 defines the legacy transaction field list; v0.12.0 decodes fields into an unvalidated model and v0.16.0 encodes that admitted model without signature, sender, chain, or fork validation |
| EIP-2930 access-list transactions | field decode/encode | EIP-2930 defines type `0x01`, eleven payload fields, and access-list shape; v0.13.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, gas, duplicate, chain, account-state, or fork validation |
| EIP-1559 dynamic-fee transactions | field decode/encode | EIP-1559 defines type `0x02`, twelve payload fields, and access-list inheritance from EIP-2930; v0.14.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, fee-order, gas, duplicate, chain, account-state, or fork validation |
| EIP-4844 blob transactions | field decode/encode | EIP-4844 defines type `0x03`, fourteen payload fields, required 20-byte `to`, max blob fee, and blob versioned hash list; v0.15.0 decodes fields and v0.16.0 encodes the admitted model without signature, sender, blob fee, KZG, data availability, blob-hash version, blob count, chain, account-state, block blob-gas, or fork validation |
| Chain and fork specs | explicit context | `execution-specs` and EIPs are pinned in `spec-lock.toml`; v0.17.0 adds caller-provided `ChainSpec`, `ForkSpec`, hardfork identity, block/timestamp activation checks, unsupported-fork errors, chain-mismatch errors, duplicate-fork errors, and non-monotonic fork/activation ordering errors without hardcoding mainnet validation rules |
| Transaction validation | typestate shell | `execution-specs` pinned in `spec-lock.toml`; v0.18.0 adds proof-gated decoded/canonical/fork-valid/sender-recovered transaction state transitions, consumes tokens on successful promotion, returns the original token on failed promotion, and keeps public proof constructors deferred until proofs can be transaction-bound; replay-domain checks, signature recovery, and concrete validity rules remain planned |
| Header validation | planned | `execution-specs` pinned in `spec-lock.toml`; validation not implemented |
| Receipt validation | planned | `execution-specs` pinned in `spec-lock.toml`; validation not implemented |
| MPT proofs | planned | `ethereum/tests` pinned in `spec-lock.toml`; proof verification not implemented |
| JSON-RPC | deferred | `execution-apis` pinned in `spec-lock.toml`; RPC not implemented |
| Engine API | deferred | not part of first default scope |
| SSZ | deferred | consensus-layer feature only if admitted |
| DevP2P/RLPx | deferred | separate threat-model expansion required |

Every release that claims support for a fork, EIP, RPC method, or wire protocol
must update this matrix and `spec-lock.toml`.
