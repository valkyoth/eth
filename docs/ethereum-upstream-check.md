# Ethereum Upstream Check

Status: v0.37.1 adds the advisory checker.

`scripts/check_ethereum_upstream.py` is a maintenance advisory tool. It checks
whether local review data still matches upstream registry and official
Ethereum source metadata.

The checker is intentionally read-only:

- crates.io is queried through JSON metadata with an explicit user agent;
- official Ethereum repositories are queried with `git ls-remote`;
- fetched bytes are never executed, evaluated, sourced, or piped into another
  command;
- repository URLs must match `https://github.com/ethereum/...`;
- pinned source revisions must be lowercase 40-character commit hashes.

## What It Reports

The REVM section reports:

- latest `revm` registry version;
- latest `revm` Rust version requirement;
- newest `revm` line compatible with this workspace's Rust `1.90.0` floor;
- the same values for `revm-primitives`.

The Ethereum source section reports each pinned source in `spec-lock.toml`
against its current remote `HEAD`.

Remote movement is not an immediate failure. It is maintenance input. A moved
source means the next implementation or maintenance release must decide whether
the new upstream revision changes fork rules, opcode/gas schedules,
precompiles, transaction types, fixtures, JSON-RPC behavior, or consensus
fixtures relevant to this crate.

## Commands

```bash
scripts/check_ethereum_upstream.py --local-only
scripts/check_ethereum_upstream.py
```

Use `--local-only` in broad local checks when network access is not required.
The release gate runs the full networked advisory check.
