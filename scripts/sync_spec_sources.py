#!/usr/bin/env python3
"""Synchronize pinned Ethereum reference repositories outside this repo."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC_LOCK = ROOT / "spec-lock.toml"
ALLOWED_REPO = re.compile(r"^https://github\.com/ethereum/[A-Za-z0-9_.-]+$")
GIT_TIMEOUT_SECONDS = 120
GIT_STATUS_TIMEOUT_SECONDS = 30
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


def validate_repo_url(name: str, repo: object) -> str:
    if not isinstance(repo, str) or not ALLOWED_REPO.fullmatch(repo):
        raise ValueError(f"{name}_repo must be an official Ethereum HTTPS repository")
    return repo


def validate_revision(name: str, rev: object) -> str:
    if not isinstance(rev, str) or len(rev) != 40 or any(char not in "0123456789abcdef" for char in rev):
        raise ValueError(f"{name}_rev must be a lowercase 40-character commit hash")
    return rev


def parse_spec_lock() -> tuple[Path, list[Source]]:
    values = tomllib.loads(SPEC_LOCK.read_text(encoding="utf-8")).get("ethereum")
    if not isinstance(values, dict):
        raise ValueError("spec-lock.toml is missing [ethereum]")

    default_store = values.get("local_reference_store_default")
    if not isinstance(default_store, str) or not default_store:
        raise ValueError("spec-lock.toml is missing local_reference_store_default")
    env_name = values.get("local_reference_store_env", "")
    if env_name and not isinstance(env_name, str):
        raise ValueError("local_reference_store_env must be a string")
    override = os.environ.get(env_name) if env_name else None
    store = Path(override).resolve() if override else (ROOT / default_store).resolve()

    sources: list[Source] = []
    for name in REPO_KEYS:
        repo = values.get(f"{name}_repo")
        rev = values.get(f"{name}_rev")
        if not repo or not rev:
            raise ValueError(f"spec-lock.toml is missing {name}_repo or {name}_rev")
        sources.append(Source(name=name, repo=validate_repo_url(name, repo), rev=validate_revision(name, rev)))
    return store, sources


def git_env() -> dict[str, str]:
    return {**os.environ, "GIT_ALLOW_PROTOCOL": "https", "GIT_TERMINAL_PROMPT": "0"}


def harden_git_args(args: list[str]) -> list[str]:
    if not args or args[0] != "git":
        return args
    return [
        "git",
        "-c",
        "core.fsmonitor=",
        "-c",
        f"core.hooksPath={os.devnull}",
        "--no-optional-locks",
        *args[1:],
    ]


def run(args: list[str], cwd: Path | None = None) -> None:
    subprocess.run(harden_git_args(args), cwd=cwd, check=True, env=git_env(), timeout=GIT_TIMEOUT_SECONDS)


def git_stdout(args: list[str], cwd: Path) -> str:
    result = subprocess.run(
        harden_git_args(args),
        cwd=cwd,
        check=True,
        text=True,
        capture_output=True,
        env=git_env(),
        timeout=GIT_STATUS_TIMEOUT_SECONDS,
    )
    return result.stdout.strip()


def sync_source(store: Path, source: Source) -> None:
    checkout = store / source.name
    if checkout.exists():
        run(["git", "remote", "set-url", "origin", source.repo], cwd=checkout)
        run(["git", "fetch", "--tags", "--prune", "origin"], cwd=checkout)
    else:
        checkout.parent.mkdir(parents=True, exist_ok=True)
        run(["git", "clone", "--", source.repo, str(checkout)])
    run(["git", "checkout", "--detach", source.rev], cwd=checkout)


def verify_source(store: Path, source: Source) -> None:
    checkout = store / source.name
    if not checkout.exists():
        raise FileNotFoundError(f"{checkout} does not exist; run without --check")
    origin = git_stdout(["git", "remote", "get-url", "origin"], cwd=checkout)
    if origin != source.repo:
        raise ValueError(f"{source.name} origin is {origin}, expected {source.repo}")
    actual = git_stdout(["git", "rev-parse", "HEAD"], cwd=checkout)
    if actual != source.rev:
        raise ValueError(f"{source.name} is at {actual}, expected {source.rev}")
    dirty = git_stdout(["git", "status", "--porcelain"], cwd=checkout)
    if dirty:
        raise ValueError(f"{source.name} checkout has uncommitted modifications")


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
