#!/usr/bin/env python3
"""Tests for the Ethereum upstream advisory checker."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import SimpleNamespace


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_ethereum_upstream.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_ethereum_upstream", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load check_ethereum_upstream.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def test_unknown_rust_version_is_not_msrv_evidence() -> None:
    assert not checker.version_leq(None, "1.90.0")


def test_invalid_rust_version_is_not_msrv_evidence() -> None:
    assert not checker.version_leq("not-a-version", "1.90.0")


def test_remote_head_rejects_empty_ls_remote_output() -> None:
    original_run = checker.subprocess.run

    def fake_run(*_args, **_kwargs):
        return SimpleNamespace(stdout="")

    checker.subprocess.run = fake_run
    try:
        try:
            checker.remote_head("https://github.com/ethereum/EIPs")
        except RuntimeError as exc:
            assert "returned no ls-remote output for HEAD" in str(exc)
        else:
            raise AssertionError("expected RuntimeError for empty ls-remote output")
    finally:
        checker.subprocess.run = original_run


def run_tests() -> None:
    test_unknown_rust_version_is_not_msrv_evidence()
    test_invalid_rust_version_is_not_msrv_evidence()
    test_remote_head_rejects_empty_ls_remote_output()


if __name__ == "__main__":
    run_tests()
