#!/usr/bin/env python3
"""Fail when a direct crates.io dependency is not latest for the workspace MSRV."""

from __future__ import annotations

import json
import re
import tomllib
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
USER_AGENT = "valkyoth-eth-crate-check/0.53.0 (https://github.com/valkyoth/eth)"
VERSION = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")


def numeric_version(value: str) -> tuple[int, int, int] | None:
    match = VERSION.fullmatch(value)
    if match is None:
        return None
    return tuple(int(part) for part in match.groups())


def dependency_tables(value: object, name: str = ""):
    if not isinstance(value, dict):
        return
    if name in {"dependencies", "dev-dependencies", "build-dependencies"}:
        yield value
        return
    for child_name, child in value.items():
        yield from dependency_tables(child, child_name)


def direct_dependencies() -> dict[str, str]:
    dependencies: dict[str, str] = {}
    for manifest in ROOT.rglob("Cargo.toml"):
        if any(part in {".git", "target"} for part in manifest.parts):
            continue
        with manifest.open("rb") as source:
            document = tomllib.load(source)
        for table in dependency_tables(document):
            for name, specification in table.items():
                if isinstance(specification, str):
                    version = specification
                elif isinstance(specification, dict):
                    if specification.get("workspace") or specification.get("path"):
                        continue
                    version = specification.get("version")
                else:
                    continue
                if not isinstance(version, str):
                    continue
                if numeric_version(version) is None:
                    raise ValueError(
                        f"{manifest}: {name} must use an exact three-part version"
                    )
                previous = dependencies.setdefault(name, version)
                if previous != version:
                    raise ValueError(
                        f"{name} has conflicting direct versions: {previous} and {version}"
                    )
    return dependencies


def crate_metadata(name: str) -> dict[str, object]:
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{name}",
        headers={"User-Agent": USER_AGENT},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except (OSError, urllib.error.URLError, json.JSONDecodeError) as error:
        raise RuntimeError(f"could not query crates.io metadata for {name}: {error}") from error


def latest_compatible(metadata: dict[str, object], msrv: tuple[int, int, int]) -> str:
    candidates: list[tuple[tuple[int, int, int], str]] = []
    versions = metadata.get("versions")
    if not isinstance(versions, list):
        raise ValueError("crates.io response has no versions list")
    for entry in versions:
        if not isinstance(entry, dict) or entry.get("yanked"):
            continue
        number = entry.get("num")
        if not isinstance(number, str):
            continue
        parsed = numeric_version(number)
        if parsed is None:
            continue
        rust_version = entry.get("rust_version")
        if isinstance(rust_version, str):
            rust = numeric_version(
                rust_version if rust_version.count(".") == 2 else f"{rust_version}.0"
            )
            if rust is None or rust > msrv:
                continue
        candidates.append((parsed, number))
    if not candidates:
        raise ValueError("crates.io response has no stable MSRV-compatible version")
    return max(candidates)[1]


def main() -> int:
    with (ROOT / "Cargo.toml").open("rb") as source:
        rust_version = tomllib.load(source)["workspace"]["package"]["rust-version"]
    parsed_msrv = numeric_version(
        rust_version if rust_version.count(".") == 2 else f"{rust_version}.0"
    )
    if parsed_msrv is None:
        raise ValueError(f"invalid workspace rust-version: {rust_version}")

    stale: list[str] = []
    dependencies = direct_dependencies()
    for name, declared in sorted(dependencies.items()):
        latest = latest_compatible(crate_metadata(name), parsed_msrv)
        if declared != latest:
            stale.append(f"{name}: declared={declared} latest-msrv={latest}")
        else:
            print(f"{name}: {declared}")
    if stale:
        raise SystemExit("outdated direct crates.io dependencies:\n- " + "\n- ".join(stale))
    print(f"all {len(dependencies)} direct crates.io dependencies are current")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
