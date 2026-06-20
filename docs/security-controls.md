# eth Security Controls

Initial controls:

- `#![forbid(unsafe_code)]` in first-party crates.
- `no_std` default crates.
- Release-profile overflow checks.
- Explicit decode budgets.
- Exact-consumption decoding policy.
- No default network, signer, local keystore, Reth, or P2P features.
- Dependency source restrictions through `deny.toml`.
- CodeQL default setup expected in GitHub settings.

Future controls:

- Fuzzing for every untrusted parser, bootstrapped in `fuzz/` before the first
  RLP parser ships.
- Official Ethereum conformance suites.
- Differential tests against independent implementations.
- RPC adversarial server tests.
- Secret redaction tests.
- SBOM and signed release manifests.
