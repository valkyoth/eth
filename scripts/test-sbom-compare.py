#!/usr/bin/env python3
"""Regression tests for semantic SPDX drift comparison."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

import compare_sbom


def write_document(path: Path, version: str, reverse: bool) -> None:
    """Write a small SPDX-shaped fixture with controlled nondeterminism."""
    packages = [
        {"SPDXID": "SPDXRef-Package-eth", "versionInfo": version},
        {"SPDXID": "SPDXRef-Package-codec", "versionInfo": "1.0.0"},
    ]
    if reverse:
        packages.reverse()
    document = {
        "SPDXID": "SPDXRef-DOCUMENT",
        "creationInfo": {
            "created": "2026-07-13T00:00:01Z" if reverse else "2026-07-13T00:00:00Z",
            "creators": ["Tool: cargo-sbom-v0.10.0"],
        },
        "documentNamespace": f"https://spdx.org/spdxdocs/{'second' if reverse else 'first'}",
        "packages": packages,
    }
    path.write_text(json.dumps(document), encoding="utf-8")


def main() -> int:
    """Exercise ignored generator metadata and detected package drift."""
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        expected = root / "expected.json"
        reordered = root / "reordered.json"
        drifted = root / "drifted.json"
        write_document(expected, "0.52.0", reverse=False)
        write_document(reordered, "0.52.0", reverse=True)
        write_document(drifted, "0.51.0", reverse=True)

        assert compare_sbom.documents_match(expected, reordered)
        assert not compare_sbom.documents_match(expected, drifted)

    print("SBOM comparison tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
