#!/usr/bin/env python3
"""Advisory upstream drift check for Ethereum and REVM sources."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC_LOCK = ROOT / "spec-lock.toml"
USER_AGENT = "valkyoth-eth-upstream-check/0.37.2 (https://github.com/valkyoth/eth)"
CRATES_IO = "https://crates.io/api/v1/crates"
WORKSPACE_MSRV = "1.90.0"
ALLOWED_REPO = re.compile(r"^https://github\.com/ethereum/[A-Za-z0-9_.-]+$")
SHA40 = re.compile(r"^[0-9a-f]{40}$")
GIT_TIMEOUT_SECONDS = 45
REPO_KEYS = (
    "execution_specs",
    "execution_tests",
    "eips",
    "execution_apis",
    "consensus_specs",
)


@dataclass(frozen=True)
class CrateReview:
    name: str
    reviewed_latest: str
    reviewed_latest_rust: str
    reviewed_msrv_compatible: str


@dataclass(frozen=True)
class SourcePin:
    name: str
    repo: str
    rev: str


CRATE_REVIEWS = (
    CrateReview("revm", "41.0.0", "1.91.0", "36.0.0"),
    CrateReview("revm-primitives", "41.0.0", "1.91.0", "22.1.0"),
)


def fail(message: str) -> int:
    print(f"error: {message}", file=sys.stderr)
    return 1


def parse_version(version: str | None) -> tuple[int, ...]:
    if not version:
        return ()
    core = version.split("-", 1)[0]
    parts: list[int] = []
    for part in core.split("."):
        if not part.isdigit():
            return ()
        parts.append(int(part))
    return tuple(parts)


def version_leq(left: str | None, right: str) -> bool:
    if left is None:
        return False
    left_parts = parse_version(left)
    right_parts = parse_version(right)
    if not left_parts or not right_parts:
        return False
    length = max(len(left_parts), len(right_parts))
    return left_parts + (0,) * (length - len(left_parts)) <= right_parts + (
        0,
    ) * (length - len(right_parts))


def version_sort_key(version: str) -> tuple[int, ...]:
    parsed = parse_version(version)
    if not parsed:
        raise ValueError(f"unsupported version format: {version}")
    return parsed


def fetch_json(url: str) -> dict:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=45) as response:
            payload = response.read()
    except urllib.error.URLError as exc:
        raise RuntimeError(f"could not fetch {url}: {exc}") from exc
    data = json.loads(payload.decode("utf-8"))
    if not isinstance(data, dict):
        raise RuntimeError(f"{url} did not return a JSON object")
    return data


def crate_metadata(name: str) -> tuple[str, str, str]:
    data = fetch_json(f"{CRATES_IO}/{name}")
    crate = data.get("crate")
    versions = data.get("versions")
    if not isinstance(crate, dict) or not isinstance(versions, list):
        raise RuntimeError(f"crates.io metadata for {name} is missing fields")

    latest = crate.get("newest_version")
    if not isinstance(latest, str):
        raise RuntimeError(f"crates.io metadata for {name} has no newest version")

    version_rows = [row for row in versions if isinstance(row, dict)]
    latest_row = next((row for row in version_rows if row.get("num") == latest), None)
    if latest_row is None:
        raise RuntimeError(f"crates.io metadata for {name} has no row for {latest}")
    latest_rust = latest_row.get("rust_version")
    if latest_rust is not None and not isinstance(latest_rust, str):
        raise RuntimeError(f"crates.io metadata for {name} has invalid rust_version")

    compatible = [
        row["num"]
        for row in version_rows
        if isinstance(row.get("num"), str)
        and not row.get("yanked", False)
        and version_leq(row.get("rust_version"), WORKSPACE_MSRV)
    ]
    if not compatible:
        raise RuntimeError(f"crates.io metadata for {name} has no MSRV-compatible version")

    newest_compatible = max(compatible, key=version_sort_key)
    return latest, latest_rust or "unspecified", newest_compatible


def load_source_pins() -> list[SourcePin]:
    values = tomllib.loads(SPEC_LOCK.read_text(encoding="utf-8")).get("ethereum")
    if not isinstance(values, dict):
        raise RuntimeError("spec-lock.toml is missing [ethereum]")

    pins: list[SourcePin] = []
    for name in REPO_KEYS:
        repo = values.get(f"{name}_repo")
        rev = values.get(f"{name}_rev")
        if not isinstance(repo, str) or not ALLOWED_REPO.fullmatch(repo):
            raise RuntimeError(f"{name}_repo must be an official Ethereum HTTPS repository")
        if not isinstance(rev, str) or not SHA40.fullmatch(rev):
            raise RuntimeError(f"{name}_rev must be a lowercase 40-character commit hash")
        pins.append(SourcePin(name, repo, rev))
    return pins


def git_env() -> dict[str, str]:
    return {**os.environ, "GIT_ALLOW_PROTOCOL": "https", "GIT_TERMINAL_PROMPT": "0"}


def remote_head(repo: str) -> str:
    result = subprocess.run(
        [
            "git",
            "-c",
            "core.fsmonitor=",
            "-c",
            f"core.hooksPath={os.devnull}",
            "--no-optional-locks",
            "ls-remote",
            "--",
            repo,
            "HEAD",
        ],
        check=True,
        capture_output=True,
        text=True,
        env=git_env(),
        timeout=GIT_TIMEOUT_SECONDS,
    )
    tokens = result.stdout.split()
    if not tokens:
        raise RuntimeError(f"{repo} returned no ls-remote output for HEAD")
    head = tokens[0]
    if not SHA40.fullmatch(head):
        raise RuntimeError(f"{repo} returned an invalid HEAD hash")
    return head


def print_crate_report() -> None:
    print("REVM registry check:")
    for review in CRATE_REVIEWS:
        latest, latest_rust, newest_compatible = crate_metadata(review.name)
        print(
            f"- {review.name}: latest={latest} rust={latest_rust}; "
            f"newest-msrv-{WORKSPACE_MSRV}={newest_compatible}"
        )
        if latest != review.reviewed_latest:
            print(f"  notice: reviewed latest was {review.reviewed_latest}")
        if latest_rust != review.reviewed_latest_rust:
            print(f"  notice: reviewed latest rust-version was {review.reviewed_latest_rust}")
        if newest_compatible != review.reviewed_msrv_compatible:
            print(
                "  notice: reviewed newest MSRV-compatible version was "
                f"{review.reviewed_msrv_compatible}"
            )


def print_source_report() -> None:
    print("Official Ethereum source check:")
    for source in load_source_pins():
        head = remote_head(source.repo)
        status = "current" if head == source.rev else "remote-head-moved"
        print(f"- {source.name}: pinned={source.rev} remote_head={head} status={status}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--local-only",
        action="store_true",
        help="validate local source policy without network requests",
    )
    args = parser.parse_args()

    try:
        pins = load_source_pins()
        if args.local_only:
            print(f"validated {len(pins)} official Ethereum source pins")
            return 0
        print("Ethereum upstream advisory check")
        print("fetched data is used only as metadata and is never executed")
        print_crate_report()
        print_source_report()
    except (RuntimeError, ValueError, subprocess.SubprocessError, json.JSONDecodeError) as exc:
        return fail(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
