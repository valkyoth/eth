# Contributing To eth

`eth` is security-sensitive Ethereum protocol infrastructure. Contributions
must keep the workspace small, explicit, tested, and honest about what is
stable.

## License

`eth` is licensed under the European Union Public Licence 1.2. By contributing,
you agree that your contribution is provided under the same license.

## Development Setup

Use the pinned Rust toolchain from `rust-toolchain.toml`.

```bash
cargo check --workspace --all-features
cargo test --workspace
```

Before opening a pull request, run:

```bash
scripts/checks.sh
```

## Security-Sensitive Changes

Treat these areas as high risk:

- wire decoding and resource limits;
- fork selection and validation state transitions;
- cryptographic verification and proof validation;
- signer and key-management APIs;
- RPC endpoint policy and response trust;
- EVM execution and state commit boundaries;
- Reth and P2P adapters;
- CI, release scripts, and dependency updates.

Do not post exploitable security details in public issues. Follow
[SECURITY.md](../SECURITY.md).

## Dependency Policy

When adding or updating crates:

- use crates.io releases unless there is a documented reason not to;
- avoid git dependencies;
- check latest versions before editing dependency declarations;
- keep `Cargo.lock` updated;
- run `cargo deny check` and `cargo audit`;
- document why the crate belongs in this workspace.
