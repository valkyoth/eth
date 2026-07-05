#!/usr/bin/env python3
"""Workspace dependency-policy assertions that must not ship inside crates."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "Cargo.lock"
FORBIDDEN_PACKAGES = frozenset(("revm", "revm-primitives"))


def main() -> int:
    with LOCK.open("rb") as handle:
        lock = tomllib.load(handle)

    package_names = {
        package.get("name")
        for package in lock.get("package", ())
        if isinstance(package.get("name"), str)
    }
    forbidden = sorted(FORBIDDEN_PACKAGES.intersection(package_names))
    if forbidden:
        print(
            "forbidden workspace dependency present: " + ", ".join(forbidden),
            file=sys.stderr,
        )
        return 1

    print("workspace dependency policy excludes REVM packages")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
