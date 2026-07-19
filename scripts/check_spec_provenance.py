#!/usr/bin/env python3
"""Verify that documentation cites the revision pinned in spec-lock.toml."""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "spec-lock.toml"
MPT_DOC = ROOT / "docs" / "mpt-nodes.md"
MPT_REVISION = re.compile(
    r"`spec-lock\.toml` pins `ethereum/execution-specs` at\s+"
    r"`([0-9a-f]{40})`"
)


def main() -> int:
    with LOCK.open("rb") as handle:
        lock = tomllib.load(handle)
    expected = lock["ethereum"]["execution_specs_rev"]
    documented = MPT_REVISION.search(MPT_DOC.read_text(encoding="utf-8"))
    if documented is None:
        print("error: docs/mpt-nodes.md has no execution-specs source pin", file=sys.stderr)
        return 1
    if documented.group(1) != expected:
        print(
            "error: docs/mpt-nodes.md execution-specs revision does not match "
            "spec-lock.toml",
            file=sys.stderr,
        )
        return 1
    print("documented MPT source revision matches spec-lock.toml")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
