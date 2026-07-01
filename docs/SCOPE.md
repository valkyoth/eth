# eth Scope

`eth` targets Ethereum toolkit functionality with conservative defaults.

## In Scope

- Bounded canonical decoding of execution-layer data.
- Typed transaction envelopes and fork-aware validation.
- Header, receipt, transaction, and proof verification.
- Optional EVM execution through REVM.
- Optional RPC client policy with explicit trust models.
- Optional signer boundary with external-signer-first design.
- Optional Reth integration at adapter boundaries.
- Optional ABI, contract-standard, ENS, consensus, Engine API, networking,
  txpool, sync, and node-adjacent boundaries after their versioned roadmap
  gates.
- Conformance evidence against pinned upstream test revisions.

## Default-Off Or Decision-Gated Before 1.0

- Full execution-client behavior unless each required boundary has a versioned
  implementation and verification path.
- Consensus, Engine API, and beacon behavior outside the scheduled optional
  consensus milestones.
- Validator-adjacent behavior until the mining, builder, and validator boundary
  decision milestone completes.
- P2P networking in default builds.
- Hardcoded public RPC endpoints.
- Implicit transaction broadcast fanout.
- Local key storage as a default feature.
- Marketing claims that `no_std` alone provides security.
