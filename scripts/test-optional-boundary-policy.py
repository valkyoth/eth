#!/usr/bin/env python3
"""Tests for the optional parser and sanitization boundary checker."""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_optional_boundary_policy.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("optional_boundary_policy", CHECKER)
    if spec is None or spec.loader is None:
        raise RuntimeError("failed to load check_optional_boundary_policy.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    checker = load_checker()

    assert "serde" in checker.DEFAULT_FORBIDDEN
    assert "serde_json" in checker.DEFAULT_FORBIDDEN
    assert "sanitization" in checker.DEFAULT_FORBIDDEN
    assert "eth-valkyoth-sanitization" in checker.DEFAULT_FORBIDDEN
    assert checker.JSON_REQUIRED == frozenset(("serde", "serde_json"))
    assert checker.SANITIZATION_REQUIRED == frozenset(("eth-valkyoth-sanitization", "sanitization"))
    assert checker.dependency_feature_enabled(
        {"package": "serde_json", "features": ["unbounded_depth"]},
        "serde_json",
        "unbounded_depth",
    )
    assert not checker.dependency_feature_enabled(
        {"package": "serde_json", "features": ["preserve_order"]},
        "serde_json",
        "unbounded_depth",
    )

    print("optional boundary policy tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
