#!/usr/bin/env python3
"""Tests for the runtime dependency policy checker."""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_runtime_dependency_policy.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("runtime_dependency_policy", CHECKER)
    if spec is None or spec.loader is None:
        raise RuntimeError("failed to load check_runtime_dependency_policy.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    checker = load_checker()
    command = tuple(checker.DEFAULT_RUNTIME_TREE_COMMAND)

    assert "--no-default-features" not in command
    assert command == (
        "cargo",
        "tree",
        "-p",
        "eth",
        "-e",
        "normal",
        "--prefix",
        "none",
    )
    assert "k256" in checker.DEFAULT_FORBIDDEN
    assert "alloy-rlp" in checker.DEFAULT_FORBIDDEN
    assert "serde_json" in checker.DEFAULT_FORBIDDEN

    print("runtime dependency policy tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
