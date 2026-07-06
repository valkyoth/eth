#!/usr/bin/env python3
"""Regression tests for release metadata gate invariants."""

from __future__ import annotations

import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "validate-release-metadata.sh"


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def main() -> int:
    release = load_toml(ROOT / "release-crates.toml")
    eth_manifest = load_toml(ROOT / "crates" / "eth" / "Cargo.toml")

    release_version = release["release"]["version"]
    eth_version = eth_manifest["package"]["version"]
    assert release_version == eth_version

    validator = VALIDATOR.read_text(encoding="utf-8")
    assert "release_version=" in validator
    assert "eth_manifest_version=" in validator
    assert "test \"$release_version\" = \"$eth_manifest_version\"" in validator
    assert 'current_pentest_report="security/pentest/v${release_version}.md"' in validator
    assert 'test -f "$current_pentest_report"' in validator
    assert "grep -q '^Status: PASS$' \"$current_pentest_report\"" in validator

    print("release metadata tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
