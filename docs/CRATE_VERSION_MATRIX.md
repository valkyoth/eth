# Crate Version Matrix

Status: `v0.52.2` corrects truncated `PUSH1..=PUSH32` consensus semantics by
right-zero-padding missing immediate bytes at code EOF and sharing declared
instruction advancement between execution and jump-destination analysis.
`eth-valkyoth-evm-core` now exposes dependency-free `no_std` stack, memory,
word, opcode, program-counter, fork, gas, state, error, and bounded
interpreter domains for basic stack/control-flow bytecode plus explicit host
state reads. The fork domain includes explicit historical Ethereum fork
identifiers, opcode-introduction metadata, and fork-specific state-read pricing
for the currently executable state opcode subset. The call/create domain now
adds explicit frame-depth, static-write, return-data, and journal checkpoint
policies while execution still fails closed before host calls or commits. The
precompile domain adds fork-aware descriptors, bounded input/gas planning,
dependency-free identity, SHA-256, RIPEMD-160, bounded ModExp, BN254 add/mul,
and BN254 pairing plus G2 subgroup validation, Fp6/Fp12 tower
arithmetic, validated tuple streaming, plan-level gas-meter charging,
line-function foundation, Miller-loop accumulation with sparse line-factor
multiplication evidence, optimized bounded final exponentiation, Frobenius point mapping
for the optimal-ate post-loop inputs, the projective post-loop line carrier,
canonical EIP-197 zero/one result-word admission, and EIP-152 BLAKE2F
execution with exact input-shape validation, final-flag validation, and
round-count gas. Advanced-precompile planning also enforces exact or non-empty
item frames, official BLS MSM discount gas, pairing gas, and fixed output
lengths. The wire domain rejects non-zero Fp padding, values at or above the
field modulus, partial frames, and alternate infinity representations before
arithmetic.
ECRECOVER executes through explicit caller-provided secp256k1 and Keccak
boundaries while KZG and BLS cryptographic precompiles fail closed until
audited backends or first-party implementations are admitted.

`eth` uses independent crate versions. The facade crate remains the main user
entry point, but support crates are published only when their own package
contents need a new crates.io version.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | `eth` uses the milestone version, for example `0.20.0`; support crates use their next independent minor, for example `0.8.0` to `0.9.0`. | Yes |
| `bugfix` | API-compatible support-crate correctness or security fixes increment the current patch exactly once, for example `0.8.0` to `0.8.1`. | Yes |
| `dependency` | Patch-bump the existing line, for example `0.3.1` to `0.3.2`. | Yes |
| `metadata` | Use the milestone version when republishing corrected package metadata. | Yes |
| `unchanged` | Keep the previous published version. | No |

`dependency` means the crate did not receive meaningful implementation or API
changes, but its manifest must change because a related workspace crate changed
in a way that the published dependency range cannot cover.

`bugfix` means implementation changed to correct behavior while preserving the
public API and support-crate type identity expected by the compatible facade
release line. It must not introduce new public API or remove existing API.

`metadata` means the crate did not receive meaningful implementation changes,
but must be republished so immutable crates.io package metadata is corrected.

`scripts/release_crates.py --check` validates `release-crates.toml` against the
workspace manifests before release. The script refuses accidental lockstep
publication when a crate is marked `unchanged`.

## v0.52.2 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `eth-valkyoth-codec` | `0.19.0` | `0.19.0` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-primitives` | `0.11.2` | `0.11.2` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-hash` | `0.11.2` | `0.11.2` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-protocol` | `0.25.2` | `0.25.2` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-verify` | `0.23.0` | `0.23.0` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-derive` | `0.17.3` | `0.17.3` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-sanitization` | `0.7.5` | `0.7.5` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-evm-core` | `0.26.0` | `0.26.1` | `bugfix` | Yes | Corrects truncated PUSH consensus handling, shares instruction advancement with jump-destination analysis, and retains support-crate type identity plus the legacy error variant for facade compatibility. |
| `eth-valkyoth-evm` | `0.10.0` | `0.10.0` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-rpc` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-signer` | `0.7.3` | `0.7.3` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-reth` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.2. |
| `eth-valkyoth-testkit` | `0.7.0` | `0.7.0` | `unchanged` | No | No package changes for v0.52.2. |
| `eth` | `0.52.1` | `0.52.2` | `code` | Yes | Updates `evm-core` to `0.26.1`, exposes consensus-correct truncated PUSH execution, and preserves support-crate type identity plus the legacy public error variant for patch compatibility. |

Update this table and `release-crates.toml` in the same commit whenever a crate
changes release state.
