#!/usr/bin/env python3
"""Regression tests for release metadata gate invariants."""

from __future__ import annotations

import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "validate-release-metadata.sh"
RELEASE_WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"


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
    assert "tomllib.load" in validator
    assert '["release"]["version"]' in validator
    assert '["package"]["version"]' in validator
    assert "sed -n" not in validator
    assert "test \"$release_version\" = \"$eth_manifest_version\"" in validator
    assert "test ! -f PENTEST.md" in validator
    assert 'current_pentest_report="security/pentest/v${release_version}.md"' not in validator
    assert 'test -f "$current_pentest_report"' not in validator
    assert "grep -q '^Status: PASS$' \"$current_pentest_report\"" not in validator
    assert "test -f .github/workflows/release.yml" in validator
    assert "workflow_dispatch:" in validator
    assert "Validate release metadata" in validator
    assert "scripts/validate-release-metadata.sh" in validator
    assert "! grep -q 'tags:' .github/workflows/release.yml" in validator
    assert "validate-release-readiness.sh" in validator
    assert "fetch-depth: 0" in validator

    release_workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    assert "workflow_dispatch:" in release_workflow
    assert "tags:" not in release_workflow
    assert '"v*"' not in release_workflow
    assert "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0" in release_workflow
    assert "fetch-depth: 0" in release_workflow
    assert "Validate release metadata" in release_workflow
    assert "scripts/validate-release-metadata.sh" in release_workflow
    assert "validate-release-readiness.sh" not in release_workflow

    print("release metadata tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
