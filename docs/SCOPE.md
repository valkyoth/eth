# eth Scope

`eth` targets Ethereum execution-layer toolkit functionality.

## In Scope

- Bounded canonical decoding of execution-layer data.
- Typed transaction envelopes and fork-aware validation.
- Header, receipt, transaction, and proof verification.
- Optional EVM execution through REVM.
- Optional RPC client policy with explicit trust models.
- Optional signer boundary with external-signer-first design.
- Optional Reth integration at adapter boundaries.
- Conformance evidence against pinned upstream test revisions.

## Out Of Scope Before 1.0

- Full execution client implementation.
- Consensus client or validator client implementation.
- Default P2P networking.
- Hardcoded public RPC endpoints.
- Implicit transaction broadcast fanout.
- Local key storage as a default feature.
- Marketing claims that `no_std` alone provides security.
