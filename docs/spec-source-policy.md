# eth Spec Source Policy

`eth` must not implement consensus-sensitive Ethereum behavior from memory.

Every milestone that changes Ethereum protocol behavior must start by checking
the current official source material, pinning exact revisions, and recording
the evidence used by tests.

## Official Sources

Use these as primary sources:

- Execution layer: `https://github.com/ethereum/execution-specs`
- EIPs: `https://github.com/ethereum/EIPs`
- Execution APIs: `https://github.com/ethereum/execution-apis`
- Consensus specs: `https://github.com/ethereum/consensus-specs`

Use consensus specs only when the milestone touches SSZ, beacon-chain concepts,
Engine API context, post-merge consensus boundaries, or consensus-layer proof
material.

## Required Workflow

Before implementing or changing consensus-sensitive behavior:

1. Check the current official source repositories.
2. Select the exact tags, releases, or commit hashes relevant to the milestone.
3. Record those revisions in `spec-lock.toml`.
4. Download or import only the required fixtures/spec files into the configured
   external reference store.
5. Add or update tests that use those pinned materials.
6. Update `docs/SPEC_MATRIX.md` with the claimed status and evidence.
7. State in release notes which spec and fixture revisions were used.

If the official sources disagree, are ambiguous, or have no fixture for the
behavior, stop and document the ambiguity before implementing. Do not silently
choose behavior based on memory, blog posts, or a single client implementation.

## Local Reference Store

External Ethereum reference material belongs outside this repository. The
default local path is:

```text
../../test/eth
```

from the repository root, which resolves to `/home/eldryoth/Work/test/eth` in
the maintainer's current checkout layout. Other developers and CI may override
the location with `ETH_REFERENCE_STORE`.

This repository records revision metadata and test expectations, not large
upstream repositories unless a release explicitly requires vendored fixtures.

Use `scripts/sync_spec_sources.py` to materialize or verify the pinned
repositories:

```sh
scripts/sync_spec_sources.py
scripts/sync_spec_sources.py --check
scripts/sync_spec_sources.py --lock-only
```

See [Ethereum Reference Store](reference-store.md) for the current pinned
revisions, local path, and fixture license notes.

## Dependency And Tool Review

When a spec milestone requires third-party crates or tooling, review the latest
versions at the same time as the official Ethereum sources. Dependency
admission still follows [Supply-Chain Security](supply-chain-security.md).

When execution or fork-aware behavior is active, maintain an advisory upstream
check script that compares the pinned `spec-lock.toml` revisions with current
official Ethereum hardfork/spec repositories, execution fixtures, and any
temporary reference engine such as REVM. The script should report when a
maintenance release may be needed for new fork rules, opcodes, gas schedules,
precompiles, transaction types, or fixtures. Upstream movement alone is not a
release claim; claims are updated only after the new sources are pinned,
implemented, tested, and pentested.
