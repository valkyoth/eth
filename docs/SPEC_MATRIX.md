# eth Specification Matrix

Status: initial placeholder for `v0.1.0`.

Official source and fixture revisions are governed by
[Spec Source Policy](spec-source-policy.md). Consensus-sensitive behavior must
not be implemented from memory.

| Area | Status | Evidence |
| --- | --- | --- |
| Execution RLP | planned | fixture revision to be pinned |
| EIP-2718 typed transactions | planned | fixture revision to be pinned |
| Transaction validation | planned | execution-spec revision to be pinned |
| Header validation | planned | execution-spec revision to be pinned |
| Receipt validation | planned | execution-spec revision to be pinned |
| MPT proofs | planned | proof fixtures to be pinned |
| JSON-RPC | deferred | RPC spec revision to be pinned |
| Engine API | deferred | not part of first default scope |
| SSZ | deferred | consensus-layer feature only if admitted |
| DevP2P/RLPx | deferred | separate threat-model expansion required |

Every release that claims support for a fork, EIP, RPC method, or wire protocol
must update this matrix and `spec-lock.toml`.
