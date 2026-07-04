#!/usr/bin/env python3
"""Synchronize pinned Ethereum reference repositories outside this repo."""

from __future__ import annotations

import argparse
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC_LOCK = ROOT / "spec-lock.toml"
REPO_KEYS = (
    "execution_specs",
    "execution_tests",
    "eips",
    "execution_apis",
    "consensus_specs",
)


@dataclass(frozen=True)
class Source:
    name: str
    repo: str
    rev: str


def parse_spec_lock() -> tuple[Path, list[Source]]:
    values: dict[str, str] = {}
    for line in SPEC_LOCK.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith(("#", "[")):
            continue
        key, separator, value = stripped.partition("=")
        if separator != "=":
            continue
        values[key.strip()] = value.strip().strip('"')

    default_store = values.get("local_reference_store_default")
    if not default_store:
        raise ValueError("spec-lock.toml is missing local_reference_store_default")
    store = Path(os.environ.get(values.get("local_reference_store_env", ""), ""))
    if not str(store):
        store = (ROOT / default_store).resolve()

    sources: list[Source] = []
    for name in REPO_KEYS:
        repo = values.get(f"{name}_repo")
        rev = values.get(f"{name}_rev")
        if not repo or not rev:
            raise ValueError(f"spec-lock.toml is missing {name}_repo or {name}_rev")
        if len(rev) != 40 or any(char not in "0123456789abcdef" for char in rev):
            raise ValueError(f"{name}_rev must be a lowercase 40-character commit hash")
        sources.append(Source(name=name, repo=repo, rev=rev))
    return store, sources


def run(args: list[str], cwd: Path | None = None) -> None:
    subprocess.run(args, cwd=cwd, check=True)


def sync_source(store: Path, source: Source) -> None:
    checkout = store / source.name
    if checkout.exists():
        run(["git", "fetch", "--tags", "--prune", "origin"], cwd=checkout)
    else:
        checkout.parent.mkdir(parents=True, exist_ok=True)
        run(["git", "clone", source.repo, str(checkout)])
    run(["git", "checkout", "--detach", source.rev], cwd=checkout)


def verify_source(store: Path, source: Source) -> None:
    checkout = store / source.name
    if not checkout.exists():
        raise FileNotFoundError(f"{checkout} does not exist; run without --check")
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=checkout,
        check=True,
        text=True,
        capture_output=True,
    )
    actual = result.stdout.strip()
    if actual != source.rev:
        raise ValueError(f"{source.name} is at {actual}, expected {source.rev}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="only validate spec-lock.toml and an existing reference store",
    )
    parser.add_argument(
        "--lock-only",
        action="store_true",
        help="validate spec-lock.toml without requiring local checkouts",
    )
    args = parser.parse_args()

    store, sources = parse_spec_lock()
    if args.lock_only:
        print(f"validated {len(sources)} pinned Ethereum sources")
        return 0
    for source in sources:
        if args.check:
            verify_source(store, source)
        else:
            sync_source(store, source)
    print(store)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
