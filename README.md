# eth

`eth` is a `no_std`-first Rust workspace for Ethereum execution-layer protocol
building blocks.

The project target is a production-ready Ethereum crate at `1.0.0`, reached
through small releases with explicit security, conformance, and dependency
evidence. The first implementation work is intentionally conservative:
bounded canonical decoding, explicit fork context, stable crate boundaries, and
security documentation before RPC, signer, REVM, Reth, or P2P adapters become
real dependencies.

## Current Status

Status: repository foundation for `v0.1.0`.

Implemented now:

- Rust workspace pinned to stable `1.96.0`.
- MSRV policy for Rust `1.90.0` through `1.96.0`.
- `no_std` facade and focused first-party crates.
- No runtime dependencies and no third-party Ethereum dependencies yet.
- EUPL-1.2 license.
- Security, modularity, supply-chain, implementation, and release planning docs.
- Local check and release-gate scripts.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `EUPL-1.2` |
| MSRV | Rust `1.90.0` |
| Pinned toolchain | Rust `1.96.0` |
| Default target | `no_std` |
| Runtime dependencies | zero external crates |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Default features | protocol-core only |
| Network/signing defaults | none |
| 1.0 target | serious production-ready Ethereum execution-layer toolkit |

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. New deployments should use
the pinned stable Rust `1.96.0` until the toolchain policy is updated.

Compatibility evidence:

| Rust | Local Evidence |
| --- | --- |
| `1.90.0` | `cargo check --workspace --all-features` |
| `1.91.0` | `cargo check --workspace --all-features` |
| `1.92.0` | `cargo check --workspace --all-features` |
| `1.93.0` | `cargo check --workspace --all-features` |
| `1.94.0` | `cargo check --workspace --all-features` |
| `1.95.0` | `cargo check --workspace --all-features` |
| `1.96.0` | full check gate |

## Workspace Shape

| Crate | Default | Purpose |
| --- | --- | --- |
| `eth` | yes | Facade crate over stable protocol-core crates. |
| `eth-valkyoth-primitives` | yes | Chain, fork, block, gas, nonce, and bounded value types. |
| `eth-valkyoth-codec` | yes | Bounded exact-consumption wire decoding policy. |
| `eth-valkyoth-protocol` | yes | Fork-aware validation states and protocol context. |
| `eth-valkyoth-verify` | yes | Verification boundaries for signatures, proofs, and replay domains. |
| `eth-valkyoth-evm` | no | Future REVM adapter boundary. |
| `eth-valkyoth-rpc` | no | Future explicit RPC trust-policy boundary. |
| `eth-valkyoth-signer` | no | Future signer isolation boundary. |
| `eth-valkyoth-reth` | no | Future Reth integration boundary. |
| `eth-valkyoth-testkit` | no | Test fixtures, conformance helpers, and adversarial inputs. |

## Checks

```bash
scripts/checks.sh
scripts/release_0_2_gate.sh
```

For dependency-policy checks, install `cargo-deny` and `cargo-audit`, then run:

```bash
cargo deny check
cargo audit
```

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Release Plan](docs/RELEASE_PLAN.md)
- [Scope](docs/SCOPE.md)
- [Threat Model](docs/threat-model.md)
- [Spec Matrix](docs/SPEC_MATRIX.md)
- [Spec Source Policy](docs/spec-source-policy.md)
- [GitHub Security Settings](docs/github-security-settings.md)
- [Secret Handling Policy](docs/secret-handling-policy.md)
- [Modularity Policy](docs/modularity-policy.md)
- [Supply-Chain Security](docs/supply-chain-security.md)
- [Unsafe Policy](docs/unsafe-policy.md)

## License

Licensed under the European Union Public Licence 1.2.
