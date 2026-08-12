# Differential Test Harness

Status: `v0.55.0` covers structural RLP and arbitrary-length ModExp through
independent in-process or external-client reference paths.

## Scope

| Area | Local path | Independent reference | Claim |
| --- | --- | --- | --- |
| Structural RLP | `eth-valkyoth-codec::differential_rlp_reference` | `alloy-rlp` `0.3.16` | Valid/invalid structural decisions and exact accepted re-encoding match for the curated corpus. |
| ModExp arithmetic | `eth-valkyoth-evm-core::modexp_differential` | `num-bigint` `0.5.1` | Exact output matches from 1 through 256-byte widths plus leading-zero, even, zero, unequal-width, sparse, truncated, and right-padded operands. |
| ModExp client behavior | `modexp_client_vectors` through precompile `0x05` | Geth `1.17.5`, Besu `26.7.1`, and Nethermind `1.39.3` | All 11 deterministic frames return byte-identical output from every client. |

Structural RLP comparison cannot distinguish every Ethereum integer-domain
rule from ordinary byte-string validity. Codec integer tests, primitive bridge
tests, and fuzz targets cover those semantic domains separately.

## Commands

Validate that the integration tests and first-party client vectors compile and
that all three client images are immutably pinned:

```sh
scripts/run_differential_tests.py --check
```

Run every reference path, including the external clients:

```sh
scripts/run_differential_tests.py
```

The runner executes the in-process paths and then
`scripts/run_modexp_client_differential.py`:

```sh
cargo test -p eth-valkyoth-codec --test differential_rlp_reference --features testing
cargo test -p eth-valkyoth-evm-core --test modexp_differential
```

The external runner requires Podman and network access to the official GitHub
release APIs and container registry. It fails when a pin is no longer the
latest stable upstream release, pulls each image by immutable multi-platform
digest, starts one client at a time, and compares 11 first-party return values
through `eth_call` to precompile `0x05`.

Each disposable container has no host mount, has `no-new-privileges`, joins a
temporary internal Podman network without outbound access, and publishes RPC
only on a random host loopback port. Client logs remain under
`target/modexp-client-differential/`; a mismatch reports the client, case, and
log path but does not dump calldata into ordinary release output.

The release host must delegate the cgroup v2 `cpu`, `memory`, and `pids`
controllers to rootless Podman. The runner checks this before pulling or
starting clients and fails closed when a controller is unavailable. Each
client receives a 2 GiB memory/swap ceiling, two-CPU quota, 512-PID ceiling,
read-only root filesystem, capability-free process, bounded tmpfs data and
temporary directories, and a 10 MiB container-log ceiling. Besu starts
directly as its image-defined unprivileged `1000:1000` account rather than
retaining capabilities for its root user-switch wrapper.

Podman operations and cleanup have explicit timeouts. Podman assigns the host
port atomically, the mapping must resolve to `127.0.0.1`, loopback RPC bypasses
environment proxy configuration, and RPC/release-metadata responses have fixed
byte ceilings.

The fuzz workspace also includes `rlp_differential` and `modexp_frame`.
`modexp_frame` drives 256-bit length parsing, both fork gas formulas, bounded
workspace admission, and atomic execution. Every payable frame within harness
capacity must execute successfully. Its execution allocation is harness-capped;
wider declarations still reach parsing and gas calculation.

## Mismatch Reporting

The RLP test accumulates all mismatches before failing and reports each corpus
case and mismatch class. In-process ModExp failures report the operand shape
whose local output differs from the independent BigUint result. External
client mismatches report the client and deterministic case name. A missing,
stale, misidentified, failed, or timed-out client fails closed.

## Comparison Boundary

The external comparison proves ModExp output semantics, not cross-client gas
accounting. The disposable development chains activate different fork
schedules, while `v0.55.0` claims EIP-198/EIP-2565 gas through Prague. Fork
formula tests, official vectors, fuzzing, and work-per-gas benchmarks remain
the authoritative gas evidence. Osaka EIP-7823/EIP-7883 behavior is not
claimed by this release.
