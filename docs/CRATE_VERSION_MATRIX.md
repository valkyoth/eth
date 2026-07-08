# Crate Version Matrix

Status: `v0.47.0` adds native ECRECOVER precompile execution.
`eth-valkyoth-evm-core` now exposes dependency-free `no_std` stack, memory,
word, opcode, program-counter, fork, gas, state, error, and bounded
interpreter domains for basic stack/control-flow bytecode plus explicit host
state reads. The fork domain includes explicit historical Ethereum fork
identifiers, opcode-introduction metadata, and fork-specific state-read pricing
for the currently executable state opcode subset. The call/create domain now
adds explicit frame-depth, static-write, return-data, and journal checkpoint
policies while execution still fails closed before host calls or commits. The
precompile domain adds fork-aware descriptors, bounded input/gas planning,
dependency-free identity, SHA-256, and RIPEMD-160 execution, and ECRECOVER
execution through explicit caller-provided secp256k1 and Keccak boundaries
while remaining cryptographic precompiles fail closed until audited backends or
first-party implementations are admitted.

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package
contents need a new crates.io version.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | `eth` uses the milestone version, for example `0.20.0`; support crates use their next independent minor, for example `0.8.0` to `0.9.0`. | Yes |
| `dependency` | Patch-bump the existing line, for example `0.3.1` to `0.3.2`. | Yes |
| `metadata` | Use the milestone version when republishing corrected package metadata. | Yes |
| `unchanged` | Keep the previous published version. | No |

`dependency` means the crate did not receive meaningful implementation or API
changes, but its manifest must change because a related workspace crate changed
in a way that the published dependency range cannot cover.

`metadata` means the crate did not receive meaningful implementation changes,
but must be republished so immutable crates.io package metadata is corrected.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests before release. The script refuses accidental lockstep
publication when a crate is marked `unchanged`.

## v0.47.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.19.0` | `0.19.0` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-primitives` | `0.11.2` | `0.11.2` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-hash` | `0.11.2` | `0.11.2` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-protocol` | `0.25.2` | `0.25.2` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-verify` | `0.21.0` | `0.21.0` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-derive` | `0.17.2` | `0.17.2` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-sanitization` | `0.7.4` | `0.7.4` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-evm-core` | `0.9.0` | `0.10.0` | `code` | Yes | Adds caller-backend ECRECOVER precompile execution without default secp256k1 or Keccak dependencies. |
| `eth-valkyoth-evm` | `0.10.0` | `0.10.0` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-signer` | `0.7.3` | `0.7.3` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.47.0. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.47.0. |
| `eth` | `0.46.0` | `0.47.0` | `code` | Yes | Updates the optional `evm-core` dependency to `eth-valkyoth-evm-core 0.10.0` and documents native ECRECOVER execution. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
