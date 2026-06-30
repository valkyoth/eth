# eth Specification Matrix

Status: source revisions pinned for `v0.9.0`; scalar, list, and canonical
integer RLP decoding plus canonical RLP encoding helpers implemented.

Official source and fixture revisions are governed by
[Spec Source Policy](spec-source-policy.md). Revisions were checked against
upstream `HEAD` on 2026-06-30 for `v0.9.0`; execution-apis and consensus-specs
moved and remain deferred areas. Consensus-sensitive behavior must not be
implemented from memory.

| Area | Status | Evidence |
| --- | --- | --- |
| Execution RLP | partial | `ethereum/tests` pinned in `spec-lock.toml`; scalar byte-string, list, canonical integer decoders, and canonical encoding helpers implemented |
| EIP-2718 typed transactions | planned | `ethereum/EIPs` pinned in `spec-lock.toml`; parser not implemented |
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
