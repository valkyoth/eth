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
- Third-party dependencies to first-party APIs.

## Baseline Mitigations

- Decode budgets are explicit.
- Core crates are `no_std`.
- First-party protocol-facing crates forbid unsafe code.
- Network, signer, EVM, Reth, and P2P code are non-default.
- Unknown registries and git sources are denied.
- Release checks require tests, dependency policy, and security review evidence.

## Residual Risks

- `no_std` does not prevent logic bugs or dependency vulnerabilities.
- Zeroization cannot guarantee erasure of all historical copies.
- TLS authenticates transport endpoints but not Ethereum state correctness.
- Quorum RPC reduces provider risk but is not cryptographic verification.
