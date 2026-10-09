# Maintenance Review: 2026-10-09

Scope: v0.61.0 implementation candidate, following published v0.60.0.
Historical signed reports and gates retain their original toolchain evidence.

## Rust And Dependencies

The [official stable manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml)
reports Rust 1.99.0 (release date 2026-10-01); this is the new pinned toolchain.
Rust 1.90.0 remains the MSRV. The two-row facade/GitHub README table now records
1.90.0 through 1.98.1 compatibility checks and the 1.99.0 full gate.

The fuzzing toolchain was updated to the 2026-10-09 nightly channel,
`1.101.0-nightly (a30aa9064 2026-10-08)`. The final gate below used that
nightly, not the previously installed one. Installed Miri/target components
were preserved; this is not a claim that Miri or formal proofs were run.

The live registry checker found syn 3.0.6 and trybuild 1.0.122; sanitization
2.1.0 and the other direct dependencies remain current. Both lockfiles were
refreshed. Updates include cfg-if, ctutils, der, hybrid-array, libc, TOML
components, unicode-ident and zeroize; fuzz-only arbitrary, cc and
find-msvc-tools also moved. No third-party runtime crypto is added to the
default EVM core. Lockfile resolution is followed by explicit MSRV checks;
dependency freshness alone is not compatibility evidence.

## Tooling And Clients

Official registry/release checks found the existing pins current:

| Tool | Version |
| --- | --- |
| cargo-deny | 0.20.2 |
| cargo-audit | 0.22.2 |
| cargo-sbom | 0.10.0 |
| cargo-fuzz | 0.13.2 |
| actions/checkout | v7.0.1, SHA `3d3c42e5aac5ba805825da76410c181273ba90b1` |

Both GitHub workflows keep immutable Checkout SHAs and read-only permissions.
CodeQL remains Default setup; no advanced workflow is introduced.
The release gate keeps its implementation/tag phase separation and per-tag
incremental pentest; public publication remains every fifth minor.

Latest execution clients were resolved from official GitHub releases and their
container tags through Skopeo, then pinned by digest:

- Geth v1.17.8: `sha256:abf3605177f8bdcfce436985a8054cca4ae59256323a4ffb72c56c04f3d1fadd`.
- Besu 26.9.0: `sha256:fc1813d67d630d7660eea6ee555c01e73bfecda3e54d24f9484c1c4a52c6fcb0`.
- Nethermind 2.1.0: `sha256:25f72063f6fc8ba89f1c08abe1ed3211203b3cf303c9618cc3a2488e1e4ab2e0`.

Fresh G2 outcomes are recorded in [G2 addition](bls12-g2-addition.md).
The same updated clients also passed all 11 existing ModExp vectors each;
this is a fresh v0.61.0 regression run, not relabelled v0.55.0 evidence.
Besu's removed dev network required the Ephemery configuration; containers
remain isolated and explicitly cleaned, with stopped-container logs preserved
until cleanup. No host-service installation or privilege relaxation was needed.

## Protocol Scope

The current EIP-2537 still specifies 600 gas, 512-byte input, 256-byte output,
and no subgroup rejection for G2ADD. Pinned fixture hashes remain unchanged.
The broader upstream monitor remains advisory: remote-head movement is not
automatic protocol admission. The [Besu release](https://github.com/besu-eth/besu/releases/tag/26.9.0)
also flags fork-related changes, including EIP-7610 removal and Amsterdam
testnet scheduling. These are not retroactively admitted through this G2
maintenance pass. The existing v0.154.0 fork-manifest admission explicitly
tracks EIP-7610 removal; Amsterdam BAL/consensus rules have v0.238.0/v0.355.0
owners. No additional roadmap slot is needed for these already assigned items.

## Verification Commands

```sh
scripts/release_0_61_0_gate.sh --implementation
python3 scripts/run_bls_g2_client_differential.py
python3 scripts/run_modexp_client_differential.py
cargo check --workspace --no-default-features --locked
```

## Verification Results

The full implementation gate passed on 2026-10-09: workspace tests/doctests,
strict Clippy, package verification, all crate README examples, release
metadata/roadmap checks, SBOM consistency, independent arithmetic suites,
official fixtures, fixed-work benchmarks and fuzz compilation. The 34 gate
regression tests, seven G2 client-runner tests and 19 existing client-harness
regression tests passed.

Fresh bounded fuzz runs completed without a failure: G2ADD 19,884 executions,
G1ADD 31,810 and Fp2 5,171, each over 31 seconds. These are smoke runs, not
exhaustive fuzzing. Cargo Deny passed all categories; both lockfiles passed
Cargo Audit against 1,296 advisories.

All 15 historical compiler checks from 1.90.0 through 1.98.1 passed. The
workspace no-default-features check passed, as did facade `evm-core`
no-default-features cross-checks for Windows MSVC, FreeBSD, macOS, iOS,
Android, WebAssembly, big-endian PowerPC64, bare-metal Thumb and RISC-V.
Cross-checking compilation does not claim native runtime testing on each OS.

Both client suites passed with the updated images: 16 G2 outcomes and 11
ModExp outcomes per client. The fixture importer reproduced all nine positive
and seven rejection G2 vectors from the pinned upstream data. Test containers
and networks were removed by the runners after verification.

No final pentest or tagging approval is asserted by this maintenance review.
