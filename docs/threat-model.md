# eth Threat Model

Status: initial threat model for repository foundation.

## Assets

- Correct protocol interpretation.
- Fork and chain context.
- Private keys, signer credentials, and signing preimages.
- RPC endpoint credentials and request privacy.
- Verified state, proof, transaction, and receipt data.
- Release artifacts and dependency integrity.

## Adversaries

- Remote peers or RPC servers returning malformed or inconsistent data.
- Attackers supplying oversized, deeply nested, or noncanonical inputs.
- Malicious or compromised RPC providers.
- Dependency or CI supply-chain attackers.
- Local attackers trying to recover secrets from logs, panics, or memory.

## Trust Boundaries

- Wire bytes to decoded values.
- Decoded values to canonical protocol values.
- Canonical values to fork-validated values.
- Fork-validated values to executed or committed state.
- RPC responses to trusted, quorum, or verified data.
- Signer requests to key material.
- Caller-provided Keccak-256 and recoverable secp256k1 backends to sender,
  authorization, and proof verification results.
- Third-party dependencies to first-party APIs.

## Baseline Mitigations

- Decode budgets are explicit.
- Core crates are `no_std`.
- First-party protocol-facing crates forbid unsafe code.
- Network, signer, EVM, Reth, and P2P code are non-default.
- Unknown registries and git sources are denied.
- Release checks require tests, dependency policy, and security review evidence.

## Cryptographic Backends

`eth-valkyoth-hash::Keccak256` and
`eth-valkyoth-verify::RecoverableSecp256k1` are explicit caller-provided
backend boundaries. A wrong Keccak backend can derive a wrong address or trie
key, and a wrong secp256k1 backend can recover a wrong signer.

The library centrally validates Ethereum signature scalar policy before any
recoverable secp256k1 backend is called: `r` and `s` must be nonzero
secp256k1 scalars, and `s` must satisfy the EIP-2 low-s bound. Backends are
still trusted for actual public-key recovery and should have KATs, malformed
signature tests, and documented state-clearing behavior if they hold mutable
cryptographic state.

## Residual Risks

- `no_std` does not prevent logic bugs or dependency vulnerabilities.
- Zeroization cannot guarantee erasure of all historical copies.
- TLS authenticates transport endpoints but not Ethereum state correctness.
- Quorum RPC reduces provider risk but is not cryptographic verification.
