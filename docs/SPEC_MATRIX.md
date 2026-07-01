# eth Specification Matrix

Status: source revisions pinned for `v0.11.0`; scalar, list, and canonical
integer RLP decoding, canonical RLP encoding helpers, primitive RLP bridging,
Keccak-256 trait boundary, RLP fuzz harness baseline, and transaction envelope
shell implemented.

Official source and fixture revisions are governed by
[Spec Source Policy](spec-source-policy.md). Revisions were checked against
upstream `HEAD` on 2026-07-01 for `v0.11.0`; execution-apis and
consensus-specs remain deferred areas. Consensus-sensitive behavior must not be
implemented from memory.

| Area | Status | Evidence |
| --- | --- | --- |
| Execution RLP | partial | `ethereum/tests` pinned in `spec-lock.toml`; scalar byte-string, list, canonical integer decoders, and canonical encoding helpers implemented |
| RLP fuzz harness | baseline | `fuzz/` workspace builds; committed hex seeds live under `fuzz/seed-corpus/`; crash reproduction is documented |
| Keccak-256 hashing | boundary only | `eth-valkyoth-hash` defines caller-provided Keccak-256 trait boundary; no concrete backend admitted |
| EIP-2718 typed transactions | shell | `ethereum/EIPs` pinned in `spec-lock.toml`; envelope classification implemented, typed payloads remain opaque |
| Transaction validation | planned | `execution-specs` pinned in `spec-lock.toml`; validation not implemented |
| Header validation | planned | `execution-specs` pinned in `spec-lock.toml`; validation not implemented |
| Receipt validation | planned | `execution-specs` pinned in `spec-lock.toml`; validation not implemented |
| MPT proofs | planned | `ethereum/tests` pinned in `spec-lock.toml`; proof verification not implemented |
| JSON-RPC | deferred | `execution-apis` pinned in `spec-lock.toml`; RPC not implemented |
| Engine API | deferred | not part of first default scope |
| SSZ | deferred | consensus-layer feature only if admitted |
| DevP2P/RLPx | deferred | separate threat-model expansion required |

Every release that claims support for a fork, EIP, RPC method, or wire protocol
must update this matrix and `spec-lock.toml`.
